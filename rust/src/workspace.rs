//! Directory-descriptor anchored POSIX IO. Paths are never reopened after validation.
//! Native Windows reparse/rename protections remain a separate migration stage.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    ffi::CString,
    fs::File,
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd, IntoRawFd},
        unix::ffi::OsStrExt,
    },
    path::{Component, Path, PathBuf},
};

const ESCAPE: &str = "path rejected: outside workspace; call get_workspace_info and choose a relative path under a configured server root (relative paths default to the first root)";

#[derive(Clone)]
pub struct Workspace {
    pub roots: Vec<PathBuf>,
    pub max_input: usize,
    pub max_output: usize,
}

fn limit(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|value| *value >= 1)
        .unwrap_or(default)
}

impl Workspace {
    pub fn hash_file(&self, index: usize, relative: &Path) -> Result<(u64, String), String> {
        let (parent, name) = self.parent(index, relative, false)?;
        let mut handle = open_at(
            &parent,
            &name,
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            0,
        )?;
        let before = handle
            .metadata()
            .map_err(|_| "artifact could not be stat'd")?;
        if !before.is_file() {
            return Err("path rejected: not a regular file".into());
        }
        if before.len() > self.max_input as u64 {
            return Err("artifact exceeds the configured size limit".into());
        }
        let mut digest = Sha256::new();
        let mut chunk = vec![0u8; 1024 * 1024];
        let mut count = 0u64;
        loop {
            let bound = (self.max_input as u64)
                .saturating_sub(count)
                .saturating_add(1)
                .min(chunk.len() as u64) as usize;
            let read = handle
                .read(&mut chunk[..bound])
                .map_err(|_| "artifact could not be stat'd")?;
            if read == 0 {
                break;
            }
            count += read as u64;
            if count > self.max_input as u64 {
                return Err("artifact exceeds the configured size limit".into());
            }
            digest.update(&chunk[..read]);
        }
        let after = handle
            .metadata()
            .map_err(|_| "artifact could not be stat'd")?;
        if before.len() != count
            || after.len() != count
            || before.modified().ok() != after.modified().ok()
        {
            return Err("artifact changed while being read".into());
        }
        Ok((count, format!("{:x}", digest.finalize())))
    }
    pub fn artifact_link(&self, index: usize, relative: &Path) -> Value {
        let root = Self::root_id(&self.roots[index]);
        let path = relative.to_string_lossy();
        json!({"root_id":root,"uri":format!("inkscape://artifact/{root}/{}",URL_SAFE_NO_PAD.encode(path.as_bytes())),"workspace_relative_path":path,"location":"server"})
    }

    /// Canonicalize the longest existing parent before any mkdir, then create
    /// the missing tail through pinned no-follow descriptors. Final links are
    /// inspected separately; the returned destination never follows one on write.
    pub fn resolve_output(
        &self,
        raw: &str,
        root_id: Option<&str>,
    ) -> Result<(usize, PathBuf), String> {
        if self.roots.is_empty() {
            return Err("path rejected: no workspace root configured".into());
        }
        let path = Path::new(raw);
        let anchored = if let Some(id) = root_id {
            let root = self
                .roots
                .iter()
                .find(|r| Self::root_id(r) == id)
                .ok_or("workspace root ID not found; call get_workspace_info")?;
            if path.is_absolute()
                || path.components().any(|c| c == Component::ParentDir)
                || raw.trim().is_empty()
                || raw.contains('\0')
            {
                return Err("root-qualified path must be a relative path without '..'".into());
            }
            root.join(path)
        } else {
            if raw.is_empty() {
                // Reference anchoring turns Path("") into the root itself; its
                // destination parent is outside that root and is rejected.
                return Err(ESCAPE.into());
            }
            if raw.contains('\0') {
                return Err("path rejected: invalid characters".into());
            }
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.roots[0].join(path)
            }
        };
        if anchored.components().any(|c| c == Component::ParentDir) {
            return Err(ESCAPE.into());
        }
        let name = anchored
            .file_name()
            .ok_or("path rejected: invalid filename")?;
        let parent = anchored.parent().ok_or("path rejected: invalid filename")?;
        let mut existing = parent;
        while !existing.exists() {
            existing = existing
                .parent()
                .ok_or("path rejected: could not resolve path")?;
        }
        let real = existing
            .canonicalize()
            .map_err(|_| "path rejected: could not resolve path")?;
        let root = self
            .roots
            .iter()
            .position(|r| real.starts_with(r))
            .ok_or(ESCAPE)?;
        let resolved = real
            .join(parent.strip_prefix(existing).map_err(|_| ESCAPE)?)
            .join(name);
        let relative = resolved
            .strip_prefix(&self.roots[root])
            .map_err(|_| ESCAPE)?
            .to_path_buf();
        // Descending creates only the contained, validated missing tail.
        self.parent(root, &relative, true)?;
        if self.file_kind(root, &relative)? == Some(libc::S_IFLNK) {
            let target = resolved
                .canonicalize()
                .or_else(|_| {
                    let link = std::fs::read_link(&resolved)?;
                    let target = if link.is_absolute() {
                        link
                    } else {
                        resolved.parent().unwrap().join(link)
                    };
                    let mut lexical = PathBuf::new();
                    for c in target.components() {
                        if c == Component::ParentDir {
                            lexical.pop();
                        } else if c != Component::CurDir {
                            lexical.push(c);
                        }
                    }
                    Ok::<_, std::io::Error>(lexical)
                })
                .map_err(|_| "path rejected: could not resolve path")?;
            if !self.roots.iter().any(|r| target.starts_with(r)) {
                return Err(ESCAPE.into());
            }
        }
        Ok((root, relative))
    }

    pub fn file_kind(&self, index: usize, relative: &Path) -> Result<Option<libc::mode_t>, String> {
        let (parent, name) = self.parent(index, relative, false)?;
        let mut info = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: pinned descriptor and one validated component; stat is read
        // only after fstatat successfully initialized it, without following links.
        let status = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                name.as_ptr(),
                info.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if status == 0 {
            return Ok(Some(unsafe { info.assume_init() }.st_mode & libc::S_IFMT));
        }
        if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
            Ok(None)
        } else {
            Err("path rejected: could not resolve path".into())
        }
    }

    pub fn ensure_directory(&self, index: usize, relative: &Path) -> Result<(), String> {
        self.parent(index, &relative.join(".directory-probe"), true)
            .map(|_| ())
    }

    /// Bounded enumeration through a pinned directory descriptor; never follow child links.
    pub fn directory_file(&self, index: usize, relative: &Path) -> Result<File, String> {
        self.parent(index, &relative.join(".directory-probe"), false)
            .map(|(directory, _)| directory)
    }
    pub fn directory_entries(
        &self,
        index: usize,
        relative: &Path,
    ) -> Result<Vec<(String, libc::mode_t)>, String> {
        let (parent, _) = self.parent(index, &relative.join(".directory-probe"), false)?;
        let fd = parent.into_raw_fd();
        // SAFETY: fd is an owned open directory descriptor. fdopendir owns it on success.
        let directory = unsafe { libc::fdopendir(fd) };
        if directory.is_null() {
            unsafe {
                libc::close(fd);
            }
            return Err("directory unavailable".into());
        }
        struct Directory(*mut libc::DIR);
        impl Drop for Directory {
            fn drop(&mut self) {
                // SAFETY: this wrapper uniquely owns the successful fdopendir handle.
                unsafe {
                    libc::closedir(self.0);
                }
            }
        }
        let directory = Directory(directory);
        let mut entries = Vec::new();
        let mut count = 0;
        loop {
            // POSIX distinguishes EOF from an enumeration fault through errno. Clear the
            // current thread's errno so partial observations cannot produce a lower counter.
            #[cfg(target_vendor = "apple")]
            let errno = unsafe { libc::__error() };
            #[cfg(not(target_vendor = "apple"))]
            let errno = unsafe { libc::__errno_location() };
            unsafe {
                *errno = 0;
            }
            // SAFETY: live owned DIR; returned entry is read before the next readdir call.
            let child = unsafe { libc::readdir(directory.0) };
            if child.is_null() {
                if unsafe { *errno } != 0 {
                    return Err("directory unavailable".into());
                }
                break;
            }
            count += 1;
            if count > 10002 {
                return Err("directory entry limit exceeded".into());
            }
            let name = unsafe { std::ffi::CStr::from_ptr((*child).d_name.as_ptr()) };
            let Ok(text) = name.to_str() else {
                continue;
            };
            if text == "." || text == ".." {
                continue;
            }
            let mut info = std::mem::MaybeUninit::<libc::stat>::uninit();
            // SAFETY: one NUL-terminated directory name and pinned descriptor. No link following.
            let status = unsafe {
                libc::fstatat(
                    fd,
                    name.as_ptr(),
                    info.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            if status == 0 {
                entries.push((
                    text.to_owned(),
                    unsafe { info.assume_init() }.st_mode & libc::S_IFMT,
                ));
            }
        }
        Ok(entries)
    }

    pub fn read_optional(
        &self,
        index: usize,
        relative: &Path,
        cap: usize,
    ) -> Result<Option<Vec<u8>>, String> {
        let (parent, name) = self.parent(index, relative, false)?;
        // SAFETY: parent is pinned and name is one validated NUL-terminated component.
        let fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
                return Ok(None);
            }
            return Err("path rejected: could not access file safely".into());
        }
        // SAFETY: successful openat transfers one owned descriptor.
        let handle = unsafe { File::from_raw_fd(fd) };
        bounded_read(handle, cap).map(Some)
    }

    pub fn remove_file(&self, index: usize, relative: &Path) -> Result<(), String> {
        let (parent, name) = self.parent(index, relative, false)?;
        // SAFETY: unlink only this single relative name under the pinned directory.
        let status = unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), 0) };
        if status != 0 {
            return Err("could not remove managed file".into());
        }
        Ok(())
    }

    /// Inspect/unlink only a regular final entry under one pinned directory.
    /// Links and directories are refused, including during retention cleanup.
    pub fn regular_info(
        &self,
        index: usize,
        relative: &Path,
        remove: bool,
    ) -> Result<Option<(u64, f64)>, String> {
        let (parent, name) = self.parent(index, relative, false)?;
        let mut info = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: pinned directory and single validated basename; no symlink following.
        if unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                name.as_ptr(),
                info.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
                return Ok(None);
            }
            return Err("could not inspect managed file".into());
        }
        let info = unsafe { info.assume_init() };
        if info.st_mode & libc::S_IFMT != libc::S_IFREG || info.st_size < 0 {
            return Err("managed file is not a regular file".into());
        }
        let mtime = info.st_mtime as f64 + info.st_mtime_nsec as f64 / 1e9;
        if remove && unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), 0) } != 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
                return Ok(None);
            }
            return Err("could not remove managed file".into());
        }
        Ok(Some((info.st_size as u64, mtime)))
    }

    pub fn read_artifact(&self, root_key: &str, token: &str) -> Result<Vec<u8>, String> {
        let rejected = "artifact unavailable or rejected by workspace/size policy";
        let index = self
            .roots
            .iter()
            .position(|root| Self::root_id(root) == root_key)
            .ok_or(rejected)?;
        if token.len() > 8192 {
            return Err(rejected.into());
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(token.trim_end_matches('='))
            .map_err(|_| rejected)?;
        let relative = std::str::from_utf8(&bytes).map_err(|_| rejected)?;
        if Path::new(relative).is_absolute()
            || Path::new(relative)
                .components()
                .any(|c| c == Component::ParentDir)
        {
            return Err(rejected.into());
        }
        let (resolved_index, resolved) = self
            .resolve_input(relative, Some(root_key))
            .map_err(|_| rejected)?;
        if index != resolved_index {
            return Err(rejected.into());
        }
        self.read(index, &resolved, self.max_output)
            .map_err(|_| rejected.into())
    }

    pub fn from_env() -> Self {
        let roots = std::env::var_os("INKSCAPE_MCP_WORKSPACE_ROOTS")
            .map(|value| {
                std::env::split_paths(&value)
                    .filter_map(|p| p.canonicalize().ok())
                    .filter(|p| p.is_dir())
                    .collect()
            })
            .unwrap_or_default();
        Self {
            roots,
            max_input: limit("INKSCAPE_MCP_MAX_INPUT_BYTES", 52428800),
            max_output: limit("INKSCAPE_MCP_MAX_OUTPUT_BYTES", 104857600),
        }
    }

    pub fn root_id(root: &Path) -> String {
        format!("{:x}", Sha256::digest(root.as_os_str().as_bytes()))[..24].to_owned()
    }

    pub fn info(&self) -> Value {
        json!({
            "roots": self.roots.iter().map(|root| json!({"root_id":Self::root_id(root),
                "name":root.file_name().unwrap_or_default().to_string_lossy(),
                "location":"server"})).collect::<Vec<_>>(),
            "relative_path_root_id":self.roots.first().map(|root| Self::root_id(root)),
            "relative_path_rule":"Relative paths resolve against the first root, never client CWD.",
            "artifact_access":"Read inkscape://artifact resource URIs through MCP; paths are server-side."
        })
    }

    pub fn resolve_input(
        &self,
        raw: &str,
        root_id: Option<&str>,
    ) -> Result<(usize, PathBuf), String> {
        let path = Path::new(raw);
        let anchored = if let Some(id) = root_id {
            let root = self
                .roots
                .iter()
                .find(|r| Self::root_id(r) == id)
                .ok_or("workspace root ID not found; call get_workspace_info")?;
            if path.is_absolute()
                || path.components().any(|c| c == Component::ParentDir)
                || raw.trim().is_empty()
                || raw.contains('\0')
            {
                return Err("root-qualified path must be a relative path without '..'".into());
            }
            root.join(path)
        } else if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.roots.first().ok_or(ESCAPE)?.join(path)
        };
        let resolved = anchored.canonicalize().map_err(|_| {
            // Existing symlinks are resolved first, matching Python. For a missing path,
            // normalize lexical parents only to distinguish an escape from missing input.
            let mut lexical = PathBuf::new();
            for component in anchored.components() {
                if component == Component::ParentDir {
                    lexical.pop();
                } else if component != Component::CurDir {
                    lexical.push(component);
                }
            }
            if self.roots.iter().any(|root| lexical.starts_with(root)) {
                "path rejected: could not resolve path"
            } else {
                ESCAPE
            }
        })?;
        let index = self
            .roots
            .iter()
            .position(|root| resolved.starts_with(root))
            .ok_or(ESCAPE)?;
        Ok((
            index,
            resolved
                .strip_prefix(&self.roots[index])
                .unwrap()
                .to_path_buf(),
        ))
    }

    pub fn read(&self, index: usize, relative: &Path, cap: usize) -> Result<Vec<u8>, String> {
        bounded_read(self.regular_file(index, relative)?, cap)
    }
    pub fn regular_file(&self, index: usize, relative: &Path) -> Result<File, String> {
        let (parent, name) = self.parent(index, relative, false)?;
        let handle = open_at(
            &parent,
            &name,
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            0,
        )?;
        if !handle.metadata().map_err(|_| "file unavailable")?.is_file() {
            return Err("path rejected: not a regular file".into());
        }
        Ok(handle)
    }

    /// Open an advisory lock through the same pinned no-follow directory descent.
    pub fn lock_file(&self, index: usize, relative: &Path) -> Result<File, String> {
        let (parent, name) = self.parent(index, relative, false)?;
        let handle = open_at(
            &parent,
            &name,
            libc::O_RDWR | libc::O_CREAT | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            0o600,
        )?;
        if !handle.metadata().map_err(|_| "lock unavailable")?.is_file() {
            return Err("path rejected: not a regular file".into());
        }
        Ok(handle)
    }

    pub fn modified(&self, index: usize, relative: &Path) -> Option<std::time::SystemTime> {
        let (parent, name) = self.parent(index, relative, false).ok()?;
        let handle = open_at(
            &parent,
            &name,
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            0,
        )
        .ok()?;
        let metadata = handle.metadata().ok()?;
        if !metadata.is_file() {
            return None;
        }
        metadata.modified().ok()
    }

    pub fn write_new(&self, index: usize, relative: &Path, bytes: &[u8]) -> Result<(), String> {
        let (parent, name) = self.parent(index, relative, true)?;
        let mut handle = open_at(
            &parent,
            &name,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW,
            0o600,
        )?;
        let result = handle.write_all(bytes).and_then(|_| handle.sync_all());
        if result.is_err() {
            // Remove a failed exclusive creation only if the name still refers
            // to the newly opened inode. Never follow or remove a replaced link.
            use std::os::unix::fs::MetadataExt;
            if let Ok(owned) = handle.metadata() {
                let mut current = std::mem::MaybeUninit::<libc::stat>::uninit();
                // SAFETY: live descriptor/name; stat read only on successful initialization.
                let status = unsafe {
                    libc::fstatat(
                        parent.as_raw_fd(),
                        name.as_ptr(),
                        current.as_mut_ptr(),
                        libc::AT_SYMLINK_NOFOLLOW,
                    )
                };
                if status == 0 {
                    let current = unsafe { current.assume_init() };
                    if owned.ino() == current.st_ino && owned.dev() == current.st_dev as u64 {
                        unsafe {
                            libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), 0);
                        }
                    }
                }
            }
            return Err("write failed".into());
        }
        Ok(())
    }

    /// Publish a complete new file atomically, never replacing a concurrently created name.
    pub fn atomic_create(&self, index: usize, relative: &Path, bytes: &[u8]) -> Result<(), String> {
        let (parent, name) = self.parent(index, relative, true)?;
        let temporary = CString::new(format!(".tmp-{}", uuid::Uuid::new_v4().simple())).unwrap();
        let mut handle = open_at(
            &parent,
            &temporary,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW,
            0o600,
        )?;
        let result = handle.write_all(bytes).and_then(|_| handle.sync_all());
        if result.is_err() {
            // SAFETY: generated temporary name under the pinned directory.
            unsafe {
                libc::unlinkat(parent.as_raw_fd(), temporary.as_ptr(), 0);
            }
            return Err("write failed".into());
        }
        // SAFETY: same pinned directory; linkat creates the destination exclusively,
        // unlike renameat, and never follows a pre-existing destination symlink.
        let status = unsafe {
            libc::linkat(
                parent.as_raw_fd(),
                temporary.as_ptr(),
                parent.as_raw_fd(),
                name.as_ptr(),
                0,
            )
        };
        let error = std::io::Error::last_os_error();
        // SAFETY: remove only the server-minted staging name after publication/refusal.
        unsafe {
            libc::unlinkat(parent.as_raw_fd(), temporary.as_ptr(), 0);
        }
        if status != 0 {
            return Err(if error.raw_os_error() == Some(libc::EEXIST) {
                "destination already exists"
            } else {
                "atomic creation failed"
            }
            .into());
        }
        parent
            .sync_all()
            .map_err(|_| "directory sync failed".into())
    }

    pub fn atomic_write(&self, index: usize, relative: &Path, bytes: &[u8]) -> Result<(), String> {
        let (parent, name) = self.parent(index, relative, true)?;
        let temporary = CString::new(format!(".tmp-{}", uuid::Uuid::new_v4().simple())).unwrap();
        let mut handle = open_at(
            &parent,
            &temporary,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW,
            0o600,
        )?;
        let result = handle.write_all(bytes).and_then(|_| handle.sync_all());
        if result.is_err() {
            // SAFETY: valid live directory descriptor and NUL-terminated generated name.
            unsafe {
                libc::unlinkat(parent.as_raw_fd(), temporary.as_ptr(), 0);
            }
            return Err("write failed".into());
        }
        // SAFETY: both names are relative to the same pinned directory descriptor.
        let status = unsafe {
            libc::renameat(
                parent.as_raw_fd(),
                temporary.as_ptr(),
                parent.as_raw_fd(),
                name.as_ptr(),
            )
        };
        if status != 0 {
            // SAFETY: only the generated temporary name in the pinned directory is removed.
            unsafe {
                libc::unlinkat(parent.as_raw_fd(), temporary.as_ptr(), 0);
            }
            return Err("atomic replacement failed".into());
        }
        parent
            .sync_all()
            .map_err(|_| "directory sync failed".into())
    }

    fn parent(
        &self,
        index: usize,
        relative: &Path,
        create: bool,
    ) -> Result<(File, CString), String> {
        let root = self.roots.get(index).ok_or("workspace root not found")?;
        if relative.is_absolute()
            || relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(ESCAPE.into());
        }
        let slash = CString::new("/").unwrap();
        // SAFETY: constant terminated path and read-only directory flags.
        let fd = unsafe {
            libc::open(
                slash.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err("workspace unavailable".into());
        }
        // SAFETY: open returned an owned descriptor, transferred exactly once.
        let mut directory = unsafe { File::from_raw_fd(fd) };
        for part in root.components().filter_map(|c| {
            if let Component::Normal(p) = c {
                Some(p)
            } else {
                None
            }
        }) {
            let name = CString::new(part.as_bytes()).map_err(|_| "invalid path")?;
            directory = open_at(
                &directory,
                &name,
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW,
                0,
            )?;
        }
        let components: Vec<_> = relative.components().collect();
        let (last, parents) = components.split_last().ok_or("invalid path")?;
        for part in parents {
            let name = CString::new(part.as_os_str().as_bytes()).map_err(|_| "invalid path")?;
            if create {
                // SAFETY: owned pinned descriptor and one validated relative component.
                let status = unsafe { libc::mkdirat(directory.as_raw_fd(), name.as_ptr(), 0o700) };
                if status != 0
                    && std::io::Error::last_os_error().raw_os_error() != Some(libc::EEXIST)
                {
                    return Err("could not create workspace directory".into());
                }
            }
            directory = open_at(
                &directory,
                &name,
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW,
                0,
            )?;
        }
        Ok((
            directory,
            CString::new(last.as_os_str().as_bytes()).map_err(|_| "invalid path")?,
        ))
    }
}

fn bounded_read(handle: File, cap: usize) -> Result<Vec<u8>, String> {
    let metadata = handle.metadata().map_err(|_| "file unavailable")?;
    if !metadata.is_file() {
        return Err("path rejected: not a regular file".into());
    }
    if metadata.len() > cap as u64 {
        return Err("input file exceeds the configured size limit".into());
    }
    let mut bytes = Vec::new();
    handle
        .take(cap.saturating_add(1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "file unavailable")?;
    if bytes.len() > cap {
        return Err("input file exceeds the configured size limit".into());
    }
    Ok(bytes)
}

fn open_at(
    directory: &File,
    name: &CString,
    flags: i32,
    mode: libc::mode_t,
) -> Result<File, String> {
    // SAFETY: descriptor is alive, name is NUL-terminated, caller validates one component.
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_CLOEXEC,
            mode as libc::c_uint,
        )
    };
    if fd < 0 {
        return Err("path rejected: could not access file safely".into());
    }
    // SAFETY: successful openat creates a fresh owned descriptor.
    Ok(unsafe { File::from_raw_fd(fd) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_files_are_refused_without_waiting_for_a_fifo_peer() {
        let root = tempfile::tempdir().unwrap();
        let canonical = root.path().canonicalize().unwrap();
        let fifo = canonical.join("pipe");
        let name = CString::new(fifo.as_os_str().as_bytes()).unwrap();
        // SAFETY: owned fixture path and fixed private permissions.
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        let workspace = Workspace {
            roots: vec![canonical],
            max_input: 1024,
            max_output: 1024,
        };
        for operation in 0..4 {
            let (cancel, cancelled) = std::sync::mpsc::channel();
            let owned_name = name.clone();
            // If O_NONBLOCK regresses, release the read-only open with an owned
            // peer, then fail the assertion instead of hanging the test process.
            let watchdog = std::thread::spawn(move || {
                if cancelled
                    .recv_timeout(std::time::Duration::from_secs(1))
                    .is_ok()
                {
                    return false;
                }
                // SAFETY: existing owned FIFO; never creates or follows a link.
                let fd = unsafe {
                    libc::open(
                        owned_name.as_ptr(),
                        libc::O_RDWR | libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                    )
                };
                assert!(fd >= 0);
                let _peer = unsafe { File::from_raw_fd(fd) };
                let _ = cancelled.recv_timeout(std::time::Duration::from_secs(5));
                true
            });
            let error = match operation {
                0 => workspace.regular_file(0, Path::new("pipe")).unwrap_err(),
                1 => workspace
                    .read_optional(0, Path::new("pipe"), 1024)
                    .unwrap_err(),
                2 => workspace.hash_file(0, Path::new("pipe")).unwrap_err(),
                _ => workspace.lock_file(0, Path::new("pipe")).unwrap_err(),
            };
            cancel.send(()).unwrap();
            assert!(
                !watchdog.join().unwrap(),
                "FIFO operation waited for a peer"
            );
            assert_eq!(error, "path rejected: not a regular file");
        }
        assert_eq!(
            workspace.file_kind(0, Path::new("pipe")).unwrap(),
            Some(libc::S_IFIFO)
        );
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }

    #[test]
    fn atomic_new_publication_has_one_winner_and_never_replaces_links() {
        let root = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![root.path().canonicalize().unwrap()],
            max_input: 4096,
            max_output: 4096,
        };
        let winners = std::thread::scope(|scope| {
            let handles = (0..8)
                .map(|i| {
                    let w = &workspace;
                    scope.spawn(move || {
                        let bytes = vec![i as u8; 4096];
                        w.atomic_create(0, Path::new("winner.bin"), &bytes).is_ok()
                    })
                })
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .filter(|b| *b)
                .count()
        });
        assert_eq!(winners, 1);
        let bytes = workspace.read(0, Path::new("winner.bin"), 4096).unwrap();
        assert_eq!(bytes.len(), 4096);
        assert!(bytes.iter().all(|b| *b == bytes[0]));
        std::fs::write(external.path().join("outside"), b"original").unwrap();
        std::os::unix::fs::symlink(
            external.path().join("outside"),
            root.path().join("linked.bin"),
        )
        .unwrap();
        assert!(
            workspace
                .atomic_create(0, Path::new("linked.bin"), b"changed")
                .is_err()
        );
        assert_eq!(
            std::fs::read(external.path().join("outside")).unwrap(),
            b"original"
        );
        assert!(root.path().join("linked.bin").is_symlink());
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 2);
    }
    #[test]
    fn directory_enumeration_is_bounded_and_does_not_follow_parent_links() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = Workspace {
            roots: vec![temp.path().canonicalize().unwrap()],
            max_input: 1024,
            max_output: 1024,
        };
        workspace
            .write_new(0, Path::new("frames/file"), b"test")
            .unwrap();
        std::os::unix::fs::symlink("/outside", temp.path().join("frames/link")).unwrap();
        let entries = workspace.directory_entries(0, Path::new("frames")).unwrap();
        assert!(entries.contains(&("file".into(), libc::S_IFREG)));
        assert!(entries.contains(&("link".into(), libc::S_IFLNK)));
        std::os::unix::fs::symlink(temp.path().join("frames"), temp.path().join("alias")).unwrap();
        assert!(workspace.directory_entries(0, Path::new("alias")).is_err());
        assert!(
            workspace
                .directory_entries(0, Path::new("../escape"))
                .is_err()
        );
        for n in 0..10001 {
            std::fs::File::create(temp.path().join("frames").join(format!("entry-{n}"))).unwrap();
        }
        assert!(workspace.directory_entries(0, Path::new("frames")).is_err());
    }

    #[test]
    fn no_follow_read_write_and_root_swap() {
        let path = std::env::temp_dir().join(format!("imcp-rust-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        let root = path.canonicalize().unwrap();
        let workspace = Workspace {
            roots: vec![root.clone()],
            max_input: 1024,
            max_output: 1024,
        };
        workspace
            .write_new(0, Path::new("nested/file"), b"original")
            .unwrap();
        assert_eq!(
            workspace.read(0, Path::new("nested/file"), 1024).unwrap(),
            b"original"
        );
        assert!(workspace.read(0, Path::new("nested/file"), 3).is_err());
        assert!(
            workspace
                .write_new(0, Path::new("nested/file"), b"replace")
                .is_err()
        );
        std::os::unix::fs::symlink("/etc/passwd", root.join("link")).unwrap();
        assert!(workspace.read(0, Path::new("link"), 1024).is_err());
        assert!(workspace.write_new(0, Path::new("link"), b"bad").is_err());
        std::os::unix::fs::symlink("/tmp", root.join("parent")).unwrap();
        assert!(
            workspace
                .write_new(0, Path::new("parent/escape"), b"bad")
                .is_err()
        );
        let moved = root.with_extension("moved");
        std::fs::rename(&root, &moved).unwrap();
        std::os::unix::fs::symlink(&moved, &root).unwrap();
        assert!(workspace.read(0, Path::new("nested/file"), 1024).is_err());
        std::fs::remove_file(root).unwrap();
        std::fs::remove_dir_all(moved).unwrap();
    }
}
