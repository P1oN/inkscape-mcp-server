use super::{
    instructions::{guarded, read},
    manifests::*,
};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

pub fn json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    serde_json::from_slice(&read(path, 2 * 1024 * 1024)?).map_err(|e| e.to_string())
}
pub fn sync_dir(path: &Path) -> Result<()> {
    fs::File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}
pub fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("missing output parent")?;
    guarded(parent)?;
    if path.exists() || path.is_symlink() {
        guarded(path)?;
    }
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temp.write_all(bytes)
        .and_then(|_| temp.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    temp.persist(path).map_err(|e| e.to_string())?;
    sync_dir(parent)
}
pub fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    write(
        path,
        &serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
}
pub fn mkdir(path: &Path) -> Result<()> {
    if path.exists() || path.is_symlink() {
        guarded(path)?;
        if !path.is_dir() {
            return Err("managed directory is not a directory".into());
        }
        return Ok(());
    }
    let parent = path.parent().ok_or("missing directory parent")?;
    if !parent.exists() {
        mkdir(parent)?;
    }
    guarded(parent)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new()
            .mode(0o700)
            .create(path)
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(unix))]
    fs::create_dir(path).map_err(|e| e.to_string())?;
    sync_dir(parent)
}
pub struct Lock(fs::File);
impl Lock {
    pub fn acquire(root: &Path) -> Result<Self> {
        Self::acquire_wait(root, Duration::ZERO)
    }
    pub fn acquire_wait(root: &Path, timeout: Duration) -> Result<Self> {
        guarded(root)?;
        let path = root.join("update.lock");
        if path.exists() || path.is_symlink() {
            guarded(&path)?;
        }
        let mut options = fs::OpenOptions::new();
        options.read(true).write(true).create(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        let file = options.open(&path).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            let deadline = Instant::now() + timeout;
            loop {
                if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
                    break;
                }
                let error = std::io::Error::last_os_error();
                if error.kind() != std::io::ErrorKind::WouldBlock {
                    return Err(error.to_string());
                }
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err("another update/launch recovery is active; retry later".into());
                }
                std::thread::sleep(remaining.min(Duration::from_millis(25)));
            }
        }
        Ok(Self(file))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            unsafe {
                libc::flock(self.0.as_raw_fd(), libc::LOCK_UN);
            }
        }
    }
}
pub fn managed_relative(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 1024
        && !s.contains(['\\', '\n', '\r', '\0'])
        && std::path::Path::new(s)
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)))
}
pub fn inventory(root: &Path, limit: u64) -> Result<BTreeMap<String, FileIdentity>> {
    inventory_with_local(root, limit, false)
}
pub fn inventory_with_local(
    root: &Path,
    limit: u64,
    skip_local: bool,
) -> Result<BTreeMap<String, FileIdentity>> {
    guarded(root)?;
    let mut pending = vec![root.to_path_buf()];
    let mut files = BTreeMap::new();
    let mut bytes = 0;
    while let Some(path) = pending.pop() {
        if skip_local && path == root.join(".inkscape-mcp-local") {
            continue;
        }
        if pending.len() + files.len() > 50000 {
            return Err("managed tree exceeds entry limit".into());
        }
        guarded(&path)?;
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if metadata.is_dir() {
            for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
                pending.push(entry.map_err(|e| e.to_string())?.path());
            }
        } else if metadata.is_file() {
            bytes += metadata.len();
            if bytes > limit {
                return Err("managed tree exceeds byte limit".into());
            }
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .ok_or("non-UTF8 managed path")?
                .to_string();
            if !managed_relative(&name) {
                return Err("unsafe managed file name".into());
            }
            files.insert(name, FileIdentity::of(&read(&path, limit)?));
        } else {
            return Err("special managed file refused".into());
        }
    }
    Ok(files)
}
pub fn copy_tree(source: &Path, target: &Path, limit: u64) -> Result<()> {
    let files = inventory(source, limit)?;
    mkdir(target)?;
    for (name, id) in &files {
        let output = target.join(name);
        mkdir(output.parent().unwrap())?;
        write(&output, &read(&source.join(name), id.bytes)?)?;
        fs::set_permissions(
            &output,
            fs::metadata(source.join(name))
                .map_err(|e| e.to_string())?
                .permissions(),
        )
        .map_err(|e| e.to_string())?;
    }
    if inventory(source, limit)? != files || inventory(target, limit)? != files {
        return Err("managed tree changed during copy".into());
    }
    Ok(())
}
