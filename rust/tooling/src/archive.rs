use crate::common::*;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
pub use inkscape_mcp_rust::update::archive::{extract, safe};
use std::{collections::HashSet, fs, io::Read, path::Path, process::Command};
pub fn pack(directory: &Path, archive: &Path) -> Result<()> {
    ensure(
        !archive.starts_with(directory),
        "archive must be outside package",
    )?;
    let file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(archive)?;
    let mut builder = tar::Builder::new(GzEncoder::new(file, Compression::default()));
    builder.append_dir_all(
        directory.file_name().ok_or("package name unavailable")?,
        directory,
    )?;
    builder.into_inner()?.finish()?;
    Ok(())
}
pub fn source(args: &Args) -> Result<()> {
    args.check(&["--output", "--working-tree"])?;
    let target = args.required("--output")?;
    let revision = command("git", &["rev-parse", "--verify", "HEAD"])?
        .trim()
        .to_string();
    ensure(
        revision.len() == 40
            && revision
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "invalid source commit",
    )?;
    if let Some(parent) = target.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&target)?;
    let mut builder = tar::Builder::new(GzEncoder::new(file, Compression::default()));
    let prefix = Path::new("inkscape-mcp-source-bootstrap");
    if args.flag("--working-tree") {
        let tracked = command("git", &["ls-files", "-z"])?;
        let additions = command(
            "git",
            &[
                "ls-files",
                "--others",
                "--exclude-standard",
                "-z",
                "--",
                "scripts",
                "skills",
                "runtime",
                "docs",
                "rust/src",
                "rust/tests",
                "rust/tooling",
                "rust/build.rs",
                "uninstall.sh",
            ],
        )?;
        let mut names: Vec<_> = tracked
            .split('\0')
            .chain(additions.split('\0'))
            .filter(|s| !s.is_empty())
            .collect();
        names.sort();
        names.dedup();
        for name in names {
            let path = Path::new(name);
            ensure(!path.is_symlink(), "working-tree export refuses symlinks")?;
            if path.is_file() {
                ensure(
                    name != "SOURCE_REVISION" && name != "SOURCE_STATE",
                    "source metadata already tracked",
                )?;
                builder.append_path_with_name(path, prefix.join(path))?;
            }
        }
        append(
            &mut builder,
            &prefix.join("SOURCE_STATE"),
            b"uncommitted-working-tree
",
        )?;
    } else {
        let bytes = tempfile::NamedTempFile::new()?;
        output(
            Command::new("git")
                .args(["archive", "--format=tar", "--output"])
                .arg(bytes.path())
                .arg(&revision),
        )?;
        let mut stream = tar::Archive::new(fs::File::open(bytes.path())?);
        for entry in stream.entries()? {
            let mut entry = entry?;
            // git archive emits a global PAX commit comment. It is not a source file;
            // retain identity through our explicit SOURCE_REVISION instead, so exports
            // remain compatible with the strict native archive extractor.
            if entry.header().entry_type().is_pax_global_extensions() {
                continue;
            }
            let path = entry.path()?.into_owned();
            ensure(
                path != Path::new("SOURCE_REVISION"),
                "source revision already tracked",
            )?;
            let mut header = entry.header().clone();
            builder.append_data(&mut header, prefix.join(&path), &mut entry)?;
        }
        append(
            &mut builder,
            &prefix.join("SOURCE_REVISION"),
            format!(
                "inkscape-mcp-source-v1
{revision}
"
            )
            .as_bytes(),
        )?;
    }
    builder.into_inner()?.finish()?;
    println!("{}", target.display());
    Ok(())
}
fn append<W: std::io::Write>(
    builder: &mut tar::Builder<W>,
    path: &Path,
    bytes: &[u8],
) -> Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append_data(&mut header, path, bytes)?;
    Ok(())
}
pub fn crate_notices(
    archive: &Path,
    checksum: &str,
    name: &str,
    version: &str,
) -> Result<(
    toml::Value,
    serde_json::Value,
    std::collections::BTreeMap<String, Vec<u8>>,
)> {
    ensure(
        fs::metadata(archive)?.len() <= 64 * 1024 * 1024 && hash(archive)? == checksum,
        "crate archive differs from lock/input cap",
    )?;
    let prefix = format!("{name}-{version}/");
    let mut notices = std::collections::BTreeMap::new();
    let mut total = 0;
    let mut stream = tar::Archive::new(GzDecoder::new(fs::File::open(archive)?));
    for (index, entry) in stream.entries()?.enumerate() {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().into_owned();
        ensure(
            index < 100_000 && path.starts_with(&prefix),
            "unexpected crate root/member cap",
        )?;
        let relative = &path[prefix.len()..];
        ensure(safe(Path::new(relative)), "unsafe crate path")?;
        let selected = Path::new(relative).components().any(|p| {
            [
                "license",
                "licence",
                "copying",
                "copyright",
                "notice",
                "unlicense",
            ]
            .iter()
            .any(|prefix| {
                p.as_os_str()
                    .to_string_lossy()
                    .to_lowercase()
                    .starts_with(prefix)
            })
        });
        if !selected && !["Cargo.toml", ".cargo_vcs_info.json"].contains(&relative) {
            continue;
        }
        ensure(
            entry.header().entry_type().is_file() && entry.size() <= 4 * 1024 * 1024,
            "unsafe/excessive crate notice",
        )?;
        total += entry.size();
        ensure(total <= 16 * 1024 * 1024, "crate notice total cap")?;
        let mut content = Vec::new();
        entry.read_to_end(&mut content)?;
        ensure(
            content.len() as u64 == entry.size() && !notices.contains_key(relative),
            "incomplete/duplicate crate notice",
        )?;
        notices.insert(relative.into(), content);
    }
    let metadata: toml::Value = toml::from_str(std::str::from_utf8(
        &notices
            .remove("Cargo.toml")
            .ok_or("crate Cargo.toml missing")?,
    )?)?;
    let vcs = serde_json::from_slice(
        &notices
            .remove(".cargo_vcs_info.json")
            .unwrap_or_else(|| b"{}".to_vec()),
    )?;
    Ok((metadata["package"].clone(), vcs, notices))
}

pub fn package_root(path: &Path) -> Result<std::path::PathBuf> {
    let mut stream = tar::Archive::new(GzDecoder::new(fs::File::open(path)?));
    let mut tops = HashSet::new();
    let mut bytes = 0u64;
    for (index, row) in stream.entries()?.enumerate() {
        let row = row?;
        let path = row.path()?;
        ensure(
            index < 10_000 && safe(&path),
            "invalid package archive path/file cap",
        )?;
        bytes = bytes
            .checked_add(row.size())
            .ok_or("package archive size overflow")?;
        ensure(
            bytes <= 512 * 1024 * 1024,
            "package archive expanded-size cap",
        )?;
        tops.insert(
            path.components()
                .next()
                .ok_or("empty package archive path")?
                .as_os_str()
                .to_os_string(),
        );
    }
    ensure(
        tops.len() == 1,
        "archive must contain exactly one package root",
    )?;
    Ok(tops.into_iter().next().unwrap().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(path: &Path, names: &[(&str, Option<&str>)]) {
        let mut stream = tar::Builder::new(GzEncoder::new(
            fs::File::create(path).unwrap(),
            Compression::default(),
        ));
        for (name, link) in names {
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            if let Some(link) = link {
                header.set_entry_type(tar::EntryType::Symlink);
                header.set_link_name(link).unwrap();
                header.set_size(0);
                header.set_cksum();
                stream.append_data(&mut header, name, &b""[..]).unwrap();
            } else {
                let content = if name.ends_with("Cargo.toml") {
                    b"[package]
name='fixture'
version='1.0'
license='MIT'
"
                    .as_slice()
                } else {
                    b"license"
                };
                header.set_size(content.len() as u64);
                header.set_cksum();
                stream.append_data(&mut header, name, content).unwrap();
            }
        }
        stream.into_inner().unwrap().finish().unwrap();
    }
    #[test]
    fn extraction_validates_every_member_before_any_write() {
        let root = tempfile::tempdir().unwrap();
        for (label, names, valid) in [
            ("valid", vec![("fixture/1.0/LICENSE", None)], true),
            (
                "duplicate",
                vec![("fixture/1.0/LICENSE", None), ("fixture/1.0/LICENSE", None)],
                false,
            ),
            (
                "root",
                vec![("fixture/1.0/LICENSE", None), ("outside", None)],
                false,
            ),
            (
                "link",
                vec![
                    ("fixture/1.0/LICENSE", None),
                    ("fixture/1.0/link", Some("/outside")),
                ],
                false,
            ),
            (
                "relative-link",
                vec![
                    ("fixture/1.0/LICENSE", None),
                    ("fixture/1.0/link", Some("../../../outside")),
                ],
                false,
            ),
        ] {
            let archive = root.path().join(format!("{label}.tar.gz"));
            fixture(&archive, &names);
            let destination = root.path().join(label);
            fs::create_dir(&destination).unwrap();
            let result = extract(&archive, &destination, Path::new("fixture/1.0"));
            assert_eq!(result.is_ok(), valid, "{label}: {result:?}");
            if !valid {
                assert_eq!(fs::read_dir(destination).unwrap().count(), 0);
            }
        }
        assert!(!safe(Path::new("fixture/1.0/../../outside")));
        assert!(!safe(Path::new("/outside")));
    }
    #[test]
    fn notices_are_hash_bound_and_never_extracted() {
        let root = tempfile::tempdir().unwrap();
        for (label, names, valid) in [
            (
                "valid",
                vec![
                    ("fixture-1.0/Cargo.toml", None),
                    ("fixture-1.0/LICENSE", None),
                ],
                true,
            ),
            (
                "licence",
                vec![
                    ("fixture-1.0/Cargo.toml", None),
                    ("fixture-1.0/LICENCE", None),
                ],
                true,
            ),
            (
                "duplicate",
                vec![
                    ("fixture-1.0/Cargo.toml", None),
                    ("fixture-1.0/LICENSE", None),
                    ("fixture-1.0/LICENSE", None),
                ],
                false,
            ),
            (
                "link",
                vec![
                    ("fixture-1.0/Cargo.toml", None),
                    ("fixture-1.0/LICENSE", Some("/outside")),
                ],
                false,
            ),
        ] {
            let archive = root.path().join(format!("{label}.crate"));
            fixture(&archive, &names);
            let checksum = hash(&archive).unwrap();
            let result = crate_notices(&archive, &checksum, "fixture", "1.0");
            assert_eq!(result.is_ok(), valid, "{label}");
            assert!(crate_notices(&archive, &"0".repeat(64), "fixture", "1.0").is_err());
            assert!(!root.path().join("fixture-1.0").exists());
        }
    }
}
#[cfg(test)]
mod raw_header_tests {
    use super::*;
    use std::io::Write;
    fn raw(path: &Path, name: &str, size: u64) {
        let mut h = tar::Header::new_gnu();
        h.set_mode(0o600);
        h.set_size(size);
        h.set_entry_type(tar::EntryType::Regular);
        h.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
        h.set_cksum();
        let mut stream = GzEncoder::new(fs::File::create(path).unwrap(), Compression::default());
        stream.write_all(h.as_bytes()).unwrap();
        stream.write_all(&[0; 1024]).unwrap();
        stream.finish().unwrap();
    }
    #[test]
    fn raw_traversal_and_size_caps_fail_before_publication() {
        let root = tempfile::tempdir().unwrap();
        for (name, size) in [
            ("fixture/1.0/../../escape", 7),
            ("/fixture/1.0/LICENSE", 7),
            ("fixture/1.0/LICENSE", 512 * 1024 * 1024 + 1),
        ] {
            let archive = root.path().join("bad.tar.gz");
            raw(&archive, name, size);
            let out = tempfile::tempdir().unwrap();
            assert!(extract(&archive, out.path(), Path::new("fixture/1.0")).is_err());
            assert_eq!(fs::read_dir(out.path()).unwrap().count(), 0);
        }
        let archive = root.path().join("large-crate.tar.gz");
        raw(&archive, "fixture-1.0/LICENSE", 4 * 1024 * 1024 + 1);
        assert!(crate_notices(&archive, &hash(&archive).unwrap(), "fixture", "1.0").is_err());
    }
}
