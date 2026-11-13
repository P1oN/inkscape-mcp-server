//! Read-only evidence for installer recovery. Discovered commands are never executed.
use super::{
    install::*,
    instructions::{self, read},
    manifests::*,
};
use serde::Serialize;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize)]
pub(super) struct Assessment {
    pub status: &'static str,
    pub source: Option<PathBuf>,
    pub explanation: &'static str,
}
fn unknown() -> Assessment {
    Assessment {
        status: "uncertain",
        source: None,
        explanation: "The binding cannot be confirmed as this project's installation. Preserve it; deselect this client or select a verified source.",
    }
}
pub(super) fn known_skill(root: &Path, skill: &Path) -> bool {
    let names = ["SKILL.md", "agents/openai.yaml"];
    let Ok(baseline) = names
        .into_iter()
        .map(|name| {
            read(&skill.join(".inkscape-mcp-upstream").join(name), 256 * 1024).map(|bytes| {
                (
                    format!("skills/inkscape-mcp/{name}"),
                    FileIdentity::of(&bytes),
                )
            })
        })
        .collect::<Result<std::collections::BTreeMap<_, _>>>()
    else {
        return false;
    };
    let matches = |files: &std::collections::BTreeMap<String, FileIdentity>| {
        baseline
            .iter()
            .all(|(name, id)| files.get(name) == Some(id))
    };
    let compiled = instructions::manifest("compiled", &instructions::default_files());
    if matches(&compiled.files) {
        return true;
    }
    // Historical baselines may be proven by a fully validated, already staged text
    // bundle. A directory name or an unvalidated manifest alone proves nothing.
    let directory = root.join("instructions");
    if instructions::guarded(&directory).is_err() {
        return false;
    }
    let Ok(entries) = fs::read_dir(directory) else {
        return false;
    };
    for entry in entries.take(65) {
        let Ok(entry) = entry else {
            return false;
        };
        if let Ok(bundle) = instructions::Instructions::load(&entry.path())
            && matches(&bundle.manifest.files)
        {
            return true;
        }
    }
    false
}
pub(super) fn assess(root: &Path, entry: Option<&Value>, skill: &Path) -> Assessment {
    let Some(entry) = entry else {
        return Assessment {
            status: "unconfigured",
            source: None,
            explanation: "No registration will be replaced.",
        };
    };
    if entry
        .get("args")
        .is_some_and(|v| v != &serde_json::json!([]))
        || entry.get("url").is_some_and(|v| !v.is_null() && v != "")
        || entry.get("type").is_some_and(|v| v != "stdio")
    {
        return unknown();
    }
    let Some(command) = entry["command"].as_str().map(Path::new) else {
        return unknown();
    };
    if !command.is_absolute()
        || command.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return unknown();
    }
    if command == root.join("bin/inkscape-mcp-launcher") {
        return Assessment {
            status: "selected",
            source: Some(root.into()),
            explanation: "The registration points to the selected installation.",
        };
    }
    if command.file_name().is_some_and(|n| n == "run-mcp.sh") {
        let source = command.parent().unwrap();
        if instructions::guarded(source).is_ok()
            && import_settings(source).is_ok()
            && command.is_file()
        {
            return Assessment {
                status: "existing",
                source: Some(source.into()),
                explanation: "An existing source has readable project setup metadata. Select it in Advanced to verify and upgrade it.",
            };
        }
        // Require the entire source to be absent, safe ancestors, a matching
        // owner marker AND a known baseline. Never infer ownership from the name.
        let safe_missing = !source.exists()
            && !source.is_symlink()
            && source
                .ancestors()
                .find(|p| p.exists() || p.is_symlink())
                .is_some_and(|p| instructions::guarded(p).is_ok());
        if safe_missing
            && known_skill(root, skill)
            && read(&skill.join(".inkscape-mcp-owner"), 8192).is_ok_and(|b| {
                b == source.as_os_str().as_encoded_bytes()
                    || b == format!("{}\n", source.display()).as_bytes()
            })
        {
            return Assessment {
                status: "stale",
                source: Some(source.into()),
                explanation: "The removed source matches this skill's owner and verified upstream baseline. Review can back up and replace this registration and transfer the managed skill.",
            };
        }
    }
    if command
        .file_name()
        .is_some_and(|n| n == "inkscape-mcp-launcher" || n == "inkscape-mcp")
        && let Some(source) = command.parent().and_then(Path::parent)
        && command.parent() == Some(source.join("bin").as_path())
        && instructions::guarded(source).is_ok()
        && selector(source).is_ok_and(|s| validate_pair(source, &s.current).is_ok())
    {
        return Assessment {
            status: "existing",
            source: Some(source.into()),
            explanation: "A verified permanent installation exists. Select it in Advanced rather than replacing its registration.",
        };
    }
    unknown()
}

#[cfg(test)]
mod tests {
    use super::super::storage::*;
    use super::*;
    use serde_json::json;
    #[test]
    fn stale_requires_matching_owner_known_baseline_and_absent_source() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let root = base.join("installation");
        let source = base.join("Downloads/inkscape-mcp-macos-arm64");
        let skill = base.join("profile/skills/inkscape-mcp");
        mkdir(&base.join("Downloads")).unwrap();
        mkdir(&skill.join(".inkscape-mcp-upstream/agents")).unwrap();
        let entry = json!({"command":source.join("run-mcp.sh"),"args":[],"env":{"PRIVATE":"fixture-secret"}});
        assert_eq!(assess(&root, Some(&entry), &skill).status, "uncertain");
        write(
            &skill.join(".inkscape-mcp-owner"),
            source.as_os_str().as_encoded_bytes(),
        )
        .unwrap();
        assert_eq!(assess(&root, Some(&entry), &skill).status, "uncertain");
        let files = instructions::default_files();
        for name in ["SKILL.md", "agents/openai.yaml"] {
            write(
                &skill.join(".inkscape-mcp-upstream").join(name),
                &files[&format!("skills/inkscape-mcp/{name}")],
            )
            .unwrap();
        }
        assert_eq!(assess(&root, Some(&entry), &skill).status, "stale");
        assert!(
            !serde_json::to_string(&assess(&root, Some(&entry), &skill))
                .unwrap()
                .contains("fixture-secret")
        );
        // User content is deliberately independent of upstream ownership proof.
        write(&skill.join("SKILL.md"), b"user customization").unwrap();
        assert_eq!(assess(&root, Some(&entry), &skill).status, "stale");
        mkdir(&source).unwrap();
        write(&source.join("run-mcp.sh"), b"must never execute").unwrap();
        assert_eq!(assess(&root, Some(&entry), &skill).status, "uncertain");
        mkdir(&source.join(".inkscape-mcp-local")).unwrap();
        write(&source.join(".inkscape-mcp-local/setup.conf"), format!("inkscape-mcp-setup-v1\n{}/package/bin/inkscape-mcp\n/usr/bin\n{}\nfalse\nper_call\n",base.display(),base.display()).as_bytes()).unwrap();
        assert_eq!(assess(&root, Some(&entry), &skill).status, "existing");
        let mut remote = entry.clone();
        remote["args"] = json!(["--unknown"]);
        assert_eq!(assess(&root, Some(&remote), &skill).status, "uncertain");
    }
    #[test]
    fn foreign_marker_or_unknown_baseline_is_preserved() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let skill = base.join("skill");
        mkdir(&skill.join(".inkscape-mcp-upstream/agents")).unwrap();
        write(
            &skill.join(".inkscape-mcp-owner"),
            base.join("foreign").as_os_str().as_encoded_bytes(),
        )
        .unwrap();
        for name in ["SKILL.md", "agents/openai.yaml"] {
            write(
                &skill.join(".inkscape-mcp-upstream").join(name),
                b"unknown baseline",
            )
            .unwrap();
        }
        let entry = json!({"command":base.join("inkscape-mcp/run-mcp.sh"),"args":[]});
        assert_eq!(
            assess(&base.join("root"), Some(&entry), &skill).status,
            "uncertain"
        );
    }
}
