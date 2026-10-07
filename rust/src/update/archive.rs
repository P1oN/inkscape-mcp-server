//! Shared archive validation. Complete validation precedes extraction.
use flate2::read::GzDecoder;
use std::{
    collections::HashSet,
    fs,
    path::{Component, Path},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn ensure(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
pub fn safe(path: &Path) -> bool {
    path.components()
        .all(|p| matches!(p, Component::Normal(_) | Component::CurDir))
        && !path.as_os_str().is_empty()
}
/// Validate the complete archive before publishing any extracted files.
pub fn extract(archive: &Path, destination: &Path, prefix: &Path) -> Result<()> {
    let mut names = HashSet::new();
    let mut size = 0u64;
    let mut stream = tar::Archive::new(GzDecoder::new(fs::File::open(archive)?));
    for (index, entry) in stream.entries()?.enumerate() {
        let entry = entry?;
        let path = entry.path()?.into_owned();
        let kind = entry.header().entry_type();
        ensure(
            index < 50_000 && safe(&path) && names.insert(path.clone()),
            "unsafe/duplicate native archive path",
        )?;
        ensure(
            path.starts_with(prefix) || (kind.is_dir() && prefix.starts_with(&path)),
            "unexpected native archive root",
        )?;
        ensure(
            kind.is_file() || kind.is_dir() || kind.is_symlink() || kind.is_hard_link(),
            "special native archive member",
        )?;
        size = size
            .checked_add(entry.size())
            .ok_or("archive size overflow")?;
        ensure(
            size <= 512 * 1024 * 1024,
            "native archive exceeds extraction cap",
        )?;
        if let Some(link) = entry.link_name()? {
            let target = if kind.is_symlink() {
                path.parent().unwrap().join(&*link)
            } else {
                link.into_owned()
            };
            // Lexically normalize inside the prefix; tar's unpack_in also guards symlink ancestors.
            let mut parts = Vec::new();
            for part in target.components() {
                match part {
                    Component::Normal(p) => parts.push(p),
                    Component::CurDir => (),
                    Component::ParentDir => {
                        ensure(parts.pop().is_some(), "native archive link escaped")?;
                    }
                    _ => return Err("absolute archive link".into()),
                }
            }
            let normalized: std::path::PathBuf = parts.iter().collect();
            ensure(
                normalized.starts_with(prefix),
                "native archive link escaped formula",
            )?;
        }
    }
    let mut stream = tar::Archive::new(GzDecoder::new(fs::File::open(archive)?));
    for entry in stream.entries()? {
        ensure(
            entry?.unpack_in(destination)?,
            "archive path escaped output",
        )?;
    }
    Ok(())
}
