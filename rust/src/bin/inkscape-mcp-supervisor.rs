//! Fixed managed GUI supervisor. No Python interpreter, MCP server or runtime compiler.
#[allow(dead_code)]
#[path = "../workspace.rs"]
mod workspace;
use serde_json::json;
use std::{
    ffi::CString,
    fs::{self, File},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{ffi::OsStrExt, fs::MetadataExt},
    },
    path::{Component, Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const LIMIT: usize = 128 * 1024 * 1024;
const ASSETS: [&str; 6] = [
    "inkscape_mcp_insert.py",
    "inkscape_mcp_insert.inx",
    "inkscape_mcp_edit.py",
    "inkscape_mcp_edit.inx",
    "inkscape_mcp_insert_payload.py",
    "inkscape_mcp_edit_errors.py",
];
fn workspace() -> workspace::Workspace {
    workspace::Workspace {
        roots: vec!["/".into()],
        max_input: LIMIT,
        max_output: LIMIT,
    }
}
fn relative(path: &Path) -> Result<&Path> {
    if path
        .components()
        .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err("unsafe directory".into());
    }
    Ok(path.strip_prefix("/")?)
}
fn read(path: &Path, cap: usize) -> Result<Vec<u8>> {
    Ok(workspace().read(0, relative(path)?, cap)?)
}
fn c_name(name: &str) -> Result<CString> {
    if name.is_empty() || name.contains('/') || name == "." || name == ".." {
        return Err("invalid output name".into());
    }
    Ok(CString::new(name)?)
}
struct Dir(File);
impl Dir {
    fn open(path: &Path, create: bool) -> Result<Self> {
        let rel = relative(path)?;
        if create {
            workspace().ensure_directory(0, rel)?;
        }
        Ok(Self(workspace().directory_file(0, rel)?))
    }
    fn kind(&self, name: &str) -> Result<Option<libc::mode_t>> {
        let name = c_name(name)?;
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: pinned directory, valid name and writable stat storage; no link following.
        if unsafe {
            libc::fstatat(
                self.0.as_raw_fd(),
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } == 0
        {
            Ok(Some(unsafe { stat.assume_init() }.st_mode & libc::S_IFMT))
        } else {
            let e = std::io::Error::last_os_error();
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(e.into())
            }
        }
    }
    fn regular_destination(&self, name: &str) -> Result<()> {
        if self.kind(name)?.is_some_and(|kind| kind != libc::S_IFREG) {
            return Err("nonregular output".into());
        }
        Ok(())
    }
    fn file(&self, name: &str) -> Result<File> {
        let name = c_name(name)?;
        // SAFETY: descriptor anchored open, fixed mode, validated single filename.
        let fd = unsafe {
            libc::openat(
                self.0.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDWR
                    | libc::O_CREAT
                    | libc::O_NOFOLLOW
                    | libc::O_NONBLOCK
                    | libc::O_CLOEXEC,
                0o600,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let file = unsafe { File::from_raw_fd(fd) };
        if !file.metadata()?.is_file() {
            return Err("invalid lock/log".into());
        }
        Ok(file)
    }
    fn remove(&self, name: &str) -> Result<()> {
        let name = c_name(name)?;
        // SAFETY: unlinks only the named directory entry, never a link target.
        if unsafe { libc::unlinkat(self.0.as_raw_fd(), name.as_ptr(), 0) } == 0 {
            return Ok(());
        }
        let error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::NotFound {
            Ok(())
        } else {
            Err(error.into())
        }
    }
    fn write(&self, name: &str, raw: &[u8], mode: libc::mode_t) -> Result<()> {
        self.regular_destination(name)?;
        let dest = c_name(name)?;
        let temporary = format!(".mcp-{}", uuid::Uuid::new_v4().simple());
        let token = c_name(&temporary)?;
        // SAFETY: fresh exclusive no-follow entry in the pinned directory.
        let fd = unsafe {
            libc::openat(
                self.0.as_raw_fd(),
                token.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                mode as libc::c_uint,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let result = (|| {
            let mut file = unsafe { File::from_raw_fd(fd) };
            file.write_all(raw)?;
            file.sync_all()?;
            self.regular_destination(name)?;
            // SAFETY: atomic publication between single names on the same directory descriptor.
            if unsafe {
                libc::renameat(
                    self.0.as_raw_fd(),
                    token.as_ptr(),
                    self.0.as_raw_fd(),
                    dest.as_ptr(),
                )
            } != 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
            self.0.sync_all()?;
            Ok(())
        })();
        let _ = self.remove(&temporary);
        result
    }
    fn resource_link(&self, target: &Path) -> Result<()> {
        let name = c_name("Resources")?;
        if self.kind("Resources")?.is_some() {
            let mut bytes = vec![0u8; 8192];
            // SAFETY: pinned descriptor/name and valid bounded output buffer.
            let length = unsafe {
                libc::readlinkat(
                    self.0.as_raw_fd(),
                    name.as_ptr(),
                    bytes.as_mut_ptr().cast(),
                    bytes.len(),
                )
            };
            if length < 0 || &bytes[..length as usize] != target.as_os_str().as_bytes() {
                return Err("unexpected resource link".into());
            }
        } else {
            let target = CString::new(target.as_os_str().as_bytes())?;
            // SAFETY: creates only the fixed Resources symlink to the validated vendor resources.
            if unsafe { libc::symlinkat(target.as_ptr(), self.0.as_raw_fd(), name.as_ptr()) } != 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            self.0.sync_all()?;
        }
        Ok(())
    }
}
fn lock(dir: &Dir) -> Result<File> {
    let file = dir.file("supervisor.lock")?;
    // SAFETY: live regular descriptor; releases when File drops.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err("supervisor lock occupied".into());
    }
    Ok(file)
}
fn wait(child: &mut Child, timeout: Duration) -> Result<bool> {
    let deadline = Instant::now() + timeout;
    loop {
        if child.try_wait()?.is_some() {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(20));
    }
}
fn stop(child: &mut Child) {
    if child.try_wait().ok().flatten().is_some() {
        return;
    }
    // SAFETY: PID belongs to this unreaped Child only. Never signal by process name/group.
    unsafe {
        libc::kill(child.id() as libc::pid_t, libc::SIGTERM);
    }
    if !matches!(wait(child, Duration::from_secs(5)), Ok(true)) {
        let _ = child.kill();
    }
    let _ = child.wait();
}
struct OwnedBus(Child);
impl Drop for OwnedBus {
    fn drop(&mut self) {
        stop(&mut self.0);
    }
}
fn captured(command: &mut Command, timeout: Duration, cap: usize) -> Result<Vec<u8>> {
    let mut output = tempfile::tempfile()?;
    let errors = tempfile::tempfile()?;
    let mut child = command
        .stdin(Stdio::null())
        .stdout(output.try_clone()?)
        .stderr(errors.try_clone()?)
        .spawn()?;
    let result = (|| {
        let deadline = Instant::now() + timeout;
        loop {
            if output.metadata()?.len() > cap as u64 || errors.metadata()?.len() > cap as u64 {
                return Err("fixed command output exceeded limit".into());
            }
            if let Some(status) = child.try_wait()? {
                if !status.success() {
                    return Err("fixed preparation command failed".into());
                }
                break;
            }
            if Instant::now() >= deadline {
                return Err("fixed preparation command timed out".into());
            }
            thread::sleep(Duration::from_millis(20));
        }
        use std::io::{Seek, SeekFrom};
        output.seek(SeekFrom::Start(0))?;
        let mut bytes = vec![];
        output.take((cap + 1) as u64).read_to_end(&mut bytes)?;
        if bytes.len() > cap {
            return Err("fixed command output exceeded limit".into());
        }
        Ok(bytes)
    })();
    if result.is_err() {
        stop(&mut child);
    }
    result
}
fn quote(path: &Path) -> Result<String> {
    Ok(format!(
        "'{}'",
        path.to_str()
            .ok_or("invalid helper path")?
            .replace('\'', "'\"'\"'")
    ))
}
#[derive(Debug)]
struct Prepared {
    executable: PathBuf,
    bridge: PathBuf,
}
fn prepare(library: &Path, root: &Path, binary: &Path) -> Result<Prepared> {
    prepare_with(library, root, binary, |executable| {
        captured(
            Command::new("/usr/bin/codesign")
                .args(["--force", "--sign", "-"])
                .arg(executable),
            Duration::from_secs(30),
            1024 * 1024,
        )?;
        Ok(())
    })
}
fn prepare_with(
    library: &Path,
    root: &Path,
    binary: &Path,
    sign: impl FnOnce(&Path) -> Result<()>,
) -> Result<Prepared> {
    let contents = binary
        .parent()
        .and_then(Path::parent)
        .ok_or("invalid vendor bundle")?;
    let resources = contents.join("Resources");
    if binary
        .parent()
        .and_then(Path::file_name)
        .is_none_or(|n| n != "MacOS")
    {
        return Err("official GTK 3 Inkscape bundle required".into());
    }
    read(&resources.join("lib/libgtk-3.0.dylib"), LIMIT)?;
    let vendor = resources.join("share/inkscape/extensions");
    Dir::open(&vendor.join("inkex"), false)?;
    // Read all bounded assets before making any persistent changes.
    let payloads = ASSETS
        .iter()
        .map(|name| {
            Ok((
                *name,
                read(&library.join("helpers").join(name), 2 * 1024 * 1024)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let executable_bytes = read(binary, LIMIT)?;
    let bridge_bytes = read(&library.join("context.so"), 16 * 1024 * 1024)?;
    let mut info = plist::Value::from_reader(std::io::Cursor::new(read(
        &contents.join("Info.plist"),
        1024 * 1024,
    )?))?;
    info.as_dictionary_mut()
        .ok_or("invalid vendor plist")?
        .insert(
            "CFBundleIdentifier".into(),
            plist::Value::String("org.inkscape.Inkscape.MCPManaged".into()),
        );
    let mut plist_bytes = vec![];
    info.to_writer_xml(&mut plist_bytes)?;
    let bytes = captured(
        Command::new(binary).arg("--user-data-directory"),
        Duration::from_secs(10),
        8192,
    )?;
    let user = PathBuf::from(std::str::from_utf8(&bytes)?.trim());
    relative(&user)?;
    let target = user.join("extensions");
    let extensions = Dir::open(&target, true)?;
    for (name, _) in &payloads {
        extensions.regular_destination(name)?;
    }
    extensions.regular_destination("inkscape_mcp_insert_run.sh")?;
    let macos = root.join("context-bridge/Inkscape.app/Contents/MacOS");
    let private = Dir::open(&macos, true)?;
    let contents_dir = Dir::open(macos.parent().ok_or("invalid private bundle")?, false)?;
    contents_dir.resource_link(&resources)?;
    let bridge_dir = Dir::open(&root.join("context-bridge"), false)?;
    for (name, raw) in payloads {
        extensions.write(name, &raw, 0o600)?;
    }
    let wrapper = format!(
        "#!/bin/sh\nunset PYTHONHOME PYTHONPATH\nexport PYTHONPATH={}\nexec {} {} \"$@\"\n",
        quote(&vendor)?,
        quote(&library.join("python/bin/python3"))?,
        quote(&target.join("inkscape_mcp_insert.py"))?
    );
    extensions.write("inkscape_mcp_insert_run.sh", wrapper.as_bytes(), 0o700)?;
    private.write("inkscape", &executable_bytes, 0o700)?;
    contents_dir.write("Info.plist", &plist_bytes, 0o600)?;
    bridge_dir.write("context.so", &bridge_bytes, 0o600)?;
    let executable = macos.join("inkscape");
    sign(&executable)?;
    Ok(Prepared {
        executable,
        bridge: root.join("context-bridge/context.so"),
    })
}
fn supervise_with(
    root: &Path,
    library: &Path,
    preparation: impl FnOnce() -> Result<Prepared>,
    ready_timeout: Duration,
) -> Result<()> {
    let dir = Dir::open(root, false)?;
    let info = dir.0.metadata()?;
    if info.uid() != unsafe { libc::getuid() } || info.mode() & 0o7777 != 0o700 {
        return Err("private session permissions required".into());
    }
    if !root.to_str().is_some_and(|s| {
        s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
    }) || root.join("bus.sock").as_os_str().as_bytes().len() >= 104
    {
        return Err("unsafe private bus path".into());
    }
    let _held = lock(&dir)?; // Failed/concurrent launch must never touch an existing manifest.
    let mut cleanup_allowed = true;
    let result = (|| {
        dir.regular_destination("session.json")?;
        let prepared = preparation()?;
        if dir
            .kind("bus.sock")?
            .is_some_and(|kind| kind != libc::S_IFSOCK)
        {
            return Err("invalid stale bus socket".into());
        }
        dir.remove("bus.sock")?;
        for name in ["bus.log", "inkscape.stdout.log", "inkscape.stderr.log"] {
            dir.regular_destination(name)?;
        }
        for name in ["bus.log", "inkscape.stdout.log", "inkscape.stderr.log"] {
            dir.write(name, b"", 0o600)?;
        }
        workspace().regular_file(0, relative(&library.join("dbus/bin/dbus-daemon"))?)?;
        read(&library.join("dbus/session.conf"), 1024 * 1024)?;
        let log = dir.file("bus.log")?;
        let address = format!("unix:path={}", root.join("bus.sock").display());
        let mut bus = OwnedBus(
            Command::new(library.join("dbus/bin/dbus-daemon"))
                .arg(format!(
                    "--config-file={}",
                    library.join("dbus/session.conf").display()
                ))
                .arg(format!("--address={address}"))
                .args(["--nofork", "--print-address=1"])
                .stdin(Stdio::null())
                .stdout(log.try_clone()?)
                .stderr(log)
                .spawn()?,
        );
        let deadline = Instant::now() + ready_timeout;
        loop {
            if bus.0.try_wait()?.is_some() {
                return Err("private bus exited".into());
            }
            if dir.kind("bus.sock")? == Some(libc::S_IFSOCK) {
                break;
            }
            if Instant::now() >= deadline {
                return Err("private bus did not become ready".into());
            }
            thread::sleep(Duration::from_millis(50));
        }
        let stdout = dir.file("inkscape.stdout.log")?;
        let stderr = dir.file("inkscape.stderr.log")?;
        let modules = std::env::var("GTK_MODULES").unwrap_or_default();
        let modules = if modules.is_empty() {
            prepared.bridge.display().to_string()
        } else {
            format!("{modules}:{}", prepared.bridge.display())
        };
        let mut gui = Command::new(&prepared.executable)
            .arg("--with-gui")
            .env("DBUS_SESSION_BUS_ADDRESS", &address)
            .env("INKSCAPE_MCP_MANAGED_DIR", root)
            .env("INKSCAPE_MCP_CONTEXT_BRIDGE", "1")
            .env("GTK_MODULES", modules)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .spawn()?;
        cleanup_allowed = false;
        let manifest = json!({"address":address,"inkscape_pid":gui.id(),"supervisor_pid":std::process::id(),"context_bridge":true});
        let publication = dir.write("session.json", &serde_json::to_vec(&manifest)?, 0o600);
        // Even if publication fails after spawn, retain ownership until this GUI exits.
        // Never kill a possibly unsaved drawing or drop its D-Bus while it is alive.
        let exited = gui.wait();
        if exited.is_err() {
            std::mem::forget(bus);
            return Err("GUI wait failed; bus retained for surviving drawing".into());
        }
        cleanup_allowed = true;
        publication?;
        Ok(()) // OwnedBus drop terminates/reaps only our bus, after GUI exit.
    })();
    let cleanup = if cleanup_allowed {
        dir.remove("session.json")
    } else {
        Ok(())
    };
    result.and(cleanup)
}
fn main_result() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !cfg!(target_os = "macos") || args.len() != 2 {
        return Err("fixed managed supervisor requires macOS and owned session inputs".into());
    }
    let root = PathBuf::from(&args[0]);
    let binary = fs::canonicalize(&args[1])?;
    let exe = std::env::current_exe()?;
    let library = exe
        .parent()
        .and_then(Path::parent)
        .ok_or("invalid supervisor installation")?
        .join("libexec/inkscape-mcp");
    supervise_with(
        &root,
        &library,
        || prepare(&library, &root, &binary),
        Duration::from_secs(5),
    )
}
fn main() {
    if let Err(error) = main_result() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        os::unix::fs::{PermissionsExt, symlink},
        sync::OnceLock,
    };
    fn root() -> tempfile::TempDir {
        tempfile::tempdir_in(Path::new("/tmp").canonicalize().unwrap()).unwrap()
    }
    fn fixture_binary() -> &'static Path {
        static FIXTURE: OnceLock<PathBuf> = OnceLock::new();
        FIXTURE.get_or_init(|| {
            let root = root();
            let source = root.path().join("fixture.rs");
            fs::write(
                &source,
                include_str!("../../tests/fixtures/supervisor-process.rs"),
            )
            .unwrap();
            let binary = root.path().join("fixture");
            let rustc = std::env::var_os("RUSTC")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cargo/bin/rustc")
                });
            assert!(
                Command::new(rustc)
                    .arg(&source)
                    .args(["--edition=2024", "-o"])
                    .arg(&binary)
                    .status()
                    .unwrap()
                    .success()
            );
            let path = binary.clone();
            // Fixture persists only for this test process; no production runtime compilation.
            let _ = root.keep();
            path
        })
    }
    fn library(root: &Path, mode: &str) -> PathBuf {
        let library = root.join("library");
        fs::create_dir_all(library.join("dbus/bin")).unwrap();
        fs::copy(fixture_binary(), library.join("dbus/bin/dbus-daemon")).unwrap();
        fs::write(library.join("dbus/bin/dbus-daemon.mode"), mode).unwrap();
        fs::write(library.join("dbus/session.conf"), "fixture config").unwrap();
        library
    }
    fn session(root: &Path) -> PathBuf {
        let path = root.join("session");
        fs::create_dir(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }
    fn prepared(root: &Path, mode: &str) -> Prepared {
        let executable = root.join("gui");
        fs::copy(fixture_binary(), &executable).unwrap();
        fs::write(executable.with_extension("mode"), mode).unwrap();
        Prepared {
            executable,
            bridge: root.join("context.so"),
        }
    }
    fn until(mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !condition() {
            assert!(Instant::now() < deadline, "fixture deadline");
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn assert_bus_gone(session: &Path) {
        let pid: i32 = fs::read_to_string(session.join("bus.pid"))
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "owned bus still running");
        assert!(!session.join("session.json").exists());
    }
    #[test]
    fn permissions_links_and_atomic_outputs() {
        let root = root();
        let path = session(root.path());
        let dir = Dir::open(&path, false).unwrap();
        let original = root.path().join("original");
        fs::write(&original, b"original").unwrap();
        fs::hard_link(&original, path.join("asset")).unwrap();
        dir.write("asset", b"updated", 0o600).unwrap();
        assert_eq!(fs::read(&original).unwrap(), b"original");
        fs::remove_file(path.join("asset")).unwrap();
        symlink(&original, path.join("asset")).unwrap();
        assert!(dir.write("asset", b"bad", 0o600).is_err());
        symlink(&path, root.path().join("alias")).unwrap();
        assert!(Dir::open(&root.path().join("alias/new"), true).is_err());
        assert!(!path.join("new").exists());
        dir.write("executable", b"fixed", 0o700).unwrap();
        assert_eq!(
            fs::metadata(path.join("executable")).unwrap().mode() & 0o777,
            0o700
        );
        assert!(dir.write("../outside", b"bad", 0o600).is_err());
        assert!(!fs::read_dir(&path).unwrap().any(|p| {
            p.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".mcp-")
        }));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            supervise_with(
                &path,
                root.path(),
                || panic!("must refuse before preparation"),
                Duration::ZERO
            )
            .is_err()
        );
    }
    #[test]
    fn competing_launch_retains_live_manifest_and_owned_bus() {
        let root = root();
        let path = session(root.path());
        let lib = library(root.path(), "");
        let gui = prepared(root.path(), "hold");
        let (s, l) = (path.clone(), lib.clone());
        let supervisor =
            thread::spawn(move || supervise_with(&s, &l, || Ok(gui), Duration::from_secs(2)));
        until(|| path.join("session.json").exists());
        let bytes = fs::read(path.join("session.json")).unwrap();
        let manifest: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            manifest["address"],
            format!("unix:path={}", path.join("bus.sock").display())
        );
        assert_eq!(manifest["context_bridge"], true);
        assert_eq!(manifest["supervisor_pid"], std::process::id());
        assert_eq!(
            fs::metadata(path.join("session.json")).unwrap().mode() & 0o777,
            0o600
        );
        let error = supervise_with(
            &path,
            &lib,
            || panic!("second launch must not prepare"),
            Duration::ZERO,
        )
        .unwrap_err();
        assert!(error.to_string().contains("lock occupied"));
        assert_eq!(fs::read(path.join("session.json")).unwrap(), bytes);
        fs::write(path.join("finish-gui"), b"").unwrap();
        supervisor.join().unwrap().unwrap();
        assert_bus_gone(&path);
    }
    #[test]
    fn bus_timeout_exit_and_gui_spawn_failure_clean_only_owned_processes() {
        for mode in ["timeout", "fail", "missing-gui"] {
            let root = root();
            let path = session(root.path());
            let lib = library(root.path(), if mode == "missing-gui" { "" } else { mode });
            let mut gui = prepared(root.path(), "exit");
            if mode == "missing-gui" {
                gui.executable = root.path().join("absent");
            }
            let result = supervise_with(&path, &lib, || Ok(gui), Duration::from_secs(1));
            assert!(result.is_err());
            assert!(!path.join("session.json").exists());
            if mode != "fail" {
                assert_bus_gone(&path);
            }
        }
    }
    #[test]
    fn preparation_error_and_partial_session_can_recover() {
        let root = root();
        let path = session(root.path());
        let lib = library(root.path(), "");
        fs::write(path.join("session.json"), b"stale manifest").unwrap();
        assert!(
            supervise_with(
                &path,
                &lib,
                || Err("injected preparation failure".into()),
                Duration::ZERO
            )
            .is_err()
        );
        assert!(!path.join("session.json").exists());
        let gui = prepared(root.path(), "exit");
        supervise_with(&path, &lib, || Ok(gui), Duration::from_secs(2)).unwrap();
        assert_bus_gone(&path);
    }
    #[test]
    fn private_bundle_preparation_plist_helpers_and_failures() {
        let root = root();
        let lib = library(root.path(), "");
        fs::create_dir_all(lib.join("helpers")).unwrap();
        for name in ASSETS {
            fs::write(lib.join("helpers").join(name), name).unwrap();
        }
        fs::write(lib.join("context.so"), b"context").unwrap();
        let contents = root.path().join("Vendor.app/Contents");
        fs::create_dir_all(contents.join("MacOS")).unwrap();
        fs::create_dir_all(contents.join("Resources/share/inkscape/extensions/inkex")).unwrap();
        fs::create_dir_all(contents.join("Resources/lib")).unwrap();
        fs::write(contents.join("Resources/lib/libgtk-3.0.dylib"), b"gtk").unwrap();
        fs::write(contents.join("Info.plist"),b"<?xml version=\"1.0\"?><plist version=\"1.0\"><dict><key>CFBundleIdentifier</key><string>vendor</string><key>Retained</key><true/></dict></plist>").unwrap();
        let binary = contents.join("MacOS/inkscape");
        fs::copy(fixture_binary(), &binary).unwrap();
        let original = fs::read(&binary).unwrap();
        let original_plist = fs::read(contents.join("Info.plist")).unwrap();
        let path = session(root.path());
        let prepared = prepare_with(&lib, &path, &binary, |private| {
            assert_ne!(private, &binary);
            Ok(())
        })
        .unwrap();
        assert_eq!(fs::read(&binary).unwrap(), original);
        assert_eq!(
            fs::read(contents.join("Info.plist")).unwrap(),
            original_plist
        );
        let info = plist::Value::from_file(
            prepared
                .executable
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("Info.plist"),
        )
        .unwrap();
        assert_eq!(
            info.as_dictionary().unwrap()["CFBundleIdentifier"].as_string(),
            Some("org.inkscape.Inkscape.MCPManaged")
        );
        assert_eq!(
            info.as_dictionary().unwrap()["Retained"].as_boolean(),
            Some(true)
        );
        let extension = contents.parent().unwrap().join("profile/extensions");
        assert!(
            fs::read_to_string(extension.join("inkscape_mcp_insert_run.sh"))
                .unwrap()
                .contains("python/bin/python3")
        );
        assert!(
            prepare_with(&lib, &path, &binary, |_| Err("codesign failure".into()))
                .unwrap_err()
                .to_string()
                .contains("codesign failure")
        );
        fs::remove_file(lib.join("helpers").join(ASSETS[0])).unwrap();
        let before = fs::read(extension.join(ASSETS[0])).unwrap();
        assert!(
            prepare_with(&lib, &path, &binary, |_| panic!(
                "missing asset must fail before sign"
            ))
            .is_err()
        );
        assert_eq!(fs::read(extension.join(ASSETS[0])).unwrap(), before);
    }
}
