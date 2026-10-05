use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub struct Args {
    pub command: String,
    pub values: BTreeMap<String, String>,
}
impl Args {
    pub fn parse() -> Result<Self> {
        let mut input = std::env::args().skip(1);
        let command = input.next().unwrap_or_else(|| "help".into());
        let mut values = BTreeMap::new();
        while let Some(key) = input.next() {
            if !key.starts_with("--") || values.contains_key(&key) {
                return Err(format!("unknown/duplicate option: {key}").into());
            }
            let value = if ["--working-tree", "--matrix"].contains(&key.as_str()) {
                "true".into()
            } else {
                input.next().ok_or("option needs a value")?
            };
            values.insert(key, value);
        }
        Ok(Self { command, values })
    }
    pub fn check(&self, allowed: &[&str]) -> Result<()> {
        for key in self.values.keys() {
            if !allowed.contains(&key.as_str()) {
                return Err(format!("unknown option: {key}").into());
            }
        }
        Ok(())
    }
    pub fn required(&self, name: &str) -> Result<PathBuf> {
        Ok(PathBuf::from(
            self.values
                .get(name)
                .ok_or(format!("required option: {name}"))?,
        ))
    }
    pub fn path(&self, name: &str, default: &str) -> PathBuf {
        PathBuf::from(self.values.get(name).map(String::as_str).unwrap_or(default))
    }
    pub fn flag(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }
}
pub fn ensure(ok: bool, message: impl Into<String>) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.into().into())
    }
}
pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn hash(path: &Path) -> Result<String> {
    Ok(sha(&read(path, 256 * 1024 * 1024)?))
}
pub fn read(path: &Path, cap: u64) -> Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    ensure(file.metadata()?.len() <= cap, "file exceeds input cap")?;
    let mut data = Vec::new();
    (&mut file).take(cap + 1).read_to_end(&mut data)?;
    ensure(data.len() as u64 <= cap, "file grew beyond input cap")?;
    Ok(data)
}
pub fn json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&read(path, 64 * 1024 * 1024)?)?)
}
pub fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}
pub fn copy(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target.parent().ok_or("copy parent unavailable")?)?;
    fs::copy(source, target)?;
    Ok(())
}
pub fn walk(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_owned()];
    while let Some(path) = pending.pop() {
        ensure(
            files.len() + pending.len() < 100_000,
            "file inventory cap exceeded",
        )?;
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(path)? {
                pending.push(entry?.path());
            }
        } else if metadata.is_file() {
            files.push(path)
        } else {
            return Err("special inventory file".into());
        }
    }
    files.sort();
    Ok(files)
}
pub fn tool(name: &str) -> Result<PathBuf> {
    for directory in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let path = directory.join(name);
        if path.is_file() {
            return Ok(path);
        }
    }
    if ["cargo", "rustc"].contains(&name) {
        let path = PathBuf::from(std::env::var_os("HOME").ok_or("HOME unavailable")?)
            .join(".cargo/bin")
            .join(name);
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(format!("required native tool missing: {name}").into())
}
/// Capture owned build-tool output with a fixed timeout and finite spill files.
pub fn output(command: &mut Command) -> Result<String> {
    let temporary = tempfile::tempdir()?;
    let out = temporary.path().join("stdout");
    let err = temporary.path().join("stderr");
    command
        .stdout(Stdio::from(fs::File::create(&out)?))
        .stderr(Stdio::from(fs::File::create(&err)?));
    let mut child = command.spawn()?;
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > Duration::from_secs(120)
            || fs::metadata(&out)?.len() + fs::metadata(&err)?.len() > 64 * 1024 * 1024
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err("owned build process exceeded time/output budget".into());
        }
        thread::sleep(Duration::from_millis(10));
    };
    ensure(
        status.success(),
        format!(
            "build tool failed: {}",
            String::from_utf8_lossy(&read(&err, 64 * 1024 * 1024)?)
        ),
    )?;
    Ok(String::from_utf8(read(&out, 64 * 1024 * 1024)?)?)
}
pub fn command(program: impl AsRef<std::ffi::OsStr>, args: &[&str]) -> Result<String> {
    output(Command::new(program).args(args))
}
pub fn relative(path: &Path, base: &Path) -> Result<PathBuf> {
    let a: Vec<_> = path.components().collect();
    let b: Vec<_> = base.components().collect();
    let same = a.iter().zip(&b).take_while(|(a, b)| a == b).count();
    ensure(same > 0, "relative path has no shared root")?;
    let mut result = PathBuf::new();
    for _ in same..b.len() {
        result.push("..")
    }
    for part in &a[same..] {
        result.push(part.as_os_str())
    }
    Ok(result)
}
