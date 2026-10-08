//! Bounded, time-limited private headless processes. Never interprets shell text.

use std::{
    io::Read,
    os::{
        fd::AsRawFd,
        unix::process::{CommandExt, ExitStatusExt},
    },
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

pub struct Outcome {
    pub success: bool,
    pub exit_code: i32,
    pub timed_out: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub duration_s: f64,
}

pub fn inkscape_binary() -> Option<PathBuf> {
    binary("inkscape")
}

pub fn binary(name: &str) -> Option<PathBuf> {
    let mut candidates: Vec<_> = std::env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .map(|directory| directory.join(name))
        .collect();
    // Ready bundles carry fixed private dependencies; development/fake PATH
    // behavior stays unchanged when no package manifest is present.
    if let Ok(library) = crate::live_launch::library()
        && std::fs::symlink_metadata(library.join("package.json")).is_ok_and(|meta| meta.is_file())
    {
        match name {
            "gdbus" | "dbus-daemon" => candidates.push(crate::runtime_layout::asset(
                &library,
                &format!("dbus/bin/{name}"),
            )),
            "inkscape" if cfg!(target_os = "macos") => candidates.push(PathBuf::from(
                "/Applications/Inkscape.app/Contents/MacOS/inkscape",
            )),
            "inkscape" if cfg!(target_os = "linux") => candidates.extend([
                PathBuf::from("/usr/local/bin/inkscape"),
                PathBuf::from("/usr/bin/inkscape"),
            ]),
            _ => {}
        }
    }
    candidates.into_iter().find(|path| {
        path.is_file()
            && std::ffi::CString::new(path.as_os_str().as_encoded_bytes())
                .is_ok_and(|s| unsafe { libc::access(s.as_ptr(), libc::X_OK) } == 0)
    })
}

pub fn timeout() -> Duration {
    timeout_from(
        std::env::var("INKSCAPE_MCP_PROCESS_TIMEOUT_S")
            .ok()
            .as_deref(),
    )
}
fn timeout_from(raw: Option<&str>) -> Duration {
    let seconds = raw
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s >= 1.0)
        .unwrap_or(60.0);
    Duration::from_secs_f64(seconds.min(86400.0))
}

fn drain<T: Read + AsRawFd + Send + 'static>(
    mut pipe: T,
    done: Arc<AtomicBool>,
    limit: usize,
) -> thread::JoinHandle<Result<Vec<u8>, &'static str>> {
    thread::spawn(move || {
        // SAFETY: only this newly-owned child pipe's file status flags are changed.
        let flags = unsafe { libc::fcntl(pipe.as_raw_fd(), libc::F_GETFL) };
        if flags < 0
            || unsafe { libc::fcntl(pipe.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
        {
            // Never enter a potentially blocking read when configuring the pipe failed.
            return Err("process pipe could not be made nonblocking");
        }
        let mut retained = Vec::new();
        let mut buffer = [0u8; 8192];
        let mut completed_at = None;
        loop {
            if done.load(Ordering::Acquire) {
                let completed_at = completed_at.get_or_insert_with(Instant::now);
                // Drain buffered output after the owned leader exits, but a descendant
                // can keep this pipe continuously readable. Waiting only for WouldBlock
                // would then defeat the process deadline and keep join() blocked forever.
                if completed_at.elapsed() >= Duration::from_millis(100) {
                    break;
                }
            }
            match pipe.read(&mut buffer) {
                Ok(0) => break,
                Ok(size) => {
                    let count = size.min(limit.saturating_sub(retained.len()));
                    retained.extend_from_slice(&buffer[..count]);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if done.load(Ordering::Acquire) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
        Ok(retained)
    })
}

pub fn run(binary: &PathBuf, arguments: &[String], timeout: Duration) -> Result<Outcome, String> {
    run_bounded(binary, arguments, timeout, 65536)
}

pub fn run_bounded(
    binary: &PathBuf,
    arguments: &[String],
    timeout: Duration,
    limit: usize,
) -> Result<Outcome, String> {
    run_bounded_with_env(binary, arguments, timeout, limit, &[])
}

/// Fixed internal callers may isolate process-owned HOME/profile/cache paths.
/// No MCP tool accepts environment overrides or arbitrary executable input.
pub fn run_bounded_with_env(
    binary: &PathBuf,
    arguments: &[String],
    timeout: Duration,
    limit: usize,
    environment: &[(&str, &std::ffi::OsStr)],
) -> Result<Outcome, String> {
    if cancelled() {
        return Err("operation cancelled".into());
    }
    let start = Instant::now();
    let mut command = Command::new(binary);
    command.envs(environment.iter().copied());
    if let Ok(library) = crate::live_launch::library()
        && binary == &crate::runtime_layout::asset(&library, "dbus/bin/gdbus")
    {
        command.env("GIO_MODULE_DIR", library.join("dbus/lib/gio/modules"));
        command.env_remove("GIO_EXTRA_MODULES");
    }
    let mut child = command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|_| {
            crate::telemetry::failure(crate::telemetry::Failure::ProcessStart);
            "Inkscape process could not be started"
        })?;
    let done = Arc::new(AtomicBool::new(false));
    let stdout = drain(
        child.stdout.take().ok_or("process stdout unavailable")?,
        done.clone(),
        limit,
    );
    let stderr = drain(
        child.stderr.take().ok_or("process stderr unavailable")?,
        done.clone(),
        65536,
    );
    let mut timed_out = false;
    let mut exit_code = -1;
    let observed = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                exit_code = status
                    .code()
                    .unwrap_or_else(|| -status.signal().unwrap_or(1));
                break Ok(status.success());
            }
            Ok(None) if cancelled() || start.elapsed() >= timeout => {
                timed_out = !cancelled();
                // SAFETY: negative pid refers ONLY to the new private process group above.
                // No existing GUI/session process belongs to this group.
                unsafe {
                    libc::kill(-(child.id() as i32), libc::SIGKILL);
                }
                break child
                    .wait()
                    .map(|_| false)
                    .map_err(|_| "could not reap timed-out process");
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(_) => {
                // Cleanup also covers observation errors: the exact owned child
                // and private group are reaped before joining bounded pipe readers.
                unsafe {
                    libc::kill(-(child.id() as i32), libc::SIGKILL);
                }
                let _ = child.wait();
                break Err("could not observe Inkscape process");
            }
        }
    };
    let was_cancelled = cancelled();
    done.store(true, Ordering::Release);
    let stdout = stdout.join();
    let stderr = stderr.join();
    if was_cancelled {
        return Err("operation cancelled".into());
    }
    if timed_out {
        crate::telemetry::failure(crate::telemetry::Failure::ProcessTimeout);
    } else if exit_code < 0 && observed.is_ok() {
        crate::telemetry::failure(crate::telemetry::Failure::ProcessCrash);
    }
    let success = observed?;
    let stdout = stdout.map_err(|_| "process stdout reader failed")??;
    let stderr = stderr.map_err(|_| "process stderr reader failed")??;
    Ok(Outcome {
        success,
        exit_code,
        timed_out,
        stdout,
        stderr,
        duration_s: start.elapsed().as_secs_f64(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuously_readable_pipe_cannot_extend_completed_process_indefinitely() {
        struct Continuous {
            file: std::fs::File,
        }
        impl AsRawFd for Continuous {
            fn as_raw_fd(&self) -> std::os::fd::RawFd {
                self.file.as_raw_fd()
            }
        }
        impl Read for Continuous {
            fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
                bytes.fill(b'x');
                Ok(bytes.len())
            }
        }
        let reader = Continuous {
            file: std::fs::File::open("/dev/null").unwrap(),
        };
        let started = Instant::now();
        let handle = drain(reader, Arc::new(AtomicBool::new(true)), 16384);
        assert_eq!(handle.join().unwrap().unwrap(), vec![b'x'; 16384]);
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn completed_process_retains_buffered_output_before_eof() {
        struct Buffered {
            file: std::fs::File,
            data: std::io::Cursor<Vec<u8>>,
        }
        impl AsRawFd for Buffered {
            fn as_raw_fd(&self) -> std::os::fd::RawFd {
                self.file.as_raw_fd()
            }
        }
        impl Read for Buffered {
            fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
                self.data.read(bytes)
            }
        }
        let expected = vec![b'x'; 32768];
        let reader = Buffered {
            file: std::fs::File::open("/dev/null").unwrap(),
            data: std::io::Cursor::new(expected.clone()),
        };
        let handle = drain(reader, Arc::new(AtomicBool::new(true)), expected.len());
        assert_eq!(handle.join().unwrap().unwrap(), expected);
    }

    #[test]
    fn failed_nonblocking_setup_never_enters_read() {
        struct Invalid;
        impl AsRawFd for Invalid {
            fn as_raw_fd(&self) -> std::os::fd::RawFd {
                -1
            }
        }
        impl Read for Invalid {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                panic!("read entered after nonblocking setup failed")
            }
        }
        let handle = drain(Invalid, Arc::new(AtomicBool::new(false)), 16384);
        assert_eq!(
            handle.join().unwrap(),
            Err("process pipe could not be made nonblocking")
        );
    }

    #[test]
    fn timeout_floor_matches_reference_and_nonfinite_values_remain_bounded() {
        for value in [
            None,
            Some("0.1"),
            Some("0"),
            Some("garbage"),
            Some("nan"),
            Some("inf"),
        ] {
            assert_eq!(timeout_from(value), Duration::from_secs(60));
        }
        assert_eq!(timeout_from(Some("1")), Duration::from_secs(1));
        assert_eq!(timeout_from(Some(" 2.5 ")), Duration::from_millis(2500));
        assert_eq!(timeout_from(Some("100000")), Duration::from_secs(86400));
    }
    #[test]
    fn bounded_output_and_timeout_are_observed() {
        let output = run(
            &PathBuf::from("/usr/bin/yes"),
            &[],
            Duration::from_millis(40),
        )
        .unwrap();
        assert!(output.timed_out);
        assert!(!output.success);
        assert!(output.stdout.len() <= 65536);
        assert!(output.duration_s < 2.0);
    }
}

// Scoped to a single blocking request, never propagated to unrelated GUI processes.
thread_local! {
    static CANCELLATION: std::cell::RefCell<Option<Box<dyn Fn() -> bool>>> = const { std::cell::RefCell::new(None) };
}
pub fn cancelled() -> bool {
    CANCELLATION.with(|slot| slot.borrow().as_ref().is_some_and(|check| check()))
}
pub struct CancelOnDrop(pub Arc<AtomicBool>);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}
pub fn with_cancellation<T>(check: impl Fn() -> bool + 'static, work: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            CANCELLATION.with(|slot| *slot.borrow_mut() = None);
        }
    }
    CANCELLATION.with(|slot| *slot.borrow_mut() = Some(Box::new(check)));
    let _reset = Reset;
    work()
}
pub fn lock_cancelable<T>(
    lock: &std::sync::Mutex<T>,
) -> Result<std::sync::MutexGuard<'_, T>, String> {
    loop {
        if cancelled() {
            return Err("operation cancelled".into());
        }
        match lock.try_lock() {
            Ok(guard) => return Ok(guard),
            Err(std::sync::TryLockError::Poisoned(_)) => return Err("operation lock failed".into()),
            Err(std::sync::TryLockError::WouldBlock) => thread::sleep(Duration::from_millis(5)),
        }
    }
}
