//! Explicit per-user Manager relocation. Never replaces a running destination app.
use super::{
    installer, instructions::guarded, manager_release as release, manifests::*, storage::*,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[cfg(target_os = "macos")]
use std::time::{Duration, Instant};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
const APP: &str = "Inkscape MCP Manager.app";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    format: u32,
    id: String,
    source: PathBuf,
    stage: PathBuf,
    target: PathBuf,
    backup: PathBuf,
    before: Option<BTreeMap<String, FileIdentity>>,
    after: BTreeMap<String, FileIdentity>,
    #[serde(default)]
    online: Option<Online>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Online {
    manifest: release::Manifest,
    channel: String,
    profile: BTreeMap<String, PathBuf>,
    observations: BTreeMap<PathBuf, Option<FileIdentity>>,
    bootstrap: Vec<installer::Change>,
    helper: FileIdentity,
    baseline_healthy: bool,
}
fn parent() -> Result<PathBuf> {
    Ok(PathBuf::from(std::env::var_os("HOME").ok_or("HOME missing")?).join("Applications"))
}
fn present(path: &Path) -> bool {
    path.exists() || path.is_symlink()
}
fn installed_location(source: &Path, per_user_parent: &Path) -> bool {
    source == per_user_parent.join(APP) || source == Path::new("/Applications").join(APP)
}
pub(super) fn destination_parent(path: &Path) -> Result<()> {
    if path != Path::new("/Applications") {
        return guarded(path);
    }
    use std::os::unix::{ffi::OsStrExt, fs::MetadataExt};
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !cfg!(target_os = "macos")
        || !meta.is_dir()
        || meta.uid() != 0
        || meta.gid() != 80
        || meta.mode() & 0o002 != 0
    {
        return Err("unsafe system Applications directory".into());
    }
    let name = std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(|e| e.to_string())?;
    if unsafe { libc::access(name.as_ptr(), libc::W_OK | libc::X_OK) } != 0 {
        return Err("The Manager in /Applications cannot be replaced by this user. Ask its administrator to update it; no competing installation was created.".into());
    }
    Ok(())
}
fn base(root: &Path, plan: &Plan) -> PathBuf {
    root.join("backups").join(&plan.id)
}
fn profile() -> Result<BTreeMap<String, PathBuf>> {
    let mut values = BTreeMap::new();
    for name in ["HOME", "CODEX_HOME", "CFFIXED_USER_HOME"] {
        if let Some(value) = std::env::var_os(name) {
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err("Manager profile path must be absolute".into());
            }
            values.insert(name.into(), path);
        }
    }
    Ok(values)
}
fn observations(root: &Path) -> Result<BTreeMap<PathBuf, Option<FileIdentity>>> {
    ["active.json", "settings.json", "manager-last.json"]
        .into_iter()
        .map(|name| {
            let path = root.join(name);
            Ok((path.clone(), installer::optional(&path)?))
        })
        .collect()
}
fn installed_bootstrap_identity(root: &Path) -> Option<super::install::BootstrapIdentity> {
    let saved: super::install::BootstrapIdentity = json(&root.join("bootstrap.json")).ok()?;
    if saved.format != 1 || !identifier(&saved.build_id) || !identifier(&saved.source_revision) {
        return None;
    }
    for name in ["bin/inkscape-mcp", "bin/inkscape-mcp-launcher"] {
        if installer::optional(&root.join(name)).ok()?? != saved.executable {
            return None;
        }
    }
    Some(saved)
}
fn no_other_transaction(root: &Path) -> Result<()> {
    for name in [
        "installation-journal.json",
        "transaction.json",
        "manager-journal.json",
    ] {
        if root.join(name).exists() || root.join(name).is_symlink() {
            return Err(format!("Recover {name} before preparing a Manager update."));
        }
    }
    Ok(())
}
fn verify_last(root: &Path) -> Result<()> {
    if present(&root.join("manager-last.json")) {
        load(root, "manager-last.json")?;
    }
    Ok(())
}
fn commit_manager_journal(root: &Path) -> Result<()> {
    verify_last(root)?;
    // A single rename is the commit decision. Never leave a new success record
    // beside a still-recoverable journal after a process/power interruption.
    fs::rename(
        root.join("manager-journal.json"),
        root.join("manager-last.json"),
    )
    .map_err(|e| e.to_string())?;
    sync_dir(root)
}
fn acknowledged(root: &Path, review: &str) -> bool {
    json::<Plan>(&root.join("manager-last.json"))
        .ok()
        .and_then(|p| serde_json::to_vec(&p).ok())
        .is_some_and(|bytes| FileIdentity::of(&bytes).sha256 == review)
}
fn actor_supports(pair: &super::install::Pair) -> bool {
    pair.runtime.format == FORMAT
        && pair.instructions.format == FORMAT
        && pair.runtime.text_interface == TEXT_INTERFACE
        && pair.instructions.text_interface == TEXT_INTERFACE
        && pair.runtime.helper_protocol == HELPER_PROTOCOL
        && pair.runtime.launcher_minimum <= LAUNCHER_VERSION
        && pair.instructions.launcher_minimum <= LAUNCHER_VERSION
}
fn probe_with_current_actor(root: &Path) -> Result<()> {
    if let Ok(selected) = super::install::selector(root)
        && actor_supports(&selected.current)
    {
        runtime_health(root)?;
    }
    // An installation already requiring the target bootstrap is probed by that
    // verified new helper during finish_online, before its mandatory acknowledgment.
    Ok(())
}
fn runtime_health(root: &Path) -> Result<()> {
    let selected = super::install::selector(root)?;
    let settings: super::install::Settings = json(&root.join("settings.json"))?;
    settings.validate()?;
    if !actor_supports(&selected.current) {
        return Err("selected runtime requires a different bootstrap".into());
    }
    #[cfg(target_os = "macos")]
    if let Some(team) = crate::client_management::signature_team(
        &std::env::current_exe().map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?
    {
        // A Manager-only review must not execute an unsigned/foreign legacy
        // runtime discovered in a damaged root. Preserve it as unverified.
        let executable = crate::runtime_layout::binary(
            &crate::runtime_layout::library(&super::install::runtime_path(
                root,
                &selected.current,
            )?),
            "inkscape-mcp",
        );
        if crate::client_management::signature_team(&executable)
            .map_err(|e| e.to_string())?
            .as_deref()
            != Some(team.as_str())
        {
            return Err("installed runtime publisher is unverified; health probe skipped".into());
        }
    }
    super::install::probe(root, &selected.current, &settings)
}
fn sync_stage(app: &Path) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut pending = vec![app.to_owned()];
    let mut dirs = vec![];
    let mut visited = 0;
    while let Some(path) = pending.pop() {
        visited += 1;
        if visited > 50000 {
            return Err("Manager stage exceeds sync entry limit".into());
        }
        super::instructions::guarded_read(&path)?;
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if metadata.is_dir() {
            dirs.push(path.clone());
            for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
                pending.push(entry.map_err(|e| e.to_string())?.path());
            }
        } else if metadata.is_file() {
            fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW)
                .open(&path)
                .and_then(|f| f.sync_all())
                .map_err(|e| e.to_string())?;
        } else {
            return Err("Manager stage contains a link or special file".into());
        }
    }
    for dir in dirs.into_iter().rev() {
        sync_dir(&dir)?;
    }
    Ok(())
}
fn preserves_selection(root: &Path, manifest: &release::Manifest) -> Result<()> {
    // Missing/damaged metadata is never repaired or replaced by this check.
    for name in ["active.json", "settings.json"] {
        if let Ok(metadata) = json::<Value>(&root.join(name))
            && let Some(format) = metadata["format"].as_u64()
            && format != u64::from(manifest.component_format)
        {
            return Err("The target Manager/bootstrap cannot preserve the saved component/settings/selector format. No components were changed.".into());
        }
    }
    // Deserialize the retained contract without imposing the *old* coordinator's
    // FORMAT. Compatibility must be decided by the verified target's format.
    if let Ok(selected) = json::<super::install::Selector>(&root.join("active.json")) {
        manifest.preserves_pair(&selected.current)?;
        if let Some(previous) = &selected.previous {
            manifest.preserves_pair(previous)?;
        }
    }
    Ok(())
}

/// Explicit staging only. Downloads do not change selected components or profiles.
pub fn prepare_online(
    root: &Path,
    transport: &impl super::download::Transport,
    channel: &str,
    major: u32,
) -> Result<Value> {
    let source = current_app()?;
    if !installed_location(&source, &parent()?) {
        return Err("Install this Manager application before updating it online.".into());
    }
    verify(&source)?;
    let manifest = release::discover_newer(transport, channel, major, &release::info(&source)?)?
        .ok_or("No Manager update on this channel.")?;
    if !release::eligible(&manifest, &release::info(&source)?) {
        return Err("No newer Manager build on this channel.".into());
    }
    if crate::client_management::signature_team(&source)
        .map_err(|e| e.to_string())?
        .as_deref()
        != Some(release::PUBLISHER)
    {
        return Err("Online Manager updates require the trusted Developer ID distribution.".into());
    }
    prepare_online_verified(
        root,
        &source,
        manifest,
        channel,
        transport,
        &std::env::current_exe().map_err(|e| e.to_string())?,
    )
}
fn prepare_online_verified(
    root: &Path,
    source: &Path,
    manifest: release::Manifest,
    channel: &str,
    transport: &impl super::download::Transport,
    actor: &Path,
) -> Result<Value> {
    let parent = source.parent().ok_or("Manager parent missing")?;
    destination_parent(parent)?;
    mkdir(root)?;
    let _lock = Lock::acquire(root)?;
    let _app_lock = Lock::manager(parent)?;
    no_other_transaction(root)?;
    verify_last(root)?;
    preserves_selection(root, &manifest)?;
    let observed = observations(root)?;
    // A broken/incomplete installation must not prevent updating Manager. Review
    // distinguishes its preserved state from a verified working baseline.
    let baseline_healthy = runtime_health(root).is_ok();
    let before = inventory(source, 512 * 1024 * 1024)?;
    let id = uuid::Uuid::new_v4().to_string();
    let base = root.join("backups").join(&id);
    mkdir(&base)?;
    let bytes = super::download::asset(transport, &manifest.tag, &manifest.asset)?;
    let archive = base.join("download.tar.gz");
    write(&archive, &bytes)?;
    super::archive::extract(&archive, &base, Path::new("inkscape-mcp-launcher"))
        .map_err(|e| e.to_string())?;
    let extracted = base.join("inkscape-mcp-launcher").join(APP);
    release::verify(&extracted, &manifest)?;
    // The standalone legacy bootstrap asset must be the very same signed helper.
    if installer::optional(&base.join("inkscape-mcp-launcher/bin/inkscape-mcp-launcher"))?
        != Some(manifest.bootstrap.executable.clone())
    {
        return Err("Manager archive bootstrap differs".into());
    }
    let stage = parent.join(format!(".inkscape-mcp-manager-stage-{id}.app"));
    // App publication is adjacent to its destination even when the selected MCP
    // installation is on another volume. Bootstrap backups stay with that root.
    if parent == Path::new("/Applications") {
        if present(&stage) {
            return Err("Manager staging destination already exists.".into());
        }
        fs::create_dir(&stage).map_err(|e| e.to_string())?;
        crate::client_management::bounded_output(
            Command::new("/usr/bin/ditto")
                .args(["--rsrc", "--extattr"])
                .arg(&extracted)
                .arg(&stage),
            8192,
        )
        .map_err(|e| e.to_string())?;
    } else {
        copy_tree(&extracted, &stage, 512 * 1024 * 1024)?;
    }
    release::verify(&stage, &manifest)?;
    // ditto preserves app metadata but does not promise durability. Sync every
    // checked file and directory, including permissions/xattrs, before journaling.
    sync_stage(&stage)?;
    if inventory(&extracted, 512 * 1024 * 1024)? != manifest.files {
        return Err("Manager changed during staging.".into());
    }
    sync_dir(parent)?;
    let mut bootstrap = Vec::new();
    // A fresh or runtime-only root has no permanent bootstrap to upgrade. Never
    // manufacture settings.json or select the offline runtime as a side effect.
    if [
        "bin/inkscape-mcp",
        "bin/inkscape-mcp-launcher",
        "bootstrap.json",
    ]
    .iter()
    .any(|n| root.join(n).exists() || root.join(n).is_symlink())
    {
        let bytes = super::instructions::read(&stage.join(release::HELPER), 128 * 1024 * 1024)?;
        for name in ["bin/inkscape-mcp-launcher", "bin/inkscape-mcp"] {
            installer::add(root, &base, &mut bootstrap, root.join(name), &bytes, true)?;
        }
        installer::add(
            root,
            &base,
            &mut bootstrap,
            root.join("bootstrap.json"),
            &serde_json::to_vec_pretty(&manifest.bootstrap).map_err(|e| e.to_string())?,
            false,
        )?;
    }
    let helper = super::instructions::read(actor, 128 * 1024 * 1024)?;
    installer::apply_file(&base.join("activation-helper"), &helper, 0o700)?;
    let plan = Plan {
        format: 1,
        id: id.clone(),
        source: source.into(),
        stage: stage.clone(),
        target: source.into(),
        backup: parent.join(format!(".inkscape-mcp-manager-backup-{id}.app")),
        before: Some(before),
        after: manifest.files.clone(),
        online: Some(Online {
            manifest,
            channel: channel.into(),
            profile: profile()?,
            observations: observed,
            bootstrap,
            helper: FileIdentity::of(&helper),
            baseline_healthy,
        }),
    };
    if observations(root)? != plan.online.as_ref().unwrap().observations
        || inventory(source, 512 * 1024 * 1024)? != *plan.before.as_ref().unwrap()
    {
        return Err("Installation changed during Manager staging; review again.".into());
    }
    write_json(&root.join("manager-prepared.json"), &plan)?;
    write_json(
        &root.join("manager-outcome.json"),
        &json!({"state":"prepared","preparation_id":plan.id}),
    )?;
    // Bind all metadata and identities, not only a preparation UUID.
    let review = FileIdentity::of(&serde_json::to_vec(&plan).map_err(|e| e.to_string())?).sha256;
    Ok(
        json!({"manager_online_prepared":true,"preparation_id":id,"review_id":review,"manager_application":source,"manager_stage":stage,"activation_helper":base.join("activation-helper"),"installed_manager":release::info(source)?,"installed_bootstrap":installed_bootstrap_identity(root),"target_manager":plan.online.as_ref().unwrap().manifest,"bootstrap_changes":plan.online.as_ref().unwrap().bootstrap.iter().map(|c|json!({"destination":c.destination,"from":c.before,"to":c.after})).collect::<Vec<_>>(),"baseline_healthy":baseline_healthy,"restart_required":true}),
    )
}

fn verify_prepared(root: &Path, plan: &Plan, review: &str) -> Result<()> {
    if FileIdentity::of(&serde_json::to_vec(plan).map_err(|e| e.to_string())?).sha256 != review {
        return Err("Manager review changed; prepare again.".into());
    }
    let online = plan.online.as_ref().ok_or("not an online update")?;
    if present(&plan.backup) {
        return Err("Manager backup destination appeared after Review; preserved.".into());
    }
    verify_last(root)?;
    preserves_selection(root, &online.manifest)?;
    if online.profile != profile()? {
        return Err("Manager profile changed after review.".into());
    }
    if observations(root)? != online.observations {
        return Err("Installation changed after Manager review.".into());
    }
    for c in &online.bootstrap {
        if installer::optional(&c.destination)? != c.before {
            return Err("Bootstrap changed after Manager review.".into());
        }
    }
    if inventory(&plan.target, 512 * 1024 * 1024)?
        != *plan.before.as_ref().ok_or("missing Manager baseline")?
    {
        return Err("Manager destination changed after review.".into());
    }
    release::verify(&plan.stage, &online.manifest)
}

/// A verified new Manager acknowledges actual startup. Launch Services success
/// alone is not activation success. The old app/bootstrap remain available.
fn finish_online(root: &Path, source: &Path) -> Result<Option<bool>> {
    if !root.join("manager-journal.json").exists() {
        return Ok(None);
    }
    let plan = load(root, "manager-journal.json")?;
    let Some(online) = &plan.online else {
        return Ok(None);
    };
    if source != plan.target {
        return Ok(None);
    }
    let _app_lock = Lock::manager(source.parent().unwrap())?;
    release::verify(source, &online.manifest)?;
    if observations(root)? != online.observations {
        return Err("Installation changed during Manager restart; journal preserved.".into());
    }
    for c in &online.bootstrap {
        let actual = installer::optional(&c.destination)?;
        if actual != c.before && actual != Some(c.after.clone()) {
            return Err("Bootstrap changed after interruption; preserved.".into());
        }
    }
    for (i, c) in online.bootstrap.iter().enumerate() {
        if installer::optional(&c.destination)? != Some(c.after.clone()) {
            installer::apply_file(
                &c.destination,
                &super::instructions::read(
                    &base(root, &plan).join(format!("after-{i}")),
                    c.after.bytes,
                )?,
                c.mode,
            )?;
        }
    }
    let health = runtime_health(root);
    if online.baseline_healthy {
        health.as_ref().map_err(|e| e.clone())?;
    }
    let healthy = health.is_ok();
    commit_manager_journal(root)?;
    Ok(Some(healthy))
}

pub fn recover_online(root: &Path) -> Result<bool> {
    if !root.join("manager-journal.json").exists() {
        return Ok(false);
    }
    let plan = load(root, "manager-journal.json")?;
    let _app_lock = Lock::manager(plan.target.parent().unwrap())?;
    idle(&plan.target)?;
    recover_plan(root, &plan)?;
    Ok(true)
}

fn publish_files(
    root: &Path,
    plan: &Plan,
    mut checkpoint: impl FnMut(&str) -> Result<()>,
) -> Result<()> {
    if present(&plan.backup) {
        return Err("Manager backup destination already exists; preserved.".into());
    }
    #[cfg(target_os = "macos")]
    {
        idle(&plan.target)?;
        idle(&plan.stage)?;
    }
    fs::rename(&plan.target, &plan.backup).map_err(|e| e.to_string())?;
    sync_dir(plan.target.parent().unwrap())?;
    checkpoint("app-backup")?;
    fs::rename(&plan.stage, &plan.target).map_err(|e| e.to_string())?;
    sync_dir(plan.target.parent().unwrap())?;
    checkpoint("app-publish")?;
    if let Some(online) = &plan.online {
        for (i, c) in online.bootstrap.iter().enumerate() {
            installer::apply_file(
                &c.destination,
                &super::instructions::read(
                    &base(root, plan).join(format!("after-{i}")),
                    c.after.bytes,
                )?,
                c.mode,
            )?;
            checkpoint(&format!("bootstrap-{i}"))?;
        }
    }
    Ok(())
}

pub fn activate_online(root: &Path, review: &str, parent_pid: i32) -> Result<Value> {
    #[cfg(target_os = "macos")]
    {
        let plan = load(root, "manager-prepared.json")?;
        if parent_pid <= 1
            || executable(parent_pid) != Some(plan.source.join(release::EXECUTABLE))
            || std::env::current_exe().map_err(|e| e.to_string())?
                != base(root, &plan).join("activation-helper")
        {
            return Err("Manager activation process identity differs.".into());
        }
        let lock = Lock::acquire(root)?;
        let app_lock = Lock::manager(plan.target.parent().unwrap())?;
        no_other_transaction(root)?;
        verify_prepared(root, &plan, review)?;
        idle_except(&plan.target, Some(parent_pid))?;
        // The UI exits only once this retained helper holds both locks and has
        // revalidated the whole review. Refusals leave the confirming UI open.
        println!(
            "{}",
            json!({"manager_activation_ready":true,"review_id":review})
        );
        std::io::Write::flush(&mut std::io::stdout()).map_err(|e| e.to_string())?;
        let deadline = Instant::now() + Duration::from_secs(30);
        while let Some(path) = executable(parent_pid) {
            if path != plan.source.join(release::EXECUTABLE) {
                return Err("Confirming Manager identity changed.".into());
            }
            if Instant::now() >= deadline {
                return Err("Confirming Manager did not exit; no files replaced.".into());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        idle(&plan.target)?;
        verify_prepared(root, &plan, review)?;
        if present(&plan.backup) {
            return Err("Manager backup destination already exists; preserved.".into());
        }
        idle(&plan.target)?;
        idle(&plan.stage)?;
        write_json(&root.join("manager-journal.json"), &plan)?;
        let publication = (|| {
            publish_files(root, &plan, |_| Ok(()))?;
            let online = plan.online.as_ref().unwrap();
            release::verify(&plan.target, &online.manifest)?;
            // Authorize the new permanent bootstrap for the retained runtime, using
            // the existing signer-bound receipts and genuine MCP probe.
            if online.baseline_healthy {
                probe_with_current_actor(root)?;
            }
            Ok(())
        })();
        if let Err(error) = publication {
            recover_plan(root, &plan)?;
            drop(app_lock);
            drop(lock);
            let _ = reopen(root, &plan);
            return Err(error);
        }
        drop(app_lock);
        drop(lock);
        let restart = reopen(root, &plan);
        if restart.is_ok() {
            let deadline = Instant::now() + Duration::from_secs(30);
            while root.join("manager-journal.json").exists() && Instant::now() < deadline {
                if acknowledged(root, review) {
                    return Ok(json!({"manager_ready":true,"manager_application":plan.target}));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            if acknowledged(root, review) {
                return Ok(json!({"manager_ready":true,"manager_application":plan.target}));
            }
            if !root.join("manager-journal.json").exists() {
                return Err("Manager update ended without its matching startup acknowledgment, possibly because offline recovery restored the previous app. Review the retained Manager outcome/backups before retrying.".into());
            }
        }
        let _lock = Lock::acquire(root)?;
        let _app_lock = Lock::manager(plan.target.parent().unwrap())?;
        if acknowledged(root, review) {
            return Ok(json!({"manager_ready":true,"manager_application":plan.target}));
        }
        // Never roll back underneath a Manager that did start but could not ack.
        idle(&plan.target).map_err(|_| "The new Manager started but did not acknowledge activation. Close it and choose recovery; manager-journal.json and the previous app/bootstrap were retained.".to_string())?;
        recover_plan(root, &plan)?;
        let _ = reopen(root, &plan);
        Err("Manager restart failed; the previous Manager/bootstrap were restored. Retry the prepared update.".into())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (root, review, parent_pid);
        Err("Manager updates require macOS".into())
    }
}
#[cfg(target_os = "macos")]
fn reopen(root: &Path, plan: &Plan) -> Result<()> {
    let online = plan.online.as_ref().ok_or("online context missing")?;
    let mut command = Command::new("/usr/bin/open");
    command.args(["-n", "-a"]).arg(&plan.target);
    for (name, path) in &online.profile {
        let mut value = std::ffi::OsString::from(format!("{name}="));
        value.push(path);
        command.arg("--env").arg(value);
    }
    command
        .args(["--args", "-InstallationDirectory"])
        .arg(root)
        .arg("-UpdateChannel")
        .arg(&online.channel);
    crate::client_management::bounded_output(&mut command, 8192).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn current_app() -> Result<PathBuf> {
    // Launch Services can spell the system /var alias even when the app was
    // prepared at its canonical /private/var location. Resolve only our running
    // executable, never an untrusted destination supplied by a client.
    let exe = std::env::current_exe()
        .and_then(|p| p.canonicalize())
        .map_err(|e| e.to_string())?;
    let contents = exe
        .parent()
        .filter(|p| p.file_name().is_some_and(|n| n == "Helpers"))
        .and_then(Path::parent)
        .ok_or("Manager relocation requires the bundled installer")?;
    if contents.file_name().is_none_or(|n| n != "Contents") {
        return Err("invalid Manager bundle".into());
    }
    Ok(contents
        .parent()
        .ok_or("invalid Manager bundle")?
        .to_owned())
}
fn verify(app: &Path) -> Result<()> {
    let info = super::instructions::read(&app.join("Contents/Info.plist"), 1024 * 1024)?;
    let info = plist::Value::from_reader(std::io::Cursor::new(info)).map_err(|e| e.to_string())?;
    if info
        .as_dictionary()
        .and_then(|d| d.get("CFBundleIdentifier"))
        .and_then(plist::Value::as_string)
        != Some("org.inkscape-mcp.manager")
    {
        return Err(
            "destination is not a managed Inkscape MCP Manager application; preserved".into(),
        );
    }
    crate::client_management::bounded_output(
        Command::new("/usr/bin/codesign")
            .args(["--verify", "--deep", "--strict"])
            .arg(app),
        8192,
    )
    .map_err(|_| "Manager signature verification failed".to_string())?;
    Ok(())
}
fn copy_verified_app(source: &Path, target: &Path) -> Result<()> {
    copy_tree(source, target, 1024 * 1024 * 1024)?;
    verify(target)
}

pub(super) fn snapshot_current(target: &Path) -> Result<PathBuf> {
    let source = current_app()?;
    copy_verified_app(&source, target)?;
    Ok(source)
}
#[cfg(target_os = "macos")]
#[link(name = "proc")]
unsafe extern "C" {
    fn proc_listallpids(buffer: *mut libc::c_void, buffersize: libc::c_int) -> libc::c_int;
    fn proc_pidpath(pid: libc::c_int, buffer: *mut libc::c_void, buffersize: u32) -> libc::c_int;
}
#[cfg(target_os = "macos")]
fn executable(pid: i32) -> Option<PathBuf> {
    let mut bytes = vec![0u8; 4096];
    let length = unsafe { proc_pidpath(pid, bytes.as_mut_ptr().cast(), bytes.len() as u32) };
    if length <= 0 {
        return None;
    }
    let end = bytes.iter().position(|b| *b == 0)?;
    Some(PathBuf::from(std::str::from_utf8(&bytes[..end]).ok()?))
}
fn idle(target: &Path) -> Result<()> {
    idle_except(target, None)
}
fn idle_except(target: &Path, allowed_pid: Option<i32>) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let mut pids = vec![0i32; 8192];
        let length = unsafe { proc_listallpids(pids.as_mut_ptr().cast(), (pids.len() * 4) as i32) };
        if length <= 0 || length as usize >= pids.len() {
            return Err(
                "unable to bound running Manager discovery; retry after closing Managers".into(),
            );
        }
        if pids.into_iter().take(length as usize).any(|pid| {
            pid != unsafe { libc::getpid() }
                && Some(pid) != allowed_pid
                && executable(pid).is_some_and(|p| p.starts_with(target))
        }) {
            return Err(
                "a Manager or helper is using the destination application; close it and retry"
                    .into(),
            );
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (target, allowed_pid);
        Err("Manager relocation requires macOS".into())
    }
}
fn load(root: &Path, name: &str) -> Result<Plan> {
    let plan: Plan = json(&root.join(name))?;
    let parent = if plan.online.is_some() {
        plan.target
            .parent()
            .ok_or("Manager parent missing")?
            .to_owned()
    } else {
        parent()?
    };
    if plan.format != 1
        || !identifier(&plan.id)
        || plan.target != parent.join(APP)
        || plan.online.is_some() && !installed_location(&plan.target, &self::parent()?)
        || plan.stage != parent.join(format!(".inkscape-mcp-manager-stage-{}.app", plan.id))
        || plan.backup != parent.join(format!(".inkscape-mcp-manager-backup-{}.app", plan.id))
    {
        return Err("invalid Manager relocation journal".into());
    }
    for files in [&plan.after].into_iter().chain(plan.before.as_ref()) {
        if files.len() > 50000
            || files.iter().any(|(name, id)| {
                !managed_relative(name) || !hash(&id.sha256) || id.bytes > 1024 * 1024 * 1024
            })
        {
            return Err("invalid Manager relocation inventory".into());
        }
    }
    destination_parent(&parent)?;
    if let Some(online) = &plan.online {
        online.manifest.validate()?;
        if online.manifest.files != plan.after
            || plan.source != plan.target
            || !matches!(online.channel.as_str(), "stable" | "prerelease")
            || online.profile.iter().any(|(name, path)| {
                !["HOME", "CODEX_HOME", "CFFIXED_USER_HOME"].contains(&name.as_str())
                    || !path.is_absolute()
            })
            || online.bootstrap.len() > 3
        {
            return Err("invalid Manager online plan/profile".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        for (i, c) in online.bootstrap.iter().enumerate() {
            let expected = if c.destination == root.join("bootstrap.json") {
                FileIdentity::of(
                    &serde_json::to_vec_pretty(&online.manifest.bootstrap)
                        .map_err(|e| e.to_string())?,
                )
            } else {
                online.manifest.bootstrap.executable.clone()
            };
            if ![
                "bin/inkscape-mcp",
                "bin/inkscape-mcp-launcher",
                "bootstrap.json",
            ]
            .iter()
            .any(|n| root.join(n) == c.destination)
                || !seen.insert(&c.destination)
                || c.mode & !0o777 != 0
                || c.after != expected
                || installer::optional(&base(root, &plan).join(format!("after-{i}")))?
                    != Some(c.after.clone())
                || c.before.is_some()
                    && installer::optional(&base(root, &plan).join(format!("before-{i}")))?
                        != c.before
            {
                return Err("Manager bootstrap backup differs".into());
            }
        }
        if installer::optional(&base(root, &plan).join("activation-helper"))?
            != Some(online.helper.clone())
        {
            return Err("Manager activation helper changed".into());
        }
        if online.observations.keys().any(|p| {
            !["active.json", "settings.json", "manager-last.json"]
                .iter()
                .any(|name| *p == root.join(name))
        }) {
            return Err("invalid Manager observations".into());
        }
    }
    Ok(plan)
}
fn recover(root: &Path) -> Result<()> {
    if !root.join("manager-journal.json").exists() {
        return Ok(());
    }
    let plan = load(root, "manager-journal.json")?;
    idle(&plan.target)?;
    recover_plan(root, &plan)
}
fn recover_plan(root: &Path, plan: &Plan) -> Result<()> {
    recover_plan_with(root, plan, |_| Ok(()))
}
fn recover_plan_with(
    root: &Path,
    plan: &Plan,
    mut checkpoint: impl FnMut(&str) -> Result<()>,
) -> Result<()> {
    if !root.join("manager-journal.json").exists() {
        return Ok(());
    }
    if present(&plan.stage) && inventory(&plan.stage, 1024 * 1024 * 1024)? != plan.after {
        return Err("Manager stage changed after interruption; preserved.".into());
    }
    // Preflight every bootstrap and app location before moving either component.
    // Recovery can itself stop between writes, so accept only the old/new identities.
    if let Some(online) = &plan.online {
        for c in &online.bootstrap {
            let actual = installer::optional(&c.destination)?;
            if actual != c.before && actual != Some(c.after.clone()) {
                return Err("bootstrap changed after interruption; preserved".into());
            }
        }
    }
    if let Some(before) = &plan.before
        && present(&plan.backup)
        && inventory(&plan.backup, 1024 * 1024 * 1024)? != *before
    {
        return Err("Manager backup changed; preserved".into());
    }
    if plan.before.is_some()
        && !present(&plan.backup)
        && (!present(&plan.target)
            || inventory(&plan.target, 1024 * 1024 * 1024)? != *plan.before.as_ref().unwrap())
    {
        return Err("Manager backup is missing; all destinations preserved.".into());
    }
    if present(&plan.target) {
        let actual = inventory(&plan.target, 1024 * 1024 * 1024)?;
        if Some(&actual) == plan.before.as_ref() && !present(&plan.backup) {
            // App already restored; finish any interrupted bootstrap restoration.
        } else {
            if actual != plan.after || present(&plan.stage) {
                return Err(
                    "Manager changed after interruption; preserve it and recover manually".into(),
                );
            }
            #[cfg(target_os = "macos")]
            {
                idle(&plan.target)?;
                idle(&plan.stage)?;
            }
            fs::rename(&plan.target, &plan.stage).map_err(|e| e.to_string())?;
            sync_dir(plan.target.parent().unwrap())?;
            checkpoint("app-unpublish")?;
        }
    }
    if let Some(before) = &plan.before
        && !present(&plan.target)
    {
        if inventory(&plan.backup, 1024 * 1024 * 1024)? != *before {
            return Err("Manager backup changed; preserved".into());
        }
        #[cfg(target_os = "macos")]
        {
            idle(&plan.target)?;
            idle(&plan.backup)?;
        }
        fs::rename(&plan.backup, &plan.target).map_err(|e| e.to_string())?;
        sync_dir(plan.target.parent().unwrap())?;
        checkpoint("app-restore")?;
    }
    if let Some(online) = &plan.online {
        for (i, c) in online.bootstrap.iter().enumerate().rev() {
            if installer::optional(&c.destination)? == c.before {
                continue;
            }
            if let Some(before) = &c.before {
                installer::apply_file(
                    &c.destination,
                    &super::instructions::read(
                        &base(root, plan).join(format!("before-{i}")),
                        before.bytes,
                    )?,
                    c.mode,
                )?;
            } else {
                fs::remove_file(&c.destination).map_err(|e| e.to_string())?;
                sync_dir(c.destination.parent().unwrap())?;
            }
            checkpoint(&format!("bootstrap-restore-{i}"))?;
        }
    }
    fs::remove_file(root.join("manager-journal.json")).map_err(|e| e.to_string())?;
    sync_dir(root)
}
pub fn prepare(root: &Path) -> Result<Value> {
    if !cfg!(target_os = "macos") {
        return Err("Manager relocation requires macOS".into());
    }
    let source = current_app()?;
    verify(&source)?;
    let parent = parent()?;
    mkdir(&parent)?;
    mkdir(root)?;
    let _lock = Lock::acquire(root)?;
    if let Some(healthy) = finish_online(root, &source)? {
        return Ok(
            json!({"manager_ready":true,"manager_application":source,"manager_update_completed":true,"runtime_warning":if healthy {None} else {Some("Manager updated. The existing MCP installation needs verification or repair. Choose Review changes.")}}),
        );
    }
    // The journal may target /Applications even when this distribution would
    // relocate to ~/Applications. Recover under the journal's actual app lock.
    recover_online(root)?;
    let _app_lock = Lock::manager(&parent)?;
    let target = parent.join(APP);
    if installed_location(&source, &parent) {
        return Ok(json!({"manager_ready":true,"manager_application":source}));
    }
    idle(&target)?;
    let before = if target.exists() || target.is_symlink() {
        verify(&target)?;
        let source_team =
            crate::client_management::signature_team(&source).map_err(|e| e.to_string())?;
        let target_team =
            crate::client_management::signature_team(&target).map_err(|e| e.to_string())?;
        if target_team.is_some() && source_team != target_team {
            return Err(
                "Manager signing team differs from the installed application; preserved".into(),
            );
        }
        Some(inventory(&target, 1024 * 1024 * 1024)?)
    } else {
        None
    };
    let id = uuid::Uuid::new_v4().to_string();
    let stage = parent.join(format!(".inkscape-mcp-manager-stage-{id}.app"));
    let backup = parent.join(format!(".inkscape-mcp-manager-backup-{id}.app"));
    copy_verified_app(&source, &stage)?;
    let after = inventory(&stage, 1024 * 1024 * 1024)?;
    let plan = Plan {
        format: 1,
        id: id.clone(),
        source,
        stage: stage.clone(),
        target: target.clone(),
        backup,
        before,
        after,
        online: None,
    };
    write_json(&root.join("manager-prepared.json"), &plan)?;
    Ok(
        json!({"manager_ready":false,"preparation_id":id,"manager_stage":stage,"manager_application":target,"replaces_existing_manager":plan.before.is_some()}),
    )
}
pub fn activate(root: &Path, id: &str, parent_pid: i32) -> Result<Value> {
    #[cfg(target_os = "macos")]
    {
        let plan = load(root, "manager-prepared.json")?;
        if plan.id != id
            || current_app()? != plan.stage
            || parent_pid <= 1
            || parent_pid == unsafe { libc::getpid() }
        {
            return Err("Manager activation context differs".into());
        }
        let deadline = Instant::now() + Duration::from_secs(30);
        while let Some(path) = executable(parent_pid) {
            if path != plan.source.join("Contents/MacOS/inkscape-mcp-manager") {
                return Err("parent Manager identity changed; preserved".into());
            }
            if Instant::now() >= deadline {
                return Err(
                    "parent Manager has not exited; retry relocation after closing it".into(),
                );
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _lock = Lock::acquire(root)?;
        recover_online(root)?;
        let _app_lock = Lock::manager(plan.target.parent().unwrap())?;
        idle(&plan.target)?;
        if inventory(&plan.stage, 1024 * 1024 * 1024)? != plan.after
            || (if present(&plan.target) {
                Some(inventory(&plan.target, 1024 * 1024 * 1024)?)
            } else {
                None
            }) != plan.before
        {
            return Err("Manager files changed after preparation; preserved".into());
        }
        verify(&plan.stage)?;
        if present(&plan.backup) {
            return Err("Manager backup destination already exists; preserved.".into());
        }
        idle(&plan.target)?;
        idle(&plan.stage)?;
        write_json(&root.join("manager-journal.json"), &plan)?;
        let result = (|| {
            if plan.before.is_some() {
                fs::rename(&plan.target, &plan.backup).map_err(|e| e.to_string())?;
                sync_dir(plan.target.parent().unwrap())?;
            }
            fs::rename(&plan.stage, &plan.target).map_err(|e| e.to_string())?;
            sync_dir(plan.target.parent().unwrap())?;
            verify(&plan.target)?;
            fs::remove_file(root.join("manager-journal.json")).map_err(|e| e.to_string())?;
            sync_dir(root)?;
            let mut reopen = Command::new("/usr/bin/open");
            reopen.args(["-n", "-a"]).arg(&plan.target);
            // Launch Services does not automatically preserve an isolated/custom
            // profile. Carry only fixed profile paths, never monitoring credentials.
            for name in ["HOME", "CODEX_HOME", "CFFIXED_USER_HOME"] {
                if let Some(value) = std::env::var_os(name) {
                    let path = PathBuf::from(&value);
                    if !path.is_absolute() {
                        return Err("Manager profile path must be absolute".into());
                    }
                    let mut assignment = std::ffi::OsString::from(format!("{name}="));
                    assignment.push(value);
                    reopen.arg("--env").arg(assignment);
                }
            }
            reopen.args(["--args", "-InstallationDirectory"]).arg(root);
            crate::client_management::bounded_output(&mut reopen, 8192)
                .map_err(|e| e.to_string())?;
            Ok(json!({"manager_ready":true,"manager_application":plan.target}))
        })();
        if result.is_err() {
            recover(root)?;
        }
        result
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (root, id, parent_pid);
        Err("Manager relocation requires macOS".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn verified_snapshot_rejects_modified_signed_instructions() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let app = root.join("Original.app");
        let executable = app.join("Contents/MacOS/manager");
        let instructions = app.join("Contents/Resources/instructions/initialize.txt");
        mkdir(executable.parent().unwrap()).unwrap();
        mkdir(instructions.parent().unwrap()).unwrap();
        fs::copy("/usr/bin/true", &executable).unwrap();
        let plist = br#"<?xml version="1.0" encoding="UTF-8"?>
    <plist version="1.0"><dict>
    <key>CFBundleIdentifier</key><string>org.inkscape-mcp.manager</string>
    <key>CFBundleExecutable</key><string>manager</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    </dict></plist>"#;
        write(&app.join("Contents/Info.plist"), plist).unwrap();
        write(&instructions, b"sealed instructions").unwrap();
        crate::client_management::bounded_output(
            Command::new("/usr/bin/codesign")
                .args(["--force", "--sign", "-"])
                .arg(&app),
            8192,
        )
        .unwrap();

        let valid = root.join("Verified.app");
        copy_verified_app(&app, &valid).unwrap();
        write(&instructions, b"modified instructions").unwrap();
        let error = copy_verified_app(&app, &root.join("Modified.app")).unwrap_err();
        assert_eq!(error, "Manager signature verification failed");
        assert_eq!(
            fs::read(valid.join("Contents/Resources/instructions/initialize.txt")).unwrap(),
            b"sealed instructions",
        );
    }
    #[test]
    fn copied_managers_in_both_applications_folders_are_ready_without_relocation() {
        let parent = Path::new("/Users/example/Applications");
        assert!(installed_location(&parent.join(APP), parent));
        assert!(installed_location(
            &Path::new("/Applications").join(APP),
            parent
        ));
        for path in [
            "/Volumes/Inkscape MCP Manager/Inkscape MCP Manager.app",
            "/Users/example/Downloads/Inkscape MCP Manager.app",
            "/Applications/Other.app",
            "/Applications/Subfolder/Inkscape MCP Manager.app",
        ] {
            assert!(!installed_location(Path::new(path), parent));
        }
    }
    fn fixture() -> (tempfile::TempDir, PathBuf, Plan) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let installation = root.join("installation");
        mkdir(&installation).unwrap();
        let target = root.join(APP);
        let stage = root.join("stage.app");
        let backup = root.join("backup.app");
        mkdir(&target).unwrap();
        write(&target.join("payload"), b"old application").unwrap();
        mkdir(&stage).unwrap();
        write(&stage.join("payload"), b"new application").unwrap();
        let before = Some(inventory(&target, 1000).unwrap());
        let after = inventory(&stage, 1000).unwrap();
        let plan = Plan {
            format: 1,
            id: "test".into(),
            source: root.join("source.app"),
            stage,
            target,
            backup,
            before,
            after,
            online: None,
        };
        write_json(&installation.join("manager-journal.json"), &plan).unwrap();
        (temp, installation, plan)
    }
    fn coordinated_fixture() -> (tempfile::TempDir, PathBuf, Plan) {
        let (temp, root, mut plan) = fixture();
        let base = base(&root, &plan);
        mkdir(&base).unwrap();
        mkdir(&root.join("bin")).unwrap();
        let mut changes = vec![];
        for name in [
            "bin/inkscape-mcp-launcher",
            "bin/inkscape-mcp",
            "bootstrap.json",
        ] {
            write(&root.join(name), b"old bootstrap").unwrap();
            installer::add(
                &root,
                &base,
                &mut changes,
                root.join(name),
                b"new bootstrap",
                name.starts_with("bin/"),
            )
            .unwrap();
        }
        plan.online = Some(Online {
            manifest: release::fixture_manifest(),
            channel: "prerelease".into(),
            profile: profile().unwrap(),
            observations: observations(&root).unwrap(),
            bootstrap: changes,
            helper: FileIdentity::of(b"test helper"),
            baseline_healthy: false,
        });
        write_json(&root.join("manager-journal.json"), &plan).unwrap();
        (temp, root, plan)
    }
    #[test]
    fn coordinated_app_bootstrap_publication_and_recovery_boundaries_are_idempotent() {
        for boundary in [
            "app-backup",
            "app-publish",
            "bootstrap-0",
            "bootstrap-1",
            "bootstrap-2",
        ] {
            let (_temp, root, plan) = coordinated_fixture();
            assert!(
                publish_files(&root, &plan, |at| if at == boundary {
                    Err("interrupted".into())
                } else {
                    Ok(())
                })
                .is_err()
            );
            recover_plan(&root, &plan).unwrap();
            recover_plan(&root, &plan).unwrap();
            assert_eq!(
                inventory(&plan.target, 1000).unwrap(),
                *plan.before.as_ref().unwrap()
            );
            for c in &plan.online.as_ref().unwrap().bootstrap {
                assert_eq!(installer::optional(&c.destination).unwrap(), c.before);
            }
            assert_eq!(inventory(&plan.stage, 1000).unwrap(), plan.after);
        }
        for boundary in [
            "app-unpublish",
            "app-restore",
            "bootstrap-restore-2",
            "bootstrap-restore-1",
            "bootstrap-restore-0",
        ] {
            let (_temp, root, plan) = coordinated_fixture();
            publish_files(&root, &plan, |_| Ok(())).unwrap();
            assert!(
                recover_plan_with(&root, &plan, |at| if at == boundary {
                    Err("recovery interrupted".into())
                } else {
                    Ok(())
                })
                .is_err()
            );
            recover_plan(&root, &plan).unwrap();
            recover_plan(&root, &plan).unwrap();
            assert_eq!(
                inventory(&plan.target, 1000).unwrap(),
                *plan.before.as_ref().unwrap()
            );
            for c in &plan.online.as_ref().unwrap().bootstrap {
                assert_eq!(installer::optional(&c.destination).unwrap(), c.before);
            }
        }
    }
    #[test]
    fn coordinated_crash_child() {
        let Some(root) = std::env::var_os("INKSCAPE_MCP_TEST_MANAGER_CRASH_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        assert_eq!(
            super::super::instructions::read(&root.join("owned-crash-fixture"), 100).unwrap(),
            b"owned Manager self-update test"
        );
        let plan: Plan = json(&root.join("manager-journal.json")).unwrap();
        let _lock = Lock::acquire(&root).unwrap();
        let _app_lock = Lock::manager(plan.target.parent().unwrap()).unwrap();
        let boundary = std::env::var("INKSCAPE_MCP_TEST_MANAGER_CRASH_BOUNDARY").unwrap();
        let crash = |at: &str| {
            if at == boundary {
                std::process::exit(91);
            }
            Ok(())
        };
        if std::env::var("INKSCAPE_MCP_TEST_MANAGER_CRASH_ACTION").unwrap() == "publish" {
            publish_files(&root, &plan, crash).unwrap();
        } else {
            recover_plan_with(&root, &plan, crash).unwrap();
        }
        panic!("requested crash boundary was not reached");
    }
    #[test]
    fn abrupt_process_exit_releases_locks_and_recovers_each_app_bootstrap_boundary() {
        for (action, boundaries) in [
            (
                "publish",
                vec![
                    "app-backup",
                    "app-publish",
                    "bootstrap-0",
                    "bootstrap-1",
                    "bootstrap-2",
                ],
            ),
            (
                "recover",
                vec![
                    "app-unpublish",
                    "app-restore",
                    "bootstrap-restore-2",
                    "bootstrap-restore-1",
                    "bootstrap-restore-0",
                ],
            ),
        ] {
            for boundary in boundaries {
                let (_temp, root, plan) = coordinated_fixture();
                write(
                    &root.join("owned-crash-fixture"),
                    b"owned Manager self-update test",
                )
                .unwrap();
                if action == "recover" {
                    publish_files(&root, &plan, |_| Ok(())).unwrap();
                }
                let mut child = Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "update::manager_app::tests::coordinated_crash_child",
                    ])
                    .env("INKSCAPE_MCP_TEST_MANAGER_CRASH_ROOT", &root)
                    .env("INKSCAPE_MCP_TEST_MANAGER_CRASH_BOUNDARY", boundary)
                    .env("INKSCAPE_MCP_TEST_MANAGER_CRASH_ACTION", action)
                    .stdout(std::process::Stdio::null())
                    .spawn()
                    .unwrap();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                let status = loop {
                    if let Some(status) = child.try_wait().unwrap() {
                        break status;
                    }
                    if std::time::Instant::now() >= deadline {
                        child.kill().unwrap();
                        child.wait().unwrap();
                        panic!("owned crash fixture exceeded deadline");
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                };
                assert_eq!(status.code(), Some(91), "{action}/{boundary}");
                let _lock = Lock::acquire(&root).unwrap();
                let _app_lock = Lock::manager(plan.target.parent().unwrap()).unwrap();
                recover_plan(&root, &plan).unwrap();
                recover_plan(&root, &plan).unwrap();
                assert_eq!(
                    inventory(&plan.target, 1000).unwrap(),
                    *plan.before.as_ref().unwrap()
                );
                assert_eq!(inventory(&plan.stage, 1000).unwrap(), plan.after);
                for c in &plan.online.as_ref().unwrap().bootstrap {
                    assert_eq!(installer::optional(&c.destination).unwrap(), c.before);
                }
            }
        }
    }
    #[test]
    fn coordinated_recovery_preflights_every_location_and_preserves_later_changes() {
        for changed in ["app", "backup", "bootstrap", "missing-backup"] {
            let (_temp, root, plan) = coordinated_fixture();
            publish_files(&root, &plan, |_| Ok(())).unwrap();
            match changed {
                "app" => write(&plan.target.join("payload"), b"user change").unwrap(),
                "backup" => write(&plan.backup.join("payload"), b"user change").unwrap(),
                "bootstrap" => write(&root.join("bin/inkscape-mcp"), b"later bootstrap").unwrap(),
                _ => fs::remove_dir_all(&plan.backup).unwrap(),
            }
            let before = inventory(root.parent().unwrap(), 1024 * 1024).unwrap();
            assert!(recover_plan(&root, &plan).is_err());
            assert_eq!(
                inventory(root.parent().unwrap(), 1024 * 1024).unwrap(),
                before
            );
        }
    }
    #[test]
    fn manager_review_pins_metadata_selection_bootstrap_and_complete_destination() {
        for changed in [
            "manifest",
            "settings",
            "active",
            "history",
            "bootstrap",
            "app",
        ] {
            let (_temp, root, mut plan) = coordinated_fixture();
            let review = FileIdentity::of(&serde_json::to_vec(&plan).unwrap()).sha256;
            match changed {
                "manifest" => plan.online.as_mut().unwrap().manifest.tag = "v99.0.0".into(),
                "settings" | "active" => {
                    write(&root.join(format!("{changed}.json")), b"later selection").unwrap()
                }
                "bootstrap" => write(&root.join("bin/inkscape-mcp"), b"later bootstrap").unwrap(),
                "history" => {
                    write(&root.join("manager-last.json"), b"foreign later history").unwrap()
                }
                _ => write(&plan.target.join("payload"), b"later app").unwrap(),
            }
            assert!(verify_prepared(&root, &plan, &review).is_err());
        }
    }
    #[test]
    fn publication_preserves_a_backup_directory_created_after_review() {
        let (_temp, root, plan) = coordinated_fixture();
        mkdir(&plan.backup).unwrap();
        let before = inventory(root.parent().unwrap(), 1024 * 1024).unwrap();
        assert!(publish_files(&root, &plan, |_| Ok(())).is_err());
        assert_eq!(
            inventory(root.parent().unwrap(), 1024 * 1024).unwrap(),
            before
        );
    }
    #[test]
    fn target_bootstrap_refuses_incompatible_retained_components_without_settings_or_mutation() {
        use super::super::install::{Pair, Selector};
        let (_temp, root, _plan) = fixture();
        let manifest = release::fixture_manifest();
        let pair = Pair {
            tag: "v1.0.0".into(),
            runtime: RuntimeManifest {
                format: 1,
                distribution_tag: "v1.0.0".into(),
                build_id: "retained".into(),
                source_revision: "old".into(),
                os: "macos".into(),
                architecture: "aarch64".into(),
                minimum_os_major: 15,
                text_interface: TEXT_INTERFACE,
                helper_protocol: HELPER_PROTOCOL,
                launcher_minimum: 1,
                asset: manifest.asset.clone(),
            },
            instructions: super::super::instructions::manifest(
                "old",
                &super::super::instructions::default_files(),
            ),
        };
        let selected = Selector {
            format: 1,
            channel: "stable".into(),
            current: pair.clone(),
            previous: Some(pair),
        };
        write_json(&root.join("active.json"), &selected).unwrap();
        let before = inventory(&root, 1024 * 1024).unwrap();
        preserves_selection(&root, &manifest).unwrap();
        for field in [
            "format", "selector", "text", "helper", "launcher", "previous",
        ] {
            let mut incompatible = manifest.clone();
            let mut selection = selected.clone();
            match field {
                "format" => incompatible.component_format += 1,
                "selector" => selection.format += 1,
                "text" => incompatible.text_interface += 1,
                "helper" => incompatible.helper_protocol += 1,
                "launcher" => incompatible.launcher_version = 0,
                _ => selection.previous.as_mut().unwrap().runtime.helper_protocol += 1,
            };
            write_json(&root.join("active.json"), &selection).unwrap();
            let snapshot = inventory(&root, 1024 * 1024).unwrap();
            assert!(preserves_selection(&root, &incompatible).is_err());
            assert_eq!(inventory(&root, 1024 * 1024).unwrap(), snapshot);
        }
        write_json(&root.join("active.json"), &selected).unwrap();
        assert_eq!(inventory(&root, 1024 * 1024).unwrap(), before);
        assert!(!root.join("settings.json").exists());
        let settings = super::super::install::Settings {
            format: FORMAT,
            legacy_skill_owner: root.clone(),
            skills: vec![],
            environment: BTreeMap::from([
                (
                    "INKSCAPE_MCP_WORKSPACE_ROOTS".into(),
                    root.display().to_string(),
                ),
                ("PATH".into(), "/usr/bin:/bin".into()),
                ("INKSCAPE_MCP_LIVE_ENABLED".into(), "false".into()),
                ("INKSCAPE_MCP_ENGINE_MODE".into(), "per_call".into()),
            ]),
        };
        write_json(&root.join("settings.json"), &settings).unwrap();
        for field in ["launcher", "helper"] {
            let mut future = selected.clone();
            future.previous = None;
            let mut target = manifest.clone();
            if field == "launcher" {
                future.current.runtime.launcher_minimum = LAUNCHER_VERSION + 1;
                target.launcher_version = LAUNCHER_VERSION + 1;
            } else {
                future.current.runtime.helper_protocol = HELPER_PROTOCOL + 1;
                target.helper_protocol = HELPER_PROTOCOL + 1;
            }
            write_json(&root.join("active.json"), &future).unwrap();
            preserves_selection(&root, &target).unwrap();
            assert!(!actor_supports(&future.current));
            // No runtime files exist; an attempted old-actor probe would fail.
            probe_with_current_actor(&root).unwrap();
        }
        fs::remove_file(root.join("active.json")).unwrap();
        write_json(
            &root.join("settings.json"),
            &json!({"format":FORMAT,"preserved":"synthetic private setting"}),
        )
        .unwrap();
        let saved = inventory(&root, 1024 * 1024).unwrap();
        let mut target = manifest;
        target.component_format += 1;
        assert!(preserves_selection(&root, &target).is_err());
        assert_eq!(inventory(&root, 1024 * 1024).unwrap(), saved);
    }
    #[cfg(unix)]
    #[test]
    fn complete_stage_sync_refuses_links_without_following_them() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        mkdir(&root.join("Contents/Resources/empty")).unwrap();
        write(&root.join("Contents/Resources/file"), b"sealed bytes").unwrap();
        sync_stage(&root).unwrap();
        std::os::unix::fs::symlink("missing", root.join("Contents/link")).unwrap();
        assert!(sync_stage(&root).is_err());
        assert!(root.join("Contents/link").is_symlink());
    }
    #[test]
    fn startup_ack_requires_atomic_matching_commit_not_offline_rollback() {
        let (_temp, root, plan) = coordinated_fixture();
        let review = FileIdentity::of(&serde_json::to_vec(&plan).unwrap()).sha256;
        publish_files(&root, &plan, |_| Ok(())).unwrap();
        assert!(!acknowledged(&root, &review));
        commit_manager_journal(&root).unwrap();
        assert!(!root.join("manager-journal.json").exists());
        assert!(acknowledged(&root, &review));
        assert!(!recover_online(&root).unwrap());
        assert_eq!(inventory(&plan.target, 1000).unwrap(), plan.after);
        for c in &plan.online.as_ref().unwrap().bootstrap {
            assert_eq!(
                installer::optional(&c.destination).unwrap(),
                Some(c.after.clone())
            );
        }
        let (_temp, root, plan) = coordinated_fixture();
        let review = FileIdentity::of(&serde_json::to_vec(&plan).unwrap()).sha256;
        publish_files(&root, &plan, |_| Ok(())).unwrap();
        recover_plan(&root, &plan).unwrap();
        assert!(!root.join("manager-journal.json").exists());
        assert!(!acknowledged(&root, &review));
    }
    #[test]
    fn destination_lock_serializes_managers_using_different_installations() {
        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path().canonicalize().unwrap();
        let lock = Lock::manager(&parent).unwrap();
        assert!(Lock::manager(&parent).is_err());
        drop(lock);
        Lock::manager(&parent).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn recovery_preserves_later_dangling_target_stage_and_backup_links() {
        for slot in ["target", "stage", "backup"] {
            let (_temp, root, plan) = coordinated_fixture();
            publish_files(&root, &plan, |_| Ok(())).unwrap();
            let path = match slot {
                "target" => &plan.target,
                "stage" => &plan.stage,
                _ => &plan.backup,
            };
            if path.exists() {
                fs::rename(path, root.join("user-preserved.app")).unwrap();
            }
            std::os::unix::fs::symlink("later-user-dangling-link", path).unwrap();
            let before = plan
                .online
                .as_ref()
                .unwrap()
                .bootstrap
                .iter()
                .map(|c| installer::optional(&c.destination).unwrap())
                .collect::<Vec<_>>();
            assert!(recover_plan(&root, &plan).is_err());
            assert_eq!(
                fs::read_link(path).unwrap(),
                Path::new("later-user-dangling-link")
            );
            assert_eq!(
                plan.online
                    .as_ref()
                    .unwrap()
                    .bootstrap
                    .iter()
                    .map(|c| installer::optional(&c.destination).unwrap())
                    .collect::<Vec<_>>(),
                before
            );
            assert!(root.join("manager-journal.json").exists());
        }
    }
    #[test]
    fn every_manager_move_boundary_restores_old_application_and_retains_new_candidate() {
        for boundary in 0..=2 {
            let (_temp, root, plan) = fixture();
            if boundary >= 1 {
                fs::rename(&plan.target, &plan.backup).unwrap();
            }
            if boundary >= 2 {
                fs::rename(&plan.stage, &plan.target).unwrap();
            }
            recover_plan(&root, &plan).unwrap();
            assert_eq!(inventory(&plan.target, 1000).unwrap(), plan.before.unwrap());
            assert_eq!(inventory(&plan.stage, 1000).unwrap(), plan.after);
            assert!(!root.join("manager-journal.json").exists());
        }
    }
    #[test]
    fn changed_manager_backup_or_destination_is_preserved_before_any_move() {
        for damage in ["backup", "target"] {
            let (_temp, root, plan) = fixture();
            fs::rename(&plan.target, &plan.backup).unwrap();
            fs::rename(&plan.stage, &plan.target).unwrap();
            let changed = if damage == "backup" {
                &plan.backup
            } else {
                &plan.target
            };
            write(&changed.join("payload"), b"later edit").unwrap();
            let before_target = inventory(&plan.target, 1000).unwrap();
            let before_backup = inventory(&plan.backup, 1000).unwrap();
            assert!(recover_plan(&root, &plan).is_err());
            assert_eq!(inventory(&plan.target, 1000).unwrap(), before_target);
            assert_eq!(inventory(&plan.backup, 1000).unwrap(), before_backup);
            assert!(!present(&plan.stage));
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
#[path = "manager_app_native_tests.rs"]
mod native_tests;
