//! Restart-proof frame numbering derived from bounded, no-follow directory observations.
use crate::{
    arguments,
    document::{Entry, Registry},
};
use regex::Regex;
use serde_json::{Value, json};
use std::{path::PathBuf, sync::LazyLock};

static FRAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^frame-([0-9]+)(?:-.*)?\.png$").unwrap());

pub struct Plan {
    pub series: String,
    pub index: u64,
    pub relative: PathBuf,
}

pub fn series(args: &Value) -> Result<String, String> {
    Ok(arguments::string(args, "series")?
        .map(crate::render::name)
        .filter(|s| !s.is_empty())
        .unwrap_or("run".into()))
}
fn directory(entry: &Entry, series: &str) -> PathBuf {
    entry.directory().join("artifacts/frames").join(series)
}
fn index(name: &str) -> Result<Option<u64>, String> {
    FRAME
        .captures(name)
        .map(|c| {
            c[1].parse()
                .map_err(|_| "frame index exceeds the supported range".into())
        })
        .transpose()
}

pub fn plan(registry: &Registry, entry: &Entry, args: &Value) -> Result<Plan, String> {
    let series = series(args)?;
    let label = arguments::string(args, "label")?
        .map(crate::render::name)
        .filter(|s| !s.is_empty())
        .map(|s| format!("-{s}"))
        .unwrap_or_default();
    let directory = directory(entry, &series);
    registry
        .workspace
        .ensure_directory(entry.root, &directory)
        .map_err(|_| "render/export failed")?;
    let entries = registry
        .workspace
        .directory_entries(entry.root, &directory)
        .map_err(|_| "render/export failed")?;
    let mut highest = 0;
    for (name, kind) in entries {
        if kind != libc::S_IFLNK
            && let Some(index) = index(&name)?
        {
            highest = highest.max(index);
        }
    }
    let index = highest
        .checked_add(1)
        .ok_or("frame index exceeds the supported range")?;
    Ok(Plan {
        series,
        index,
        relative: directory.join(format!("frame-{index:03}{label}.png")),
    })
}

pub fn list(registry: &Registry, args: &Value) -> Result<Value, String> {
    let id = args["doc_id"].as_str().ok_or("doc_id must be a string")?;
    let entry = registry.entries.get(id).ok_or("document id not found")?;
    let series = series(args)?;
    let directory = directory(entry, &series);
    let entries = registry
        .workspace
        .directory_entries(entry.root, &directory)
        .unwrap_or_default();
    let mut found = Vec::new();
    for (name, kind) in entries {
        if kind != libc::S_IFLNK
            && let Some(index) = index(&name)?
        {
            found.push((index, name));
        }
    }
    found.sort();
    let frames = found.into_iter().map(|(index,name)| {
        let path = directory.join(name).to_string_lossy().into_owned();
        json!({"frame_index":index,"artifact_path":path,"workspace_relative_path":path,"artifact":null})
    }).collect::<Vec<_>>();
    Ok(json!({"doc_id":id,"series":series,"frames":frames}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numbering_observes_disk_skips_links_and_preserves_exclusive_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let entry = Entry {
            id: "d_12345678".into(),
            root: 0,
            source: "fixture.svg".into(),
            opened_at: "test".into(),
        };
        let mut registry = Registry {
            workspace: crate::workspace::Workspace {
                roots: vec![temp.path().canonicalize().unwrap()],
                max_input: 1024,
                max_output: 1024,
            },
            entries: indexmap::IndexMap::new(),
        };
        let args = json!({"doc_id":entry.id,"series":"../ My Run /","label":"after / edit"});
        let first = plan(&registry, &entry, &args).unwrap();
        assert_eq!(first.series, "My-Run");
        assert_eq!(first.index, 1);
        assert!(first.relative.ends_with("frame-001-after-edit.png"));
        registry
            .workspace
            .write_new(0, &first.relative, b"first")
            .unwrap();
        assert!(
            registry
                .workspace
                .write_new(0, &first.relative, b"clobber")
                .is_err()
        );
        let directory = first.relative.parent().unwrap();
        registry
            .workspace
            .write_new(0, &directory.join("frame-007-seed.png"), b"seed")
            .unwrap();
        std::os::unix::fs::symlink(
            "/outside",
            temp.path().join(directory).join("frame-999-link.png"),
        )
        .unwrap();
        let next = plan(&registry, &entry, &args).unwrap();
        assert_eq!(next.index, 8);
        assert_eq!(
            registry.workspace.read(0, &first.relative, 1024).unwrap(),
            b"first"
        );
        registry.entries.insert(entry.id.clone(), entry);
        let listing = list(&registry, &args).unwrap();
        assert_eq!(listing["frames"].as_array().unwrap().len(), 2);
        assert_eq!(listing["frames"][0]["frame_index"], 1);
        assert_eq!(listing["frames"][1]["frame_index"], 7);
        // A different process/runtime instance derives the same counter from existing files.
        let registry2 = Registry {
            workspace: registry.workspace,
            entries: registry.entries,
        };
        assert_eq!(
            plan(&registry2, &registry2.entries["d_12345678"], &args)
                .unwrap()
                .index,
            8
        );
        let mut invalid = args.clone();
        invalid["series"] = json!("must-not-create");
        invalid["label"] = json!(123);
        assert!(plan(&registry2, &registry2.entries["d_12345678"], &invalid).is_err());
        assert!(
            !temp
                .path()
                .join(directory.parent().unwrap())
                .join("must-not-create")
                .exists()
        );
    }
}
