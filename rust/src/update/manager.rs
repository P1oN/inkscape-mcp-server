use super::{
    download::{self, Transport},
    install::*,
    instructions::{self, read},
    manifests::*,
    storage::*,
    transaction,
};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub fn migrate(root: &Path, repo: &Path, channel: &str) -> Result<Value> {
    if !matches!(channel, "stable" | "prerelease") {
        return Err("invalid channel".into());
    }
    if root.starts_with(repo) || repo.starts_with(root) {
        return Err(
            "permanent installation must be separate from the source/ready checkout".into(),
        );
    }
    mkdir(root)?;
    let lock = Lock::acquire(root)?;
    if root.join("active.json").exists() {
        let settings: Settings = super::storage::json(&root.join("settings.json"))?;
        if settings.legacy_skill_owner != repo {
            return Err("installation already belongs to another migration source".into());
        }
        transaction::recover(root, &settings)?;
        let selected = selector(root)?;
        probe(root, &selected.current, &settings)?;
        let launcher = root.join("bin/inkscape-mcp-launcher");
        drop(lock);
        crate::client_management::migrate_bindings(repo, &launcher).map_err(|e| e.to_string())?;
        let mut result = installation_report(root, &selected);
        result["client_reconnect_required"] = json!(true);
        return Ok(result);
    }
    for name in [
        "bin",
        "runtime",
        "instructions",
        "staging",
        "backups",
        "history",
    ] {
        mkdir(&root.join(name))?;
    }
    let (package, settings) = import_settings(repo)?;
    if root.starts_with(&package)
        || package.starts_with(root)
        || std::env::split_paths(std::ffi::OsStr::new(
            &settings.environment["INKSCAPE_MCP_WORKSPACE_ROOTS"],
        ))
        .any(|workspace| root.starts_with(&workspace) || workspace.starts_with(root))
    {
        return Err(
            "installation must not overlap the runtime package or drawing workspace".into(),
        );
    }
    let backup = root.join("backups/legacy-configuration");
    mkdir(&backup)?;
    for name in ["setup.conf", "sentry.conf", "clients.json"] {
        let source = repo.join(".inkscape-mcp-local").join(name);
        if source.exists() {
            write(&backup.join(name), &read(&source, 8192)?)?;
        }
    }

    if !backup.join("clients.json").exists() {
        write_json(&backup.join("clients.json"), &Vec::<String>::new())?;
    }
    let metadata: Value = super::storage::json(&package.join("libexec/inkscape-mcp/package.json"))?;
    if metadata["update_contract"]["text_interface"] != TEXT_INTERFACE
        || metadata["update_contract"]["helper_protocol"] != HELPER_PROTOCOL
    {
        return Err("selected package predates independent updates; build/select a current package with source setup, then migrate".into());
    }
    let inventory = read(&package.join("FILES.json"), 2 * 1024 * 1024)?;
    let runtime = RuntimeManifest {
        format: FORMAT,
        distribution_tag: "local".into(),
        build_id: metadata["build_info"]["build_id"]
            .as_str()
            .ok_or("runtime build identity missing")?
            .into(),
        source_revision: metadata["build_info"]["revision"]
            .as_str()
            .ok_or("runtime revision missing")?
            .into(),
        os: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        minimum_os_major: 15,
        text_interface: TEXT_INTERFACE,
        helper_protocol: HELPER_PROTOCOL,
        launcher_minimum: LAUNCHER_VERSION,
        asset: Asset {
            name: "local-runtime.tar.gz".into(),
            identity: FileIdentity::of(&inventory),
        },
    };
    stage_runtime(root, &package, &runtime)?;
    let files = instructions::default_files();
    let manifest = instructions::manifest("migration", &files);
    let bundle = root.join("staging/initial-instructions");
    mkdir(&bundle)?;
    for (name, bytes) in files {
        let target = bundle.join(name);
        mkdir(target.parent().unwrap())?;
        write(&target, &bytes)?;
    }
    write_json(&bundle.join("manifest.json"), &manifest)?;
    stage_instructions(root, &bundle, &manifest)?;
    let selector = Selector {
        format: FORMAT,
        current: Pair {
            runtime,
            instructions: manifest,
            tag: "local".into(),
        },
        previous: None,
        channel: channel.into(),
    };
    doctor(root, &selector.current, &settings)?;
    probe(root, &selector.current, &settings)?;
    // Permanent binary is independent of the selected runtime. Copy only our own executable.
    let executable = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let target = root.join("bin/inkscape-mcp-launcher");
    write(&target, &read(&executable, 128 * 1024 * 1024)?)?;
    fs::set_permissions(
        &target,
        fs::metadata(&executable)
            .map_err(|e| e.to_string())?
            .permissions(),
    )
    .map_err(|e| e.to_string())?;
    write_json(
        &root.join("bootstrap.json"),
        &bootstrap_identity(&read(&executable, 128 * 1024 * 1024)?),
    )?;
    // CLI entry point is another native copy; no wrapper executing downloaded scripts.
    let entry = root.join("bin/inkscape-mcp");
    write(&entry, &read(&executable, 128 * 1024 * 1024)?)?;
    fs::set_permissions(
        &entry,
        fs::metadata(&executable)
            .map_err(|e| e.to_string())?
            .permissions(),
    )
    .map_err(|e| e.to_string())?;
    let app = package.join("management/Inkscape MCP Manager.app");
    if app.exists() {
        copy_tree(
            &app,
            &root.join("Inkscape MCP Manager.app"),
            16 * 1024 * 1024,
        )?;
    }
    write_json(&root.join("settings.json"), &settings)?;
    write_json(&root.join("active.json"), &selector)?;
    drop(lock);
    crate::client_management::migrate_bindings(repo,&target).map_err(|e|format!("installation staged; client migration failed: {e}; retry binding migration with the client manager"))?;
    let mut result = installation_report(root, &selector);
    result["launcher_path"] = json!(target);
    result["cli_path"] = json!(entry);
    result["client_reconnect_required"] = json!(true);
    result["skill_reload_required"] = json!(false);
    Ok(result)
}
#[derive(Clone, Copy)]
pub enum Component {
    Instructions,
    Runtime,
    Both,
}
fn newer_release(candidate: &str, baseline: &str) -> bool {
    match (
        semver::Version::parse(candidate.trim_start_matches('v')),
        semver::Version::parse(baseline.trim_start_matches('v')),
    ) {
        // A previous component-only update may already have selected this tag.
        // Remaining component identities must still be eligible for completion.
        (Ok(candidate), Ok(baseline)) => candidate >= baseline,
        _ => true,
    }
}
/// Startup discovery has no write lock, recovery or settings dependency on a fresh
/// profile. The Manager supplies its sealed offline component metadata.
pub fn check_bundled(
    root: &Path,
    transport: &impl Transport,
    bundle: &Path,
    channel: &str,
    os_major: u32,
) -> Result<Value> {
    if root.join("active.json").exists() || root.join("active.json").is_symlink() {
        return update(
            root,
            transport,
            Component::Both,
            true,
            Some(channel),
            None,
            os_major,
        );
    }
    if root.join("installation-journal.json").exists() || root.join("transaction.json").exists() {
        return Err("Recover the interrupted installation before checking updates.".into());
    }
    let metadata: Value = super::storage::json(&bundle.join("offline-components.json"))?;
    let runtime = metadata["runtime_build"]
        .as_str()
        .filter(|v| identifier(v))
        .ok_or("bundled runtime identity is missing")?;
    let text = instructions::Instructions::load(&bundle.join("instructions"))?;
    let manifest = download::discover_compatible(transport, channel, None, os_major)?;
    let changed = newer_release(
        &manifest.tag,
        metadata["distribution"].as_str().unwrap_or("development"),
    ) && (manifest.runtime.build_id != runtime
        || manifest.instructions.content_id != text.manifest.content_id);
    if [
        "active.json",
        "installation-journal.json",
        "transaction.json",
    ]
    .iter()
    .any(|name| root.join(name).exists() || root.join(name).is_symlink())
    {
        return Err("installation changed during release discovery; check again".into());
    }
    Ok(
        json!({"baseline":"bundled","update_available":changed,"available":{"distribution":manifest.tag,"runtime_build":manifest.runtime.build_id,"instructions_version":manifest.instructions.version},"client_reconnect_required":false,"skill_reload_required":false}),
    )
}
pub fn update(
    root: &Path,
    transport: &impl Transport,
    component: Component,
    check: bool,
    channel: Option<&str>,
    version: Option<&str>,
    os_major: u32,
) -> Result<Value> {
    update_reviewed(
        root, transport, component, check, channel, version, os_major, None,
    )
}
#[allow(clippy::too_many_arguments)]
pub fn update_reviewed(
    root: &Path,
    transport: &impl Transport,
    component: Component,
    check: bool,
    channel: Option<&str>,
    version: Option<&str>,
    os_major: u32,
    review: Option<&str>,
) -> Result<Value> {
    if [
        "active.json",
        "installation-journal.json",
        "transaction.json",
    ]
    .iter()
    .all(|name| !root.join(name).exists() && !root.join(name).is_symlink())
    {
        return Err("No MCP runtime is installed in the selected folder. Choose Install first, or select an existing MCP installation. The Codex settings folder is not an MCP installation.".into());
    }
    let lock = if check {
        None
    } else {
        Some(Lock::acquire(root)?)
    };
    if !check {
        super::installer::recover_before_launch(root)?;
    }
    let settings: Settings = super::storage::json(&root.join("settings.json"))?;
    settings.validate()?;
    if !check {
        transaction::recover(root, &settings)?;
    }
    let before = selector(root)?;
    let channel = channel.unwrap_or(&before.channel);
    let manifest = download::discover_compatible(transport, channel, version, os_major)?;
    let mut pair = before.current.clone();
    match component {
        Component::Instructions => {
            if pair.runtime.text_interface != manifest.instructions.text_interface {
                return Err(
                    "current runtime cannot load available instructions; request a combined update"
                        .into(),
                );
            }
            pair.instructions = manifest.instructions.clone();
        }
        Component::Runtime => {
            if pair.instructions.text_interface != manifest.runtime.text_interface {
                return Err("current instructions cannot run with available runtime; request a combined update".into());
            }
            pair.runtime = manifest.runtime.clone();
        }
        Component::Both => {
            pair.runtime = manifest.runtime.clone();
            pair.instructions = manifest.instructions.clone();
        }
    }
    if pair.instructions.content_id == before.current.instructions.content_id {
        pair.instructions = before.current.instructions.clone();
    }
    pair.tag = manifest.tag.clone();
    let changed = (version.is_some() || newer_release(&manifest.tag, &before.current.tag))
        && (pair.runtime.build_id != before.current.runtime.build_id
            || pair.instructions.content_id != before.current.instructions.content_id);
    let mut result = installation_report(root, &before);
    result["available"] = json!({"distribution":manifest.tag,"runtime_build":manifest.runtime.build_id,"runtime_revision":manifest.runtime.source_revision,"instructions_version":manifest.instructions.version,"instructions_content":manifest.instructions.content_id});
    result["update_available"] = json!(changed);
    result["client_reconnect_required"] = json!(false);
    result["skill_reload_required"] = json!(false);
    result["plan"] = json!({"runtime":{"from":before.current.runtime.build_id,"to":pair.runtime.build_id,"changes":pair.runtime.build_id != before.current.runtime.build_id},"instructions":{"from":before.current.instructions.version,"to":pair.instructions.version,"changes":pair.instructions.content_id != before.current.instructions.content_id},"channel":channel,"release":manifest.tag});
    let review_id = FileIdentity::of(
            &serde_json::to_vec(&json!({"before":before,"settings":settings,"plan":result["plan"],"target":pair,"manifest":manifest}))
            .map_err(|e| e.to_string())?,
    )
    .sha256;
    result["review_id"] = json!(review_id);
    if review.is_some_and(|expected| expected != review_id) {
        return Err("The reviewed update changed. Check and review again before applying.".into());
    }
    if check {
        if selector(root)? != before
            || serde_json::to_value(super::storage::json::<Settings>(
                &root.join("settings.json"),
            )?)
            .map_err(|e| e.to_string())?
                != serde_json::to_value(&settings).map_err(|e| e.to_string())?
            || root.join("installation-journal.json").exists()
            || root.join("transaction.json").exists()
        {
            return Err("installation changed during release discovery; check again".into());
        }
        return Ok(result);
    }
    let _lock = lock;
    if selector(root)? != before {
        return Err("installation changed during release discovery; retry".into());
    }
    if !changed {
        if channel != before.channel {
            let mut selected = before.clone();
            selected.channel = channel.into();
            write_json(&root.join("active.json"), &selected)?;
            result["channel"] = json!(channel);
        }
        result["changed"] = json!(false);

        return Ok(result);
    }
    let temporary = tempfile::Builder::new()
        .prefix("update-")
        .tempdir_in(root.join("staging"))
        .map_err(|e| e.to_string())?;
    let stage = temporary.path();
    if pair.instructions != before.current.instructions {
        eprintln!(
            "Downloading and validating instruction bundle {}…",
            pair.instructions.version
        );
        let bytes = download::asset(transport, &manifest.tag, &manifest.instruction_asset)?;
        let archive = stage.join("instructions.tar.gz");
        write(&archive, &bytes)?;
        super::archive::extract(&archive, stage, Path::new("inkscape-mcp-instructions"))
            .map_err(|e| e.to_string())?;
        stage_instructions(
            root,
            &stage.join("inkscape-mcp-instructions"),
            &pair.instructions,
        )?;
    }
    if pair.runtime.build_id != before.current.runtime.build_id {
        eprintln!(
            "Downloading and validating runtime {}…",
            pair.runtime.build_id
        );
        let bytes = download::asset(
            transport,
            &pair.runtime.distribution_tag,
            &pair.runtime.asset,
        )?;
        let archive = stage.join("runtime.tar.gz");
        write(&archive, &bytes)?;
        super::archive::extract(&archive, stage, Path::new("inkscape-mcp-macos-arm64"))
            .map_err(|e| e.to_string())?;
        stage_runtime(root, &stage.join("inkscape-mcp-macos-arm64"), &pair.runtime)?;
    }
    eprintln!("Checking candidate STDIO/workspace and preparing skill merges…");
    let after = Selector {
        format: FORMAT,
        current: pair,
        previous: Some(before.current.clone()),
        channel: channel.into(),
    };
    doctor(root, &after.current, &settings)?;
    let activation = transaction::activate(root, before, after.clone(), &settings, None)?;
    let changed = activation.changed;
    result = installation_report(root, &after);
    result["changed"] = json!(changed);
    result["client_reconnect_required"] = json!(changed);
    result["skill_reload_required"] = json!(activation.skills_changed);
    result["skipped_skills"] = json!(activation.skipped_skills);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn release_order_handles_numeric_versions_and_prereleases() {
        assert!(newer_release("v0.1.10", "v0.1.9"));
        assert!(!newer_release("v0.1.7", "v0.1.8"));
        assert!(!newer_release("v1.0.0-beta.1", "v1.0.0"));
        assert!(newer_release("v1.0.0", "v1.0.0-beta.1"));
        assert!(newer_release("v1.0.0", "v1.0.0"));
    }
}
