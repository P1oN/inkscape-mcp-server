//! Private bounded headless shell pool. Commands come only from typed internal kernels.
//! Staged inputs are reopened for every operation; working files never reach the child.
use std::{
    collections::VecDeque,
    io::{Read, Write},
    os::{fd::AsRawFd, unix::process::ExitStatusExt},
    path::{Path, PathBuf},
    process::{Child, ChildStderr, ChildStdout, Command, Stdio},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

const PIPE_CAP: usize = 65_536;
const COMMAND_CAP: usize = 1_048_576;

fn nonblocking(fd: i32) -> Result<(), ()> {
    // SAFETY: fixed fcntl operations on newly owned subprocess pipe descriptors.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(());
    }
    Ok(())
}
fn wait_ready(fds: &mut [libc::pollfd], deadline: Instant) -> Result<(), ()> {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            crate::telemetry::failure(crate::telemetry::Failure::ProcessTimeout);
            return Err(());
        }
        let milliseconds = remaining
            .as_millis()
            .saturating_add(1)
            .min(i32::MAX as u128) as i32;
        // SAFETY: poll receives this live slice of owned pipe descriptors and a
        // bounded deadline; no descriptor ownership is transferred.
        let result =
            unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, milliseconds) };
        if result > 0 {
            return if fds.iter().any(|fd| fd.revents & libc::POLLNVAL != 0) {
                Err(())
            } else {
                Ok(())
            };
        }
        if result == 0 {
            crate::telemetry::failure(crate::telemetry::Failure::ProcessTimeout);
            return Err(());
        }
        if std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted {
            return Err(());
        }
    }
}
fn watch(fd: i32, events: i16) -> libc::pollfd {
    libc::pollfd {
        fd,
        events,
        revents: 0,
    }
}
fn safe_path(path: &Path) -> Result<String, ()> {
    let text = path.to_str().ok_or(())?;
    if text.contains([';', '\n', '\r']) || text.len() > COMMAND_CAP / 4 {
        return Err(());
    }
    Ok(text.to_owned())
}
fn drain(pipe: &mut impl Read, target: &mut Vec<u8>) -> Result<bool, ()> {
    let mut chunk = [0u8; 4096];
    loop {
        match pipe.read(&mut chunk) {
            Ok(0) => return Ok(true),
            Ok(n) => {
                if target.len() + n > PIPE_CAP {
                    return Err(());
                }
                target.extend_from_slice(&chunk[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(()),
        }
    }
}
struct Worker {
    child: Child,
    stdout: ChildStdout,
    stderr: ChildStderr,
    used: Instant,
    binary: PathBuf,
    opened: bool,
}
impl Worker {
    fn start(binary: &Path, timeout: Duration) -> Result<Self, ()> {
        let mut child = Command::new(binary)
            .arg("--shell")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| {
                crate::telemetry::failure(crate::telemetry::Failure::ProcessStart);
            })?;
        let stdout = child.stdout.take().expect("owned stdout pipe");
        let stderr = child.stderr.take().expect("owned stderr pipe");
        let mut worker = Self {
            child,
            stdout,
            stderr,
            used: Instant::now(),
            binary: binary.to_owned(),
            opened: false,
        };
        nonblocking(worker.stdout.as_raw_fd())?;
        nonblocking(worker.stderr.as_raw_fd())?;
        nonblocking(
            worker
                .child
                .stdin
                .as_ref()
                .expect("owned stdin pipe")
                .as_raw_fd(),
        )?;
        worker.prompt(timeout)?;
        Ok(worker)
    }
    fn alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            Ok(Some(status)) => {
                if status.signal().is_some() {
                    crate::telemetry::failure(crate::telemetry::Failure::ProcessCrash);
                }
                false
            }
            Err(_) => false,
        }
    }
    fn prompt(&mut self, timeout: Duration) -> Result<(), ()> {
        let deadline = Instant::now() + timeout;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut stderr_open = true;
        loop {
            let eof = drain(&mut self.stdout, &mut stdout)?;
            // Drain synchronous stderr before accepting the stdout ready prompt.
            if stderr_open {
                stderr_open = !drain(&mut self.stderr, &mut stderr)?;
            }
            if stdout == b"> " || stdout.ends_with(b"\n> ") {
                if String::from_utf8_lossy(&stderr)
                    .contains("InkscapeApplication::parse_actions: could not find action for:")
                {
                    return Err(());
                }
                self.used = Instant::now();
                return Ok(());
            }
            if Instant::now() >= deadline {
                crate::telemetry::failure(crate::telemetry::Failure::ProcessTimeout);
                return Err(());
            }
            // Observe the owned child even on EOF so signal exits are reported.
            let alive = self.alive();
            if eof || !alive {
                return Err(());
            }
            wait_ready(
                &mut [
                    watch(self.stdout.as_raw_fd(), libc::POLLIN),
                    watch(
                        if stderr_open {
                            self.stderr.as_raw_fd()
                        } else {
                            -1
                        },
                        libc::POLLIN,
                    ),
                ],
                deadline,
            )?;
        }
    }
    fn execute(&mut self, command: &str, timeout: Duration) -> Result<(), ()> {
        if command.contains(['\n', '\r']) || command.len() > COMMAND_CAP || !self.alive() {
            return Err(());
        }
        let line = format!("{command}\n");
        let deadline = Instant::now() + timeout;
        let mut offset = 0;
        while offset < line.len() {
            match self
                .child
                .stdin
                .as_mut()
                .ok_or(())?
                .write(&line.as_bytes()[offset..])
            {
                Ok(0) => return Err(()),
                Ok(n) => offset += n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    wait_ready(
                        &mut [watch(
                            self.child.stdin.as_ref().ok_or(())?.as_raw_fd(),
                            libc::POLLOUT,
                        )],
                        deadline,
                    )?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => (),
                Err(_) => return Err(()),
            }
            if Instant::now() >= deadline {
                crate::telemetry::failure(crate::telemetry::Failure::ProcessTimeout);
                return Err(());
            }
        }
        self.prompt(deadline.saturating_duration_since(Instant::now()))
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(mut stdin) = self.child.stdin.take() {
            let _ = stdin.write_all(b"quit\n");
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(5));
                }
                _ => break,
            }
        }
        let _ = self.child.kill();
        let _ = self.child.wait(); // Only this exact owned headless shell child.
    }
}
#[derive(Clone, Copy)]
struct Limits {
    cap: usize,
    idle: Duration,
    timeout: Duration,
}
#[derive(Default)]
struct Pool {
    workers: VecDeque<(PathBuf, Worker)>,
}
impl Pool {
    fn run(
        &mut self,
        key: &Path,
        binary: &Path,
        input: &Path,
        lines: &[String],
        limits: Limits,
    ) -> Result<(), ()> {
        safe_path(key)?;
        let input = safe_path(input)?;
        self.workers.retain_mut(|(_, worker)| {
            worker.alive() && worker.used.elapsed() <= limits.idle && worker.binary == binary
        });
        let mut worker =
            if let Some(index) = self.workers.iter().position(|(stored, _)| stored == key) {
                self.workers.remove(index).ok_or(())?.1
            } else {
                while self.workers.len() >= limits.cap.max(1) {
                    self.workers.pop_front();
                }
                Worker::start(binary, limits.timeout)?
            };
        if worker.opened {
            worker.execute("file-close", limits.timeout)?;
        }
        // Always reopen the secure, operation-owned staged input. No stale in-memory edits.
        worker.execute(&format!("file-open:{input}"), limits.timeout)?;
        worker.opened = true;
        for line in lines {
            worker.execute(line, limits.timeout)?;
        }
        self.workers.push_back((key.to_owned(), worker));
        Ok(())
    }
}
static POOL: OnceLock<Mutex<Pool>> = OnceLock::new();
fn configured() -> bool {
    std::env::var("INKSCAPE_MCP_ENGINE_MODE").is_ok_and(|v| v.trim().eq_ignore_ascii_case("shell"))
}
fn run(key: &Path, binary: &Path, input: &Path, lines: &[String]) -> Result<(), ()> {
    if !configured() {
        return Err(());
    }
    let cap = std::env::var("INKSCAPE_MCP_ENGINE_MAX_PROCESSES")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v >= 1)
        .unwrap_or(2);
    let idle = std::env::var("INKSCAPE_MCP_ENGINE_IDLE_TIMEOUT_S")
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 1.0)
        .unwrap_or(300.0)
        .min(86400.0);
    POOL.get_or_init(|| Mutex::new(Pool::default()))
        .lock()
        .map_err(|_| ())?
        .run(
            key,
            binary,
            input,
            lines,
            Limits {
                cap,
                idle: Duration::from_secs_f64(idle),
                timeout: crate::process::timeout(),
            },
        )
}
fn export_line(output: &Path, fmt: &str, width: Option<i64>) -> Result<String, ()> {
    if !matches!(fmt, "png" | "svg") {
        return Err(());
    }
    let output = safe_path(output)?;
    Ok(format!(
        "export-type:{fmt}; {}export-width:{}; export-area-page; export-filename:{output}; export-do",
        if fmt == "svg" {
            "export-plain-svg; "
        } else {
            ""
        },
        width.unwrap_or(0)
    ))
}
/// Only whole-document PNG/SVG callers enter here. Failure selects the fixed CLI path.
pub(crate) fn export(
    key: &Path,
    binary: &Path,
    input: &Path,
    output: &Path,
    fmt: &str,
    width: Option<i64>,
) -> Result<(), ()> {
    run(key, binary, input, &[export_line(output, fmt, width)?])
}
/// Actions already passed the typed kernel's approval, capability and token gates.
pub(crate) fn actions(
    key: &Path,
    binary: &Path,
    input: &Path,
    output: &Path,
    actions: &str,
) -> Result<(), ()> {
    run(
        key,
        binary,
        input,
        &[actions.to_owned(), export_line(output, "svg", None)?],
    )
}
pub(crate) fn shutdown() {
    if let Some(pool) = POOL.get()
        && let Ok(mut pool) = pool.lock()
    {
        pool.workers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn fake() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("inkscape");
        let python = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(".venv/bin/python");
        let python = if python.is_file() {
            python
        } else {
            PathBuf::from("/usr/bin/python3")
        };
        std::fs::write(&binary, format!(r#"#!{}
import os,sys,time
sys.stdout.write('synthetic banner\n> ');sys.stdout.flush()
for line in sys.stdin:
    command=line.rstrip('\n')
    if command=='quit':break
    if command=='hang':time.sleep(10)
    if command=='crash':sys.exit(1)
    if command=='close-stderr':os.close(2)
    if command=='split':
        sys.stdout.write(command+'\n>');sys.stdout.flush();time.sleep(.01)
        sys.stdout.write(' ');sys.stdout.flush();continue
    if command=='unknown':
        sys.stderr.write('InkscapeApplication::parse_actions: could not find action for: unknown\n');sys.stderr.flush()
    if command=='flood':sys.stdout.write('x'*70000)
    sys.stdout.write(command+'\n> ');sys.stdout.flush()
"#, python.display())).unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        (dir, binary)
    }
    fn limits() -> Limits {
        Limits {
            cap: 2,
            idle: Duration::from_secs(60),
            timeout: Duration::from_secs(1),
        }
    }
    #[test]
    fn framed_reuse_force_reopen_and_bounded_lru_idle_reaping() {
        let (_dir, binary) = fake();
        let mut pool = Pool::default();
        let input = Path::new("/owned/input.svg");
        let a = Path::new("/owned/a.svg");
        let b = Path::new("/owned/b.svg");
        let c = Path::new("/owned/c.svg");
        pool.run(a, &binary, input, &["ok".into()], limits())
            .unwrap();
        let first = pool.workers[0].1.child.id();
        pool.run(a, &binary, input, &["ok-again".into()], limits())
            .unwrap();
        assert_eq!(pool.workers[0].1.child.id(), first);
        pool.run(b, &binary, input, &[], limits()).unwrap();
        pool.run(a, &binary, input, &[], limits()).unwrap();
        assert_eq!(pool.workers.back().unwrap().1.child.id(), first);
        pool.run(c, &binary, input, &[], limits()).unwrap();
        assert_eq!(pool.workers.len(), 2);
        assert!(!pool.workers.iter().any(|(key, _)| key == b));
        thread::sleep(Duration::from_millis(10));
        let short = Limits {
            idle: Duration::from_millis(1),
            ..limits()
        };
        pool.run(c, &binary, input, &[], short).unwrap();
        assert_eq!(pool.workers.len(), 1);
    }
    #[test]
    fn fragmented_prompt_and_closed_stderr_remain_bounded_and_framed() {
        let (_dir, binary) = fake();
        let mut worker = Worker::start(&binary, Duration::from_secs(1)).unwrap();
        for command in ["split", "close-stderr", "ok", "split"] {
            worker.execute(command, Duration::from_secs(1)).unwrap();
        }
        assert!(
            wait_ready(
                &mut [watch(i32::MAX, libc::POLLIN)],
                Instant::now() + Duration::from_millis(50)
            )
            .is_err()
        );
        assert!(wait_ready(&mut [watch(-1, libc::POLLIN)], Instant::now()).is_err());
    }
    #[test]
    fn faults_discard_worker_without_retry_and_paths_reject_action_separators() {
        let (_dir, binary) = fake();
        let mut pool = Pool::default();
        for command in ["unknown", "crash", "flood", "hang"] {
            let start = Instant::now();
            let bounded = Limits {
                timeout: Duration::from_millis(100),
                ..limits()
            };
            assert!(
                pool.run(
                    Path::new("/owned/a"),
                    &binary,
                    Path::new("/owned/in"),
                    &[command.into()],
                    bounded
                )
                .is_err()
            );
            assert!(pool.workers.is_empty());
            assert!(start.elapsed() < Duration::from_secs(4));
        }
        for path in ["/owned/a;quit", "/owned/a\nquit", "/owned/a\rquit"] {
            assert!(safe_path(Path::new(path)).is_err());
        }
        assert!(
            pool.run(
                Path::new("/owned/a"),
                &binary,
                Path::new("/owned/in"),
                &["a\nquit".into()],
                limits()
            )
            .is_err()
        );
        assert!(pool.workers.is_empty());
        assert_eq!(
            export_line(Path::new("/owned/out.png"), "png", None).unwrap(),
            "export-type:png; export-width:0; export-area-page; export-filename:/owned/out.png; export-do"
        );
        assert!(export_line(Path::new("/owned/out.pdf"), "pdf", None).is_err());
    }
}
