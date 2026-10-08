//! Explicit managed launch through a fixed packaged supervisor. No compiler/user Python.
use crate::{
    live_attach::{self, Attached},
    live_probe::Inputs,
    process,
    workspace::Workspace,
};
use std::{
    fs::File,
    os::{
        fd::AsRawFd,
        unix::{fs::MetadataExt, process::CommandExt},
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
const ERROR: &str = "managed Inkscape launch failed; check launcher diagnostics";
pub fn library() -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|_| ERROR)?;
    crate::runtime_layout::library_for_executable(&executable)
}
fn workspace() -> Workspace {
    Workspace {
        roots: vec!["/".into()],
        max_input: 8192,
        max_output: 1024 * 1024,
    }
}
fn secure_directory(path: &Path) -> Result<PathBuf, String> {
    let mut absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir().map_err(|_| ERROR)?.join(path)
    };
    // Normalize only the known macOS system aliases, never arbitrary user links.
    if cfg!(target_os = "macos") {
        for (alias, target) in [("/tmp", "/private/tmp"), ("/var", "/private/var")] {
            if let Ok(tail) = absolute.strip_prefix(alias)
                && std::fs::read_link(alias).is_ok_and(|link| link == Path::new(target))
            {
                absolute = Path::new(target).join(tail);
            }
        }
    }
    let relative = absolute.strip_prefix("/").map_err(|_| ERROR)?;
    let workspace = workspace();
    workspace.ensure_directory(0, relative).map_err(|_| ERROR)?;
    let handle = workspace.directory_file(0, relative).map_err(|_| ERROR)?;
    let meta = handle.metadata().map_err(|_| ERROR)?;
    if meta.uid() != unsafe { libc::getuid() }
        || meta.mode() & 0o7777 != 0o700
        || absolute
            .join("bus.sock")
            .as_os_str()
            .as_encoded_bytes()
            .len()
            >= 104
    {
        return Err(ERROR.into());
    }
    if !absolute.to_str().is_some_and(|s| {
        s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"/._-".contains(&c))
    }) {
        return Err(ERROR.into());
    }
    Ok(absolute)
}
struct LaunchLock(File);
impl Drop for LaunchLock {
    fn drop(&mut self) {
        // Close alone can retain a flock while another thread's forked child holds
        // the inherited open-file description. This lock belongs to this scope.
        unsafe {
            libc::flock(self.0.as_raw_fd(), libc::LOCK_UN);
        }
    }
}
fn lock(path: &Path, deadline: Instant) -> Result<LaunchLock, String> {
    let file = workspace()
        .lock_file(0, path.strip_prefix("/").map_err(|_| ERROR)?)
        .map_err(|_| ERROR)?;
    loop {
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
            return Ok(LaunchLock(file));
        }
        if std::io::Error::last_os_error().raw_os_error() != Some(libc::EWOULDBLOCK)
            || Instant::now() >= deadline
        {
            return Err(ERROR.into());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}
fn probe(root: &Path, cap: usize) -> Result<Attached, String> {
    let mut input = Inputs::environment(process::timeout(), cap);
    input.directory = Some(root.to_owned());
    live_attach::refresh(input).map_err(|_| ERROR.into())
}
pub fn launch(cap: usize) -> Result<Attached, String> {
    if !cfg!(target_os = "macos") {
        return Err("live_launch requires macOS".into());
    }
    let configured = std::env::var_os("INKSCAPE_MCP_MANAGED_DIR").map(PathBuf::from);
    let path = configured.unwrap_or_else(|| {
        Path::new("/tmp")
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from("/private/tmp"))
            .join(format!("inkscape-mcp-{}", unsafe { libc::getuid() }))
    });
    let root = secure_directory(&path)?;
    let deadline = Instant::now() + Duration::from_secs(15);
    let _launch_lock = lock(&root.join("launch.lock"), deadline)?;
    let attached = probe(&root, cap)?;
    if attached.inputs.address.is_some() {
        return Ok(attached);
    }
    // Do not start another GUI when a surviving supervisor holds its lock but bus is lost.
    let supervisor = lock(&root.join("supervisor.lock"), Instant::now())?;
    drop(supervisor);
    let library = library()?;
    let supervisor = crate::runtime_layout::binary(&library, "inkscape-mcp-supervisor");
    for path in [
        &supervisor,
        &crate::runtime_layout::asset(&library, "context.so"),
        &crate::runtime_layout::asset(&library, "dbus/bin/dbus-daemon"),
    ] {
        workspace()
            .regular_file(0, path.strip_prefix("/").map_err(|_| ERROR)?)
            .map_err(|_| ERROR)?;
    }
    let binary = process::inkscape_binary().ok_or(ERROR)?;
    let log = root.join("supervisor.log");
    workspace()
        .atomic_write(0, log.strip_prefix("/").map_err(|_| ERROR)?, b"")
        .map_err(|_| ERROR)?;
    let output = workspace()
        .lock_file(0, log.strip_prefix("/").map_err(|_| ERROR)?)
        .map_err(|_| ERROR)?;
    let mut command = Command::new(supervisor);
    command
        .arg(&root)
        .arg(binary)
        .stdin(Stdio::null())
        .stdout(output.try_clone().map_err(|_| ERROR)?)
        .stderr(output);
    // SAFETY: only async-signal-safe setsid runs in the newly forked owned child.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
    let mut child = command.spawn().map_err(|_| ERROR)?;
    loop {
        if let Ok(attached) = probe(&root, cap)
            && attached.inputs.address.is_some()
        {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            return Ok(attached);
        }
        if child.try_wait().map_err(|_| ERROR)?.is_some() {
            return Err(ERROR.into());
        }
        if Instant::now() >= deadline {
            // A slow supervisor may already own an unsaved drawing. Never terminate it.
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            return Err(ERROR.into());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};
    #[test]
    fn scope_release_unlocks_even_while_a_forked_child_retains_the_descriptor() {
        let fixture = tempfile::tempdir().unwrap();
        let path = fixture.path().canonicalize().unwrap().join("launch.lock");
        let first = lock(&path, Instant::now()).unwrap();
        let mut pipe = [0; 2];
        assert_eq!(unsafe { libc::pipe(pipe.as_mut_ptr()) }, 0);
        // The child uses only async-signal-safe libc calls and never executes an app.
        let child = unsafe { libc::fork() };
        if child == 0 {
            unsafe {
                libc::close(pipe[1]);
                let mut byte = 0u8;
                libc::read(pipe[0], (&mut byte as *mut u8).cast(), 1);
                libc::_exit(0);
            }
        }
        unsafe {
            libc::close(pipe[0]);
        }
        if child < 0 {
            unsafe {
                libc::close(pipe[1]);
            }
            panic!("owned fixture fork failed");
        }
        drop(first);
        let acquired = lock(&path, Instant::now());
        unsafe {
            let byte = 1u8;
            libc::write(pipe[1], (&byte as *const u8).cast(), 1);
            libc::close(pipe[1]);
            libc::waitpid(child, std::ptr::null_mut(), 0);
        }
        assert!(
            acquired.is_ok(),
            "inherited descriptor retained the released scope lock"
        );
    }
    #[test]
    fn private_session_and_locks_refuse_link_escape_and_parallel_launch() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().canonicalize().unwrap();
        let session = root.join("session");
        secure_directory(&session).unwrap();
        std::fs::set_permissions(&session, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(secure_directory(&session).is_err());
        std::fs::set_permissions(&session, std::fs::Permissions::from_mode(0o700)).unwrap();
        symlink(&session, root.join("alias")).unwrap();
        assert!(secure_directory(&root.join("alias/new")).is_err());
        assert!(!session.join("new").exists());
        let original = root.join("original");
        std::fs::write(&original, b"external original").unwrap();
        symlink(&original, session.join("launch.lock")).unwrap();
        assert!(lock(&session.join("launch.lock"), Instant::now()).is_err());
        assert_eq!(std::fs::read(&original).unwrap(), b"external original");
        std::fs::remove_file(session.join("launch.lock")).unwrap();
        let first = lock(&session.join("launch.lock"), Instant::now()).unwrap();
        assert!(lock(&session.join("launch.lock"), Instant::now()).is_err());
        drop(first);
        lock(&session.join("launch.lock"), Instant::now()).unwrap();
    }
}
