//! Durable multi-location skill/selector transaction. Interrupted operations restore the old pair.
use super::{
    install::*,
    instructions::{guarded, read},
    manifests::*,
    storage::*,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillChange {
    pub destination: PathBuf,
    pub before: BTreeMap<String, FileIdentity>,
    pub after: BTreeMap<String, FileIdentity>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Journal {
    pub format: u32,
    pub id: String,
    pub before: Selector,
    pub after: Selector,
    pub skills: Vec<SkillChange>,
}
fn transaction_path(root: &Path, id: &str) -> Result<PathBuf> {
    if !identifier(id) {
        return Err("invalid transaction identity".into());
    }
    Ok(root.join("backups").join(id))
}
fn move_path(source: &Path, target: &Path) -> Result<()> {
    guarded(source)?;
    guarded(target.parent().ok_or("missing move parent")?)?;
    if target.exists() || target.is_symlink() {
        return Err("transaction destination occupied".into());
    }
    fs::rename(source, target).map_err(|e| e.to_string())?;
    sync_dir(source.parent().unwrap())?;
    sync_dir(target.parent().unwrap())
}
fn record_name(pair: &Pair) -> String {
    FileIdentity::of(&serde_json::to_vec(pair).unwrap()).sha256
}
fn validate_journal(root: &Path, journal: &Journal, settings: &Settings) -> Result<PathBuf> {
    if journal.format != FORMAT
        || journal.skills.len() > 2
        || journal
            .skills
            .iter()
            .any(|s| !settings.skills.contains(&s.destination))
    {
        return Err("invalid update journal; recover installation manually".into());
    }
    let base = transaction_path(root, &journal.id)?;
    guarded(&base)?;
    for (i, skill) in journal.skills.iter().enumerate() {
        for files in [&skill.before, &skill.after] {
            if files.len() > 4096
                || files.iter().any(|(p, id)| {
                    !managed_relative(p) || (id.bytes > 8 * 1024 * 1024 || !hash(&id.sha256))
                })
            {
                return Err("invalid skill journal inventory".into());
            }
        }
        guarded(skill.destination.parent().ok_or("skill parent missing")?)?;
        for name in [format!("old-{i}"), format!("new-{i}")] {
            if base.join(&name).exists() {
                guarded(&base.join(name))?;
            }
        }
    }
    Ok(base)
}
pub fn recover(root: &Path, settings: &Settings) -> Result<bool> {
    let path = root.join("transaction.json");
    if !path.exists() && !path.is_symlink() {
        return Ok(false);
    }
    let journal: Journal = json(&path)?;
    let base = validate_journal(root, &journal, settings)?;
    let current = selector(root)?;
    if current != journal.before && current != journal.after {
        return Err("selector changed after interrupted update; manual recovery required".into());
    }
    // Preflight every destination before restoring anything.
    for (i, skill) in journal.skills.iter().enumerate() {
        let old = base.join(format!("old-{i}"));
        if skill.destination.exists() {
            let actual = inventory(&skill.destination, 16 * 1024 * 1024)?;
            if actual != skill.before && actual != skill.after {
                return Err(format!(
                    "skill edited after interrupted update; preserved at {}; restore from {} manually",
                    skill.destination.display(),
                    base.display()
                ));
            }
        }
        if old.exists() && inventory(&old, 16 * 1024 * 1024)? != skill.before {
            return Err("transaction backup changed".into());
        }
    }
    for (i, skill) in journal.skills.iter().enumerate().rev() {
        let old = base.join(format!("old-{i}"));
        if old.exists() {
            if skill.destination.exists() {
                move_path(&skill.destination, &base.join(format!("interrupted-{i}")))?;
            }
            move_path(&old, &skill.destination)?;
        } else if !skill.destination.exists()
            || inventory(&skill.destination, 16 * 1024 * 1024)? != skill.before
        {
            return Err("missing recovery skill backup".into());
        }
    }
    if current != journal.before {
        let selector_backup = base.join("selector-before.json");
        let saved: Selector = json(&selector_backup)?;
        if saved != journal.before {
            return Err("recovery selector backup differs".into());
        }
        fs::rename(selector_backup, root.join("active.json")).map_err(|e| e.to_string())?;
        sync_dir(root)?;
    }
    fs::remove_file(path).map_err(|e| e.to_string())?;
    sync_dir(root)?;
    Ok(true)
}
fn prepare_skill(
    root: &Path,
    base: &Path,
    index: usize,
    path: &Path,
    pair: &Pair,
    settings: &Settings,
) -> Result<SkillChange> {
    let before = inventory(path, 16 * 1024 * 1024)?;
    if !before.contains_key(".inkscape-mcp-owner") {
        return Err("skill is not managed; preserve it and migrate its baseline first".into());
    }
    let owner = String::from_utf8(read(&path.join(".inkscape-mcp-owner"), 8192)?)
        .map_err(|e| e.to_string())?;
    if owner.trim() != root.to_string_lossy()
        && owner.trim() != settings.legacy_skill_owner.to_string_lossy()
    {
        return Err("skill ownership changed; preserved".into());
    }
    let staged = base.join(format!("new-{index}"));
    copy_tree(path, &staged, 16 * 1024 * 1024)?;
    let instructions = instruction_path(root, pair)?;
    for name in ["SKILL.md", "agents/openai.yaml"] {
        let old = read(&path.join(".inkscape-mcp-upstream").join(name), 256 * 1024)?;
        let local = read(&path.join(name), 256 * 1024)?;
        let upstream = read(
            &instructions.join("skills/inkscape-mcp").join(name),
            256 * 1024,
        )?;
        let merged = if local == old {
            upstream.clone()
        } else if upstream == old || upstream == local {
            local
        } else {
            // Fixed merge utility, never an artifact-provided executable or script.
            let incoming = base.join(format!("incoming-{index}"));
            let local_input = base.join(format!("local-input-{index}"));
            let baseline_input = base.join(format!("baseline-input-{index}"));
            write(&local_input, &local)?;
            write(&baseline_input, &old)?;
            write(&incoming, &upstream)?;
            let output = Command::new("/usr/bin/diff3")
                .args(["-m", "--"])
                .arg(local_input)
                .arg(baseline_input)
                .arg(&incoming)
                .output()
                .map_err(|e| e.to_string())?;
            if output.stdout.len() > 512 * 1024 {
                return Err("skill merge exceeds byte limit".into());
            }
            if !output.status.success() {
                write(&staged.join(name), &output.stdout)?;
                return Err(format!(
                    "skill merge conflict; active installation preserved. Proposal: {}",
                    staged.display()
                ));
            }
            output.stdout
        };
        write(&staged.join(name), &merged)?;
        write(&staged.join(".inkscape-mcp-upstream").join(name), &upstream)?;
    }
    write(
        &staged.join(".inkscape-mcp-owner"),
        root.to_str().ok_or("invalid installation path")?.as_bytes(),
    )?;
    let after = inventory(&staged, 16 * 1024 * 1024)?;
    Ok(SkillChange {
        destination: path.into(),
        before,
        after,
    })
}
pub fn activate(
    root: &Path,
    before: Selector,
    after: Selector,
    settings: &Settings,
    restore: Option<&Journal>,
) -> Result<bool> {
    if before == after {
        return Ok(false);
    }
    probe(root, &after.current, settings)?;
    let id = uuid::Uuid::new_v4().to_string();
    let base = transaction_path(root, &id)?;
    mkdir(&base)?;
    let mut skills = Vec::new();
    if before.current.instructions.content_id != after.current.instructions.content_id
        || restore.is_some()
    {
        for (i, path) in settings.skills.iter().enumerate() {
            if let Some(restore) = restore {
                let old = restore
                    .skills
                    .iter()
                    .position(|s| &s.destination == path)
                    .ok_or("missing rollback skill record")?;
                let entry = &restore.skills[old];
                let current = inventory(path, 16 * 1024 * 1024)?;
                if current != entry.after {
                    return Err(format!(
                        "skill edited since update; rollback refuses to overwrite {}",
                        path.display()
                    ));
                }
                let saved = transaction_path(root, &restore.id)?.join(format!("old-{old}"));
                if inventory(&saved, 16 * 1024 * 1024)? != entry.before {
                    return Err("rollback backup changed".into());
                }
                copy_tree(&saved, &base.join(format!("new-{i}")), 16 * 1024 * 1024)?;
                skills.push(SkillChange {
                    destination: path.clone(),
                    before: current,
                    after: entry.before.clone(),
                });
            } else {
                skills.push(prepare_skill(
                    root,
                    &base,
                    i,
                    path,
                    &after.current,
                    settings,
                )?);
            }
        }
    }
    let journal = Journal {
        format: FORMAT,
        id,
        before,
        after,
        skills,
    };
    write_json(&base.join("selector-before.json"), &journal.before)?;
    write_json(&base.join("selector-after.json"), &journal.after)?;
    // Inventory recheck after preparation; no write has occurred in skill discovery yet.
    if selector(root)? != journal.before {
        return Err("installation changed while staging".into());
    }
    for skill in &journal.skills {
        if inventory(&skill.destination, 16 * 1024 * 1024)? != skill.before {
            return Err("skill changed while preparing update".into());
        }
    }
    write_json(&root.join("transaction.json"), &journal)?;
    let result = (|| {
        for (i, skill) in journal.skills.iter().enumerate() {
            move_path(&skill.destination, &base.join(format!("old-{i}")))?;
            move_path(&base.join(format!("new-{i}")), &skill.destination)?;
        }
        fs::rename(base.join("selector-after.json"), root.join("active.json"))
            .map_err(|e| e.to_string())?;
        sync_dir(root)?;
        write_json(
            &root
                .join("history")
                .join(format!("{}.json", record_name(&journal.after.current))),
            &journal,
        )?;
        fs::remove_file(root.join("transaction.json")).map_err(|e| e.to_string())?;
        sync_dir(root)?;
        Ok(true)
    })();
    if result.is_err() {
        recover(root, settings).map_err(|e| format!("update failed; recovery required: {e}"))?;
    }
    result
}
pub fn rollback(root: &Path, settings: &Settings) -> Result<bool> {
    let before = selector(root)?;
    let pair = before
        .previous
        .clone()
        .ok_or("no preceding compatible installation")?;
    let record: Journal = json(
        &root
            .join("history")
            .join(format!("{}.json", record_name(&before.current))),
    )?;
    if record.after.current != before.current || record.before.current != pair {
        return Err("rollback record differs from selected pair".into());
    }
    let after = Selector {
        format: FORMAT,
        current: pair,
        previous: Some(before.current.clone()),
        channel: record.before.channel.clone(),
    };
    // Runtime-only updates have no skill changes to restore.
    activate(
        root,
        before,
        after,
        settings,
        (!record.skills.is_empty()).then_some(&record),
    )
}
