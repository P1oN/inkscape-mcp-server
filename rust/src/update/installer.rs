//! Fixed native first-install operations. Discovery never executes existing bindings.
//! Prepared changes are identity-bound; a durable journal compensates interrupted writes.
use super::{
    install::*,
    instructions::{self, guarded, read},
    manifests::*,
    storage::*,
    transaction,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub package: PathBuf,
    pub instructions: PathBuf,
    pub workspace: PathBuf,
    pub inkscape: PathBuf,
    pub clients: Vec<String>,
    #[serde(default)]
    pub install_skills: bool,
    #[serde(default)]
    pub legacy_source: Option<PathBuf>,
    pub channel: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Change {
    destination: PathBuf,
    before: Option<FileIdentity>,
    after: FileIdentity,
    mode: u32,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Prepared {
    format: u32,
    id: String,
    changes: Vec<Change>,
    selector: Selector,
}
fn present(path: &Path) -> bool {
    path.exists() || path.is_symlink()
}
fn optional(path: &Path) -> Result<Option<FileIdentity>> {
    if !present(path) {
        return Ok(None);
    }
    guarded(path)?;
    if !path.is_file() {
        return Err("installation destination must be a regular file".into());
    }
    Ok(Some(FileIdentity::of(&read(path, 128 * 1024 * 1024)?)))
}
fn ancestor(path: &Path) -> Result<()> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("installation paths must be absolute without parent traversal".into());
    }
    let existing = path
        .ancestors()
        .find(|p| present(p))
        .ok_or("installation parent missing")?;
    guarded(existing)
}
// Inkscape is a user-selected external prerequisite, not managed writable state.
// macOS /Applications is normally root:admin and group-writable. Keep that precise
// system exception separate from the stricter package/state/workspace guards.
fn external_engine(path: &Path) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    if !path.is_absolute() {
        return Err("Inkscape path must be absolute".into());
    }
    let mut current = PathBuf::new();
    for part in path.components() {
        if matches!(
            part,
            std::path::Component::ParentDir | std::path::Component::CurDir
        ) {
            return Err("unsafe Inkscape path".into());
        }
        current.push(part);
        let meta = fs::symlink_metadata(&current).map_err(|e| e.to_string())?;
        let applications = cfg!(target_os = "macos")
            && current == Path::new("/Applications")
            && meta.uid() == 0
            && meta.gid() == 80
            && meta.is_dir();
        if meta.file_type().is_symlink()
            || (meta.uid() != 0 && meta.uid() != unsafe { libc::geteuid() })
            || meta.mode() & 0o002 != 0
            || meta.mode() & 0o020 != 0 && !applications && meta.mode() & 0o1000 == 0
        {
            return Err("unsafe Inkscape prerequisite path".into());
        }
    }
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.mode() & 0o111 == 0 {
        return Err("choose an executable Inkscape prerequisite".into());
    }
    Ok(())
}
fn bindings() -> Result<Vec<crate::client_management::InstallationBinding>> {
    crate::client_management::installation_bindings().map_err(|e| e.to_string())
}
fn client_available(name: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|p| {
            use std::os::unix::fs::PermissionsExt;
            p.join(name)
                .metadata()
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
    }) || [
        PathBuf::from("/Applications"),
        PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join("Applications"),
    ]
    .iter()
    .any(|p| {
        p.join(if name == "codex" {
            "Codex.app"
        } else {
            "Claude.app"
        })
        .is_dir()
    })
}
/// Safe metadata only: client environment and saved monitoring values never leave Rust.
fn saved_engine(settings: &Settings) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    std::env::split_paths(std::ffi::OsStr::new(settings.environment.get("PATH")?))
        .map(|directory| directory.join("inkscape"))
        .find(|path| {
            fs::metadata(path)
                .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        })
}
fn repair_missing_engine(settings: &mut Settings, chosen: &Path) -> Result<()> {
    if saved_engine(settings).is_none() {
        let parent = chosen.parent().ok_or("missing Inkscape directory")?;
        let paths = std::iter::once(parent.to_path_buf()).chain(
            settings
                .environment
                .get("PATH")
                .into_iter()
                .flat_map(|value| std::env::split_paths(std::ffi::OsStr::new(value))),
        );
        let path = std::env::join_paths(paths)
            .map_err(|_| "Inkscape directory cannot be represented in PATH")?;
        settings.environment.insert(
            "PATH".into(),
            path.into_string()
                .map_err(|_| "invalid Inkscape directory")?,
        );
    }
    settings.validate()
}
pub fn inspect(root: &Path) -> Result<Value> {
    ancestor(root)?;
    let mut candidates = BTreeSet::new();
    let mut clients = Vec::new();
    for crate::client_management::InstallationBinding { name, entry, .. } in bindings()? {
        let (entry, error) = match entry {
            Ok(entry) => (entry, None),
            Err(error) => (None, Some(error)),
        };
        let command = entry
            .as_ref()
            .and_then(|e| e["command"].as_str())
            .map(PathBuf::from);
        let candidate = command.as_ref().and_then(|p| {
            if !p.is_absolute()
                || entry.as_ref().is_some_and(|e| {
                    e["args"].as_array().is_some_and(|a| !a.is_empty())
                        || e["url"].as_str().is_some_and(|s| !s.is_empty())
                })
            {
                return None;
            }
            if p.file_name().is_some_and(|n| n == "run-mcp.sh") {
                let source = p.parent()?;
                if ancestor(source).is_ok()
                    && source.join(".inkscape-mcp-local/setup.conf").is_file()
                {
                    return Some(source.to_owned());
                }
            }
            if p.file_name()
                .is_some_and(|n| n == "inkscape-mcp-launcher" || n == "inkscape-mcp")
                && p.parent()?.file_name().is_some_and(|n| n == "bin")
            {
                let source = p.parent()?.parent()?;
                if ancestor(source).is_ok() && source.join("active.json").is_file() {
                    return Some(source.to_owned());
                }
            }
            None
        });
        if let Some(path) = &candidate {
            candidates.insert(path.clone());
        }
        clients.push(json!({"name": name, "available":client_available(&name), "error":error, "configured": entry.is_some(), "candidate": candidate, "command_exists": command.as_ref().is_some_and(|p| p.is_file()), "foreign_binding": entry.is_some() && candidate.is_none()}));
    }
    let mut result = json!({"state":"not_installed","installation":root,"clients":clients,"candidates":candidates,"choice_required":candidates.len()>1,"recovery_required":present(&root.join("installation-journal.json")) || present(&root.join("transaction.json"))});
    let mut configured_engine = None;
    if present(&root.join("active.json")) {
        match selector(root).and_then(|s| {
            let settings: Settings = super::storage::json(&root.join("settings.json"))?;
            settings.validate()?;
            validate_pair(root, &s.current)?;
            Ok(s)
        }) {
            Ok(selected) => {
                result["state"] = json!("installed");
                result["installed"] = installation_report(root, &selected);
                if result["installed"]["bootstrap_damaged"] == true {
                    result["state"] = json!("damaged");
                }
                let settings: Settings = super::storage::json(&root.join("settings.json"))?;
                configured_engine = saved_engine(&settings);
                result["workspace"] = json!(
                    settings
                        .environment
                        .get("INKSCAPE_MCP_WORKSPACE_ROOTS")
                        .and_then(|roots| std::env::split_paths(std::ffi::OsStr::new(roots)).next())
                        .and_then(|path| path.to_str().map(str::to_owned))
                );
            }
            Err(_) => {
                result["state"] = json!("damaged");
                result["diagnostic"] = json!(
                    "The selected installation is incomplete or incompatible. Retry recovery or select a verified package."
                );
            }
        }
    } else if !candidates.is_empty() {
        result["state"] = json!("legacy_found");
    }
    result["inkscape"] = json!(configured_engine.or_else(|| {
        [
            "/Applications/Inkscape.app/Contents/MacOS/inkscape",
            "/usr/bin/inkscape",
        ]
        .into_iter()
        .find(|p| Path::new(p).is_file())
        .map(PathBuf::from)
    }));
    Ok(result)
}
pub fn inspect_legacy(root: &Path, source: &Path) -> Result<Value> {
    ancestor(source)?;
    let (package, settings) = import_settings(source)?;
    let mut result = inspect(root)?;
    result["state"] = json!("legacy_found");
    result["workspace"] = json!(
        std::env::split_paths(std::ffi::OsStr::new(
            &settings.environment["INKSCAPE_MCP_WORKSPACE_ROOTS"]
        ))
        .next()
        .and_then(|p| p.to_str().map(str::to_owned))
    );
    result["inkscape"] = json!(saved_engine(&settings));
    let metadata: Value =
        super::storage::json(&crate::runtime_layout::library(&package).join("package.json"))?;
    result["legacy_build"] = metadata["build_info"].clone();
    Ok(result)
}
fn skill_owned(root: &Path, skill: &Path, settings: &Settings) -> Result<bool> {
    let marker = skill.join(".inkscape-mcp-owner");
    if !present(&marker) {
        return Ok(false);
    }
    let owner = String::from_utf8(read(&marker, 8192)?).map_err(|e| e.to_string())?;
    Ok(owner.trim() == root.to_string_lossy()
        || owner.trim() == settings.legacy_skill_owner.to_string_lossy())
}
fn allowed(root: &Path, destination: &Path) -> Result<bool> {
    if [
        "settings.json",
        "bootstrap.json",
        "active.json",
        "bin/inkscape-mcp",
        "bin/inkscape-mcp-launcher",
        "backups/legacy-configuration/clients.json",
    ]
    .iter()
    .any(|p| root.join(p) == destination)
    {
        return Ok(true);
    }
    binding_allowed(destination, bindings()?)
}
fn binding_allowed(
    destination: &Path,
    bindings: Vec<crate::client_management::InstallationBinding>,
) -> Result<bool> {
    for crate::client_management::InstallationBinding {
        config,
        skill,
        entry,
        ..
    } in bindings
    {
        if destination == config {
            entry?;
            return Ok(true);
        }
        if [
            "SKILL.md",
            "agents/openai.yaml",
            ".inkscape-mcp-upstream/SKILL.md",
            ".inkscape-mcp-upstream/agents/openai.yaml",
            ".inkscape-mcp-owner",
        ]
        .iter()
        .any(|p| skill.join(p) == destination)
        {
            return Ok(true);
        }
    }
    Ok(false)
}
fn add(
    root: &Path,
    base: &Path,
    changes: &mut Vec<Change>,
    destination: PathBuf,
    bytes: &[u8],
    executable: bool,
) -> Result<()> {
    ancestor(&destination)?;
    if !allowed(root, &destination)?
        || changes.len() >= 32
        || changes.iter().any(|c| c.destination == destination)
    {
        return Err("invalid installation change".into());
    }
    let before = optional(&destination)?;
    let after = FileIdentity::of(bytes);
    if before.as_ref() == Some(&after) {
        return Ok(());
    }
    let index = changes.len();
    if let Some(id) = &before {
        write(
            &base.join(format!("before-{index}")),
            &read(&destination, id.bytes)?,
        )?;
    }
    write(&base.join(format!("after-{index}")), bytes)?;
    use std::os::unix::fs::PermissionsExt;
    let mode = if before.is_some() {
        fs::metadata(&destination)
            .map_err(|e| e.to_string())?
            .permissions()
            .mode()
            & 0o777
    } else if executable {
        0o700
    } else {
        0o600
    };
    changes.push(Change {
        destination,
        before,
        after,
        mode,
    });
    Ok(())
}
fn loaded(root: &Path, journal: &str) -> Result<(Prepared, PathBuf)> {
    let plan: Prepared = super::storage::json(&root.join(journal))?;
    if plan.format != 1 || !identifier(&plan.id) || plan.changes.len() > 32 {
        return Err("invalid installation journal".into());
    }
    let base = root.join("backups").join(&plan.id);
    guarded(&base)?;
    let mut seen = BTreeSet::new();
    for (i, c) in plan.changes.iter().enumerate() {
        if !allowed(root, &c.destination)? || !seen.insert(&c.destination) || c.mode & !0o777 != 0 {
            return Err("invalid installation journal destination".into());
        }
        ancestor(&c.destination)?;
        if optional(&base.join(format!("after-{i}")))? != Some(c.after.clone())
            || c.before.is_some() && optional(&base.join(format!("before-{i}")))? != c.before
        {
            return Err("installation backup identity differs".into());
        }
    }
    Ok((plan, base))
}
/// Preparation may stage immutable files but never changes client bindings or selection.
pub fn prepare(root: &Path, mut request: Request) -> Result<Value> {
    super::progress::stage(
        "prerequisites",
        "Checking prerequisites and preservation guards…",
    );
    ancestor(root)?;
    if !matches!(request.channel.as_str(), "stable" | "prerelease")
        || request.clients.is_empty()
        || request.clients.len() > 2
        || request
            .clients
            .iter()
            .any(|n| !matches!(n.as_str(), "codex" | "claude"))
        || request.clients.iter().collect::<BTreeSet<_>>().len() != request.clients.len()
    {
        return Err("choose one or both supported clients and a valid channel".into());
    }
    for path in [&request.package, &request.instructions, &request.workspace] {
        ancestor(path)?;
        guarded(path)?;
        if root.starts_with(path) || path.starts_with(root) {
            return Err(
                "installation overlaps package, instructions, workspace or Inkscape".into(),
            );
        }
    }
    external_engine(&request.inkscape)?;
    if request.inkscape.starts_with(root) {
        return Err("Inkscape overlaps the installation".into());
    }
    if !request.workspace.is_dir()
        || !request.inkscape.is_file()
        || request.inkscape.file_name().is_none_or(|n| n != "inkscape")
    {
        return Err("choose an existing SVG workspace and Inkscape executable".into());
    }
    mkdir(root)?;
    let _lock = Lock::acquire(root)?;
    recover_locked(root)?;
    for name in [
        "bin",
        "runtime",
        "instructions",
        "staging",
        "backups",
        "history",
        "backups/legacy-configuration",
    ] {
        mkdir(&root.join(name))?;
    }
    super::progress::stage("prepare", "Preparing and verifying bundled files…");
    let offline = tempfile::Builder::new()
        .prefix("offline-")
        .tempdir_in(root.join("staging"))
        .map_err(|e| e.to_string())?;
    if request.package.is_file() {
        let staging = offline.path().canonicalize().map_err(|e| e.to_string())?;
        super::archive::extract(
            &request.package,
            &staging,
            Path::new("inkscape-mcp-macos-arm64"),
        )
        .map_err(|e| e.to_string())?;
        request.package = staging.join("inkscape-mcp-macos-arm64");
    }
    let existing = if present(&root.join("active.json")) {
        Some(selector(root)?)
    } else {
        None
    };
    if existing.is_none()
        && [
            "settings.json",
            "bootstrap.json",
            "bin/inkscape-mcp",
            "bin/inkscape-mcp-launcher",
        ]
        .iter()
        .any(|p| present(&root.join(p)))
    {
        return Err("installation root contains unrecognized files; select another root or recover it first".into());
    }
    let mut settings = if let Some(source) = &request.legacy_source {
        ancestor(source)?;
        if let Some(old) = &existing {
            let installed: Settings = super::storage::json(&root.join("settings.json"))?;
            if installed.legacy_skill_owner != *source {
                return Err("existing installation belongs to another migration source; preserve it or choose another root".into());
            }
            let _ = old;
        }
        import_settings(source)?.1
    } else if existing.is_some() {
        super::storage::json(&root.join("settings.json"))?
    } else {
        Settings {
            format: 1,
            legacy_skill_owner: root.into(),
            environment: BTreeMap::from([
                (
                    "INKSCAPE_MCP_WORKSPACE_ROOTS".into(),
                    request
                        .workspace
                        .to_str()
                        .ok_or("invalid workspace")?
                        .into(),
                ),
                (
                    "PATH".into(),
                    format!(
                        "{}:/usr/bin:/bin",
                        request.inkscape.parent().unwrap().display()
                    ),
                ),
                ("INKSCAPE_MCP_LIVE_ENABLED".into(), "true".into()),
                ("INKSCAPE_MCP_ENGINE_MODE".into(), "per_call".into()),
                ("SENTRY_TRACES_SAMPLE_RATE".into(), "0".into()),
            ]),
            skills: Vec::new(),
        }
    };
    settings.validate()?;
    repair_missing_engine(&mut settings, &request.inkscape)?;
    if std::env::split_paths(std::ffi::OsStr::new(
        &settings.environment["INKSCAPE_MCP_WORKSPACE_ROOTS"],
    ))
    .any(|workspace| root.starts_with(&workspace) || workspace.starts_with(root))
    {
        return Err("installation overlaps the saved drawing workspace".into());
    }
    if let Some(old) = &existing {
        transaction::recover(root, &settings)?;
        validate_pair(root, &old.current)?;
    }
    let metadata: Value = super::storage::json(
        &crate::runtime_layout::library(&request.package).join("package.json"),
    )?;
    if metadata["update_contract"]["text_interface"] != TEXT_INTERFACE
        || metadata["update_contract"]["helper_protocol"] != HELPER_PROTOCOL
    {
        return Err("package does not support the installation contract".into());
    }
    let runtime = RuntimeManifest {
        format: 1,
        distribution_tag: "bundled".into(),
        build_id: metadata["build_info"]["build_id"]
            .as_str()
            .ok_or("missing build")?
            .into(),
        source_revision: metadata["build_info"]["revision"]
            .as_str()
            .ok_or("missing revision")?
            .into(),
        os: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        minimum_os_major: 15,
        text_interface: TEXT_INTERFACE,
        helper_protocol: HELPER_PROTOCOL,
        launcher_minimum: LAUNCHER_VERSION,
        asset: Asset {
            name: "bundled-runtime.tar.gz".into(),
            identity: FileIdentity::of(&read(
                &request.package.join("FILES.json"),
                2 * 1024 * 1024,
            )?),
        },
    };
    stage_runtime(root, &request.package, &runtime)?;
    let bundle = instructions::Instructions::load(&request.instructions)?;
    stage_instructions(root, &request.instructions, &bundle.manifest)?;
    let mut selected = Selector {
        format: 1,
        current: Pair {
            runtime,
            instructions: bundle.manifest,
            tag: "bundled".into(),
        },
        previous: existing.as_ref().map(|s| s.current.clone()),
        channel: existing
            .as_ref()
            .map(|s| s.channel.clone())
            .unwrap_or(request.channel),
    };
    if let Some(old) = &existing
        && old.current == selected.current
    {
        selected = old.clone();
    }
    // Authorize the retained rollback runtime for this helper before replacing it.
    if let Some(previous) = &selected.previous {
        stage_runtime(root, &runtime_path(root, previous)?, &previous.runtime)?;
    }
    super::progress::stage("probe", "Checking the installed MCP connection…");
    doctor(root, &selected.current, &settings)?;
    probe(root, &selected.current, &settings)?;
    let id = uuid::Uuid::new_v4().to_string();
    let base = root.join("backups").join(&id);
    mkdir(&base)?;
    let mut changes = Vec::new();
    let mut skipped_skills = Vec::new();
    let launcher = root.join("bin/inkscape-mcp-launcher");
    let current = std::env::current_exe().map_err(|e| e.to_string())?;
    // Always our bundled executable, never the discovered legacy launcher.
    let bytes = read(&current, 128 * 1024 * 1024)?;
    let runtime_launcher = crate::runtime_layout::binary(
        &crate::runtime_layout::library(&runtime_path(root, &selected.current)?),
        "inkscape-mcp-launcher",
    );
    if FileIdentity::of(&bytes) != FileIdentity::of(&read(&runtime_launcher, 128 * 1024 * 1024)?) {
        // An explicit bootstrap upgrade may retain an older immutable signed runtime.
        // Ad-hoc checkpoints require byte identity; independent bootstrap code must
        // have the same verified Developer ID team as the compatible runtime.
        let actor_team =
            crate::client_management::signature_team(&current).map_err(|e| e.to_string())?;
        let runtime_team = crate::client_management::signature_team(&runtime_launcher)
            .map_err(|e| e.to_string())?;
        if actor_team.is_none() || actor_team != runtime_team {
            return Err("bundled installer differs from the verified runtime launcher and has no matching Developer ID team".into());
        }
    }
    add(root, &base, &mut changes, launcher.clone(), &bytes, true)?;
    add(
        root,
        &base,
        &mut changes,
        root.join("bin/inkscape-mcp"),
        &bytes,
        true,
    )?;
    add(
        root,
        &base,
        &mut changes,
        root.join("bootstrap.json"),
        &serde_json::to_vec_pretty(&bootstrap_identity(&bytes)).map_err(|e| e.to_string())?,
        false,
    )?;
    for crate::client_management::InstallationBinding {
        name,
        config,
        entry,
        skill,
    } in bindings()?
    {
        if !request.clients.contains(&name) {
            continue;
        }
        let entry = entry?;
        if !client_available(&name) {
            return Err(format!(
                "{name} client is missing; install it before connecting"
            ));
        }
        if entry.as_ref().is_some_and(|e| {
            e["command"].as_str() != launcher.to_str()
                && request
                    .legacy_source
                    .as_ref()
                    .is_none_or(|s| e["command"].as_str() != s.join("run-mcp.sh").to_str())
                || e["args"].as_array().is_some_and(|a| !a.is_empty())
                || e["url"].as_str().is_some_and(|s| !s.is_empty())
        }) {
            return Err("client binding belongs to another installation; explicitly select its verified source or preserve it".into());
        }
        let before = if present(&config) {
            String::from_utf8(read(&config, 4 * 1024 * 1024)?).map_err(|e| e.to_string())?
        } else {
            String::new()
        };
        let after = crate::client_management::installation_registration(&name, &before, &launcher)
            .map_err(|e| e.to_string())?;
        add(root, &base, &mut changes, config, after.as_bytes(), false)?;
        let owned = skill_owned(root, &skill, &settings)?;
        if present(&skill) && !owned {
            settings.skills.retain(|p| p != &skill);
            skipped_skills.push(skill);
        } else if owned {
            let merged = transaction::prepare_skill(
                root,
                &base,
                changes.len(),
                &skill,
                &selected.current,
                &settings,
            )?;
            let staged = base.join(format!("new-{}", changes.len()));
            for file in [
                "SKILL.md",
                "agents/openai.yaml",
                ".inkscape-mcp-upstream/SKILL.md",
                ".inkscape-mcp-upstream/agents/openai.yaml",
                ".inkscape-mcp-owner",
            ] {
                add(
                    root,
                    &base,
                    &mut changes,
                    skill.join(file),
                    &read(&staged.join(file), 256 * 1024)?,
                    false,
                )?;
            }
            let _ = merged;
            if !settings.skills.contains(&skill) {
                settings.skills.push(skill);
            }
        } else if request.install_skills {
            mkdir(skill.parent().ok_or("missing skill parent")?)?;
            for file in ["SKILL.md", "agents/openai.yaml"] {
                let bytes = read(
                    &instruction_path(root, &selected.current)?
                        .join("skills/inkscape-mcp")
                        .join(file),
                    256 * 1024,
                )?;
                add(root, &base, &mut changes, skill.join(file), &bytes, false)?;
                add(
                    root,
                    &base,
                    &mut changes,
                    skill.join(".inkscape-mcp-upstream").join(file),
                    &bytes,
                    false,
                )?;
            }
            add(
                root,
                &base,
                &mut changes,
                skill.join(".inkscape-mcp-owner"),
                root.to_str().ok_or("invalid root")?.as_bytes(),
                false,
            )?;
            if !settings.skills.contains(&skill) {
                settings.skills.push(skill);
            }
        }
    }
    settings.validate()?;
    let record = root.join("backups/legacy-configuration/clients.json");
    let mut recorded: Vec<String> = if present(&record) {
        super::storage::json(&record)?
    } else {
        Vec::new()
    };
    for name in &request.clients {
        if !recorded.contains(name) {
            recorded.push(name.clone());
        }
    }
    add(
        root,
        &base,
        &mut changes,
        root.join("settings.json"),
        &serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?,
        false,
    )?;
    add(
        root,
        &base,
        &mut changes,
        record,
        &serde_json::to_vec(&recorded).unwrap(),
        false,
    )?;
    // Selection is the last publication boundary; client bindings may point here while
    // the journal exists, so launcher startup recovers before serving MCP.
    add(
        root,
        &base,
        &mut changes,
        root.join("active.json"),
        &serde_json::to_vec_pretty(&selected).unwrap(),
        false,
    )?;
    let plan = Prepared {
        format: 1,
        id: id.clone(),
        changes,
        selector: selected,
    };
    write_json(&root.join("installation-prepared.json"), &plan)?;
    Ok(
        json!({"state":"prepared","skipped_skills":skipped_skills,"preparation_id":id,"changes":plan.changes.iter().map(|c|&c.destination).collect::<Vec<_>>(),"installed":report(&plan.selector)}),
    )
}
fn apply_file(path: &Path, bytes: &[u8], mode: u32) -> Result<()> {
    mkdir(path.parent().ok_or("missing destination parent")?)?;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    if present(path) {
        guarded(path)?;
    }
    let mut staged =
        tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|e| e.to_string())?;
    staged.write_all(bytes).map_err(|e| e.to_string())?;
    staged
        .as_file()
        .set_permissions(fs::Permissions::from_mode(mode))
        .map_err(|e| e.to_string())?;
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    staged.persist(path).map_err(|e| e.to_string())?;
    sync_dir(path.parent().unwrap())
}
fn recover_locked(root: &Path) -> Result<bool> {
    if !present(&root.join("installation-journal.json")) {
        return Ok(false);
    }
    let (plan, base) = loaded(root, "installation-journal.json")?;
    // Preflight all locations: do not erase changes made after an interruption.
    for c in &plan.changes {
        let actual = optional(&c.destination)?;
        if actual != c.before && actual != Some(c.after.clone()) {
            return Err("installation destination changed after interruption; preserved for manual recovery".into());
        }
    }
    for (i, c) in plan.changes.iter().enumerate().rev() {
        if optional(&c.destination)? == c.before {
            continue;
        }
        if let Some(id) = &c.before {
            apply_file(
                &c.destination,
                &read(&base.join(format!("before-{i}")), id.bytes)?,
                c.mode,
            )?;
        } else {
            fs::remove_file(&c.destination).map_err(|e| e.to_string())?;
            sync_dir(c.destination.parent().unwrap())?;
        }
    }
    fs::remove_file(root.join("installation-journal.json")).map_err(|e| e.to_string())?;
    sync_dir(root)?;
    Ok(true)
}
pub fn recover(root: &Path) -> Result<Value> {
    let _lock = Lock::acquire(root)?;
    let mut changed = recover_locked(root)?;
    if present(&root.join("transaction.json")) {
        let settings: Settings = super::storage::json(&root.join("settings.json"))?;
        settings.validate()?;
        changed |= transaction::recover(root, &settings)?;
    }
    Ok(json!({"state":"recovered","changed":changed}))
}
/// Called under the launch/update lock before reading the active selector.
pub fn recover_before_launch(root: &Path) -> Result<bool> {
    recover_locked(root)
}
/// Restore the last wizard upgrade only while all changed destinations still match.
/// Later client/skill edits are preserved and require manual reconciliation.
pub fn rollback_if_selected(root: &Path) -> Result<Option<Value>> {
    if !present(&root.join("installation-last.json")) {
        return Ok(None);
    }
    let (plan, _) = loaded(root, "installation-last.json")?;
    if selector(root)? != plan.selector || plan.selector.previous.is_none() {
        return Ok(None);
    }
    for change in &plan.changes {
        if optional(&change.destination)? != Some(change.after.clone()) {
            return Err(
                "installation changed since wizard upgrade; rollback preserves later edits".into(),
            );
        }
    }
    super::progress::stage(
        "activate",
        "Updating client configuration, skills and installation settings…",
    );
    write_json(&root.join("installation-journal.json"), &plan)?;
    recover_locked(root)?;
    let mut result = installation_report(root, &selector(root)?);
    result["changed"] = json!(true);
    result["client_reconnect_required"] = json!(true);
    result["skill_reload_required"] = json!(true);
    Ok(Some(result))
}
pub fn activate(root: &Path, id: &str) -> Result<Value> {
    let _lock = Lock::acquire(root)?;
    recover_locked(root)?;
    let (plan, base) = loaded(root, "installation-prepared.json")?;
    if plan.id != id {
        return Err("prepared installation identity differs; prepare again".into());
    }
    for c in &plan.changes {
        if optional(&c.destination)? != c.before {
            return Err("installation changed after preparation; prepare again".into());
        }
    }
    let settings: Settings = if let Some((i, c)) = plan
        .changes
        .iter()
        .enumerate()
        .find(|(_, c)| c.destination == root.join("settings.json"))
    {
        serde_json::from_slice(&read(&base.join(format!("after-{i}")), c.after.bytes)?)
            .map_err(|e| e.to_string())?
    } else {
        super::storage::json(&root.join("settings.json"))?
    };
    super::progress::stage("probe", "Rechecking the prepared MCP connection…");
    doctor(root, &plan.selector.current, &settings)?;
    probe(root, &plan.selector.current, &settings)?;
    if plan.changes.is_empty() {
        let mut result = report(&plan.selector);
        result["changed"] = json!(false);
        result["state"] = json!("completed");
        result["client_reconnect_required"] = json!(false);
        result["skill_reload_required"] = json!(false);
        return Ok(result);
    }
    write_json(&root.join("installation-journal.json"), &plan)?;
    let result = (|| {
        for (i, c) in plan.changes.iter().enumerate() {
            if optional(&c.destination)? != c.before {
                return Err(
                    "installation destination changed during activation; recovery required".into(),
                );
            }
            apply_file(
                &c.destination,
                &read(&base.join(format!("after-{i}")), c.after.bytes)?,
                c.mode,
            )?;
        }
        write_json(&root.join("installation-last.json"), &plan)?;
        fs::remove_file(root.join("installation-journal.json")).map_err(|e| e.to_string())?;
        sync_dir(root)?;
        super::progress::stage(
            "complete",
            "Installation completed. Restart connected clients to load it.",
        );
        let mut result = report(&plan.selector);
        result["changed"] = json!(!plan.changes.is_empty());
        result["state"] = json!("completed");
        result["client_reconnect_required"] = json!(true);
        result["skill_reload_required"] = json!(true);
        Ok(result)
    })();
    if result.is_err() {
        recover_locked(root)?;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::instructions::Instructions;
    #[test]
    fn recovery_checks_only_the_target_clients_configuration() {
        let make = || {
            vec![
                crate::client_management::InstallationBinding {
                    name: "claude".into(),
                    config: "/home/.claude.json".into(),
                    entry: Err("unsafe configuration".into()),
                    skill: "/home/skill".into(),
                },
                crate::client_management::InstallationBinding {
                    name: "codex".into(),
                    config: "/home/.codex/config.toml".into(),
                    entry: Ok(None),
                    skill: "/home/codex-skill".into(),
                },
            ]
        };
        assert!(binding_allowed(Path::new("/home/.codex/config.toml"), make()).unwrap());
        assert!(binding_allowed(Path::new("/home/.claude.json"), make()).is_err());
        assert!(binding_allowed(Path::new("/home/skill/SKILL.md"), make()).unwrap());
    }
    #[test]
    fn skill_ownership_preserves_unmanaged_and_foreign_skills() {
        let (_temp, root, plan, _) = journal();
        let skill = root.join("custom-skill");
        mkdir(&skill).unwrap();
        write(&skill.join("SKILL.md"), b"user content").unwrap();
        let settings = Settings {
            format: 1,
            legacy_skill_owner: root.join("legacy"),
            environment: BTreeMap::new(),
            skills: Vec::new(),
        };
        assert!(!skill_owned(&root, &skill, &settings).unwrap());
        let marker = skill.join(".inkscape-mcp-owner");
        for (owner, expected) in [
            (root.join("foreign"), false),
            (root.clone(), true),
            (settings.legacy_skill_owner.clone(), true),
        ] {
            write(&marker, owner.to_str().unwrap().as_bytes()).unwrap();
            assert_eq!(skill_owned(&root, &skill, &settings).unwrap(), expected);
        }
        fs::remove_file(&marker).unwrap();
        std::os::unix::fs::symlink("missing", &marker).unwrap();
        assert!(skill_owned(&root, &skill, &settings).is_err());
        assert_eq!(read(&skill.join("SKILL.md"), 128).unwrap(), b"user content");
        assert!(!plan.changes.is_empty());
    }
    #[test]
    fn custom_engine_discovery_and_missing_engine_repair_preserve_other_settings() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let old = base.join("custom engine");
        let new = base.join("moved engine");
        for directory in [&old, &new] {
            mkdir(directory).unwrap();
            fs::write(directory.join("inkscape"), b"presence only; never execute").unwrap();
            fs::set_permissions(
                directory.join("inkscape"),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
        }
        let mut settings = Settings {
            format: 1,
            legacy_skill_owner: base.clone(),
            environment: BTreeMap::from([
                ("PATH".into(), old.to_str().unwrap().into()),
                (
                    "INKSCAPE_MCP_WORKSPACE_ROOTS".into(),
                    base.to_str().unwrap().into(),
                ),
                ("INKSCAPE_MCP_LIVE_ENABLED".into(), "false".into()),
                ("INKSCAPE_MCP_ENGINE_MODE".into(), "shell".into()),
                (
                    "SENTRY_DSN".into(),
                    "https://fixture@sentry.invalid/1".into(),
                ),
            ]),
            skills: Vec::new(),
        };
        let before = serde_json::to_value(&settings).unwrap();
        assert_eq!(saved_engine(&settings), Some(old.join("inkscape")));
        repair_missing_engine(&mut settings, &new.join("inkscape")).unwrap();
        assert_eq!(serde_json::to_value(&settings).unwrap(), before);
        fs::remove_file(old.join("inkscape")).unwrap();
        assert!(saved_engine(&settings).is_none());
        repair_missing_engine(&mut settings, &new.join("inkscape")).unwrap();
        assert_eq!(saved_engine(&settings), Some(new.join("inkscape")));
        assert_eq!(
            std::env::split_paths(std::ffi::OsStr::new(&settings.environment["PATH"]))
                .collect::<Vec<_>>(),
            vec![new.clone(), old]
        );
        let mut repaired = serde_json::to_value(&settings).unwrap();
        repaired["environment"]["PATH"] = before["environment"]["PATH"].clone();
        assert_eq!(repaired, before);
        fs::set_permissions(new.join("inkscape"), fs::Permissions::from_mode(0o600)).unwrap();
        assert!(saved_engine(&settings).is_none());
        assert!(
            external_engine(&new.join("inkscape"))
                .unwrap_err()
                .contains("executable")
        );
        let missing = serde_json::to_value(&settings).unwrap();
        assert!(
            repair_missing_engine(&mut settings, &base.join("bad:directory/inkscape")).is_err()
        );
        assert_eq!(serde_json::to_value(&settings).unwrap(), missing);
    }
    fn journal() -> (tempfile::TempDir, PathBuf, Prepared, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap().join("install");
        mkdir(&root.join("backups/checkpoint")).unwrap();
        let base = root.join("backups/checkpoint");
        let mut changes = Vec::new();
        write(&root.join("settings.json"), b"old settings").unwrap();
        add(
            &root,
            &base,
            &mut changes,
            root.join("settings.json"),
            b"new settings",
            false,
        )
        .unwrap();
        add(
            &root,
            &base,
            &mut changes,
            root.join("active.json"),
            b"new selector",
            false,
        )
        .unwrap();
        let runtime = RuntimeManifest {
            format: 1,
            distribution_tag: "test".into(),
            build_id: "test".into(),
            source_revision: "unknown".into(),
            os: "macos".into(),
            architecture: "aarch64".into(),
            minimum_os_major: 15,
            text_interface: 1,
            helper_protocol: 5,
            launcher_minimum: 1,
            asset: Asset {
                name: "runtime.tar.gz".into(),
                identity: FileIdentity::of(b"test"),
            },
        };
        let selector = Selector {
            format: 1,
            current: Pair {
                runtime,
                instructions: instructions::manifest("test", &instructions::default_files()),
                tag: "test".into(),
            },
            previous: None,
            channel: "stable".into(),
        };
        let plan = Prepared {
            format: 1,
            id: "checkpoint".into(),
            changes,
            selector,
        };
        write_json(&root.join("installation-journal.json"), &plan).unwrap();
        (temp, root, plan, base)
    }
    #[test]
    fn identical_instruction_text_with_new_label_preserves_both_bundles() {
        let (_temp, root, plan, _base) = journal();
        mkdir(&root.join("instructions")).unwrap();
        mkdir(&root.join("staging")).unwrap();
        let source = root.join("source-text");
        mkdir(&source).unwrap();
        let files = instructions::default_files();
        for (name, bytes) in &files {
            let path = source.join(name);
            mkdir(path.parent().unwrap()).unwrap();
            write(&path, bytes).unwrap();
        }
        let first = instructions::manifest("first", &files);
        let second = instructions::manifest("second", &files);
        write_json(&source.join("manifest.json"), &first).unwrap();
        stage_instructions(&root, &source, &first).unwrap();
        let mut pair = plan.selector.current;
        pair.instructions = first.clone();
        let first_path = instruction_path(&root, &pair).unwrap();
        write_json(&source.join("manifest.json"), &second).unwrap();
        stage_instructions(&root, &source, &second).unwrap();
        pair.instructions = second.clone();
        let second_path = instruction_path(&root, &pair).unwrap();
        assert_ne!(first_path, second_path);
        assert_eq!(Instructions::load(&first_path).unwrap().manifest, first);
        assert_eq!(Instructions::load(&second_path).unwrap().manifest, second);
    }
    #[test]
    fn discovery_binds_bootstrap_identity_to_both_files_without_execution() {
        let (_temp, root, plan, _base) = journal();
        mkdir(&root.join("bin")).unwrap();
        let bytes = b"synthetic launcher presence; never execute";
        for name in ["inkscape-mcp", "inkscape-mcp-launcher"] {
            write(&root.join("bin").join(name), bytes).unwrap();
        }
        let old = installation_report(&root, &plan.selector);
        assert!(old["launcher_build"].is_null());
        assert!(old.get("bootstrap_damaged").is_none());
        write_json(&root.join("bootstrap.json"), &bootstrap_identity(bytes)).unwrap();
        let verified = installation_report(&root, &plan.selector);
        assert_eq!(verified["launcher_build"], env!("INKSCAPE_MCP_BUILD_ID"));
        write(&root.join("bin/inkscape-mcp"), b"changed launcher").unwrap();
        let damaged = installation_report(&root, &plan.selector);
        assert_eq!(damaged["bootstrap_damaged"], true);
        assert!(damaged["launcher_build"].is_null());
    }
    #[test]
    fn every_publication_boundary_recovers_idempotently() {
        for boundary in 0..=2 {
            let (_temp, root, plan, base) = journal();
            for (i, c) in plan.changes.iter().enumerate().take(boundary) {
                apply_file(
                    &c.destination,
                    &read(&base.join(format!("after-{i}")), c.after.bytes).unwrap(),
                    c.mode,
                )
                .unwrap();
            }
            assert!(recover(&root).unwrap()["changed"].as_bool().unwrap());
            assert!(!recover(&root).unwrap()["changed"].as_bool().unwrap());
            assert_eq!(
                read(&root.join("settings.json"), 100).unwrap(),
                b"old settings"
            );
            assert!(!root.join("active.json").exists());
        }
    }
    #[test]
    fn edited_destination_bad_backup_and_forged_journal_refuse_before_restoration() {
        for scenario in ["edit", "backup", "destination", "duplicate", "symlink"] {
            let (_temp, root, mut plan, base) = journal();
            apply_file(&plan.changes[0].destination, b"new settings", 0o600).unwrap();
            match scenario {
                "edit" => write(&root.join("active.json"), b"later user edit").unwrap(),
                "backup" => write(&base.join("before-0"), b"damaged backup").unwrap(),
                "destination" => {
                    plan.changes[0].destination = root.parent().unwrap().join("foreign");
                    write_json(&root.join("installation-journal.json"), &plan).unwrap();
                }
                "duplicate" => {
                    plan.changes[1].destination = plan.changes[0].destination.clone();
                    write_json(&root.join("installation-journal.json"), &plan).unwrap();
                }
                "symlink" => {
                    std::os::unix::fs::symlink(root.join("settings.json"), root.join("active.json"))
                        .unwrap()
                }
                _ => unreachable!(),
            }
            assert!(recover(&root).is_err(), "{scenario}");
            assert_eq!(
                read(&root.join("settings.json"), 100).unwrap(),
                b"new settings"
            );
            assert!(root.join("installation-journal.json").exists());
        }
    }
    #[test]
    fn path_overlap_and_unknown_request_fields_refuse() {
        assert!(serde_json::from_value::<Request>(json!({"shell":"anything"})).is_err());
        assert!(ancestor(Path::new("/tmp/../escape")).is_err());
        assert!(ancestor(Path::new("relative")).is_err());
    }
}
