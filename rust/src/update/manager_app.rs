//! Explicit per-user Manager relocation. Never replaces a running destination app.
use super::{instructions::guarded, manifests::*, storage::*};
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
}
fn parent() -> Result<PathBuf> {
    Ok(PathBuf::from(std::env::var_os("HOME").ok_or("HOME missing")?).join("Applications"))
}
fn current_app() -> Result<PathBuf> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
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
        let _ = target;
        Err("Manager relocation requires macOS".into())
    }
}
fn load(root: &Path, name: &str) -> Result<Plan> {
    let plan: Plan = json(&root.join(name))?;
    let parent = parent()?;
    if plan.format != 1
        || !identifier(&plan.id)
        || plan.target != parent.join(APP)
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
    guarded(&parent)?;
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
    if let Some(before) = &plan.before
        && plan.backup.exists()
        && inventory(&plan.backup, 1024 * 1024 * 1024)? != *before
    {
        return Err("Manager backup changed; preserved".into());
    }
    if plan.target.exists() {
        let actual = inventory(&plan.target, 1024 * 1024 * 1024)?;
        if Some(&actual) == plan.before.as_ref() && !plan.backup.exists() {
            fs::remove_file(root.join("manager-journal.json")).map_err(|e| e.to_string())?;
            return sync_dir(root);
        }
        if actual != plan.after || plan.stage.exists() {
            return Err(
                "Manager changed after interruption; preserve it and recover manually".into(),
            );
        }
        fs::rename(&plan.target, &plan.stage).map_err(|e| e.to_string())?;
        sync_dir(plan.target.parent().unwrap())?;
    }
    if let Some(before) = &plan.before {
        if inventory(&plan.backup, 1024 * 1024 * 1024)? != *before {
            return Err("Manager backup changed; preserved".into());
        }
        fs::rename(&plan.backup, &plan.target).map_err(|e| e.to_string())?;
        sync_dir(plan.target.parent().unwrap())?;
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
    recover(root)?;
    let target = parent.join(APP);
    if source == target {
        return Ok(json!({"manager_ready":true,"manager_application":target}));
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
    copy_tree(&source, &stage, 1024 * 1024 * 1024)?;
    verify(&stage)?;
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
        recover(root)?;
        idle(&plan.target)?;
        if inventory(&plan.stage, 1024 * 1024 * 1024)? != plan.after
            || (if plan.target.exists() {
                Some(inventory(&plan.target, 1024 * 1024 * 1024)?)
            } else {
                None
            }) != plan.before
        {
            return Err("Manager files changed after preparation; preserved".into());
        }
        verify(&plan.stage)?;
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
        };
        write_json(&installation.join("manager-journal.json"), &plan).unwrap();
        (temp, installation, plan)
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
            assert!(!plan.stage.exists());
        }
    }
}
