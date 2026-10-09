//! Permanent offline MCP entry point and explicit update management CLI.
use inkscape_mcp_rust::update::{
    download::Https,
    install::*,
    manager::{self, Component},
    manifests::*,
    storage::*,
    transaction,
};
use serde_json::{Value, json};
use std::{env, path::PathBuf};
const HELP: &str = "inkscape-mcp — permanent MCP launcher and updater\n\nNo arguments: start MCP (offline; JSON-RPC only on stdout).\n  update --check            Show installed and available versions (read-only)\n  update --instructions     Update server text and merge managed skills\n  update --runtime          Update the complete compatible runtime\n  update                    Update a compatible runtime/instruction pair\n  rollback                  Restore the preceding pair and owned skill state\n  disconnect --client NAME  Remove an owned Codex/Claude binding (runtime-independent)\n  uninstall                 Disconnect owned clients and archive the installation/skills\n  --version                 Show selected runtime/text/launcher identity\n  install-inspect           Read installation and client discovery\n  install-prepare           Prepare a bounded installation request from stdin JSON\n  install-activate --preparation ID  Activate the reviewed prepared installation\n  install-recover           Recover interrupted installation changes\n  manager-prepare           Stage the bundled per-user Manager application\n  manager-activate --preparation ID --parent-pid PID  Replace after Manager exits\n  migrate --source DIR      Migrate a configured source/ready installation once\n\nOptions: --install-dir DIR (isolated installation), --channel stable|prerelease,\n         --release TAG (explicit immutable release), --json (structured result).\nDefault channel: stable. Prereleases require --channel prerelease.\nReconnect clients after activation; running MCP/Inkscape sessions retain their helpers.\nExit codes: 0 success/no update, 1 failed operation, 2 invalid arguments. Never prompts.\nAdd INSTALL_DIR/bin to PATH, or use its absolute inkscape-mcp path.";
struct Options {
    client: Option<String>,
    action: String,
    root: PathBuf,
    source: Option<PathBuf>,
    preparation: Option<String>,
    parent_pid: Option<i32>,
    channel: Option<String>,
    release: Option<String>,
    component: Component,
    check: bool,
    json: bool,
}
fn parse() -> Result<Options> {
    let mut input = env::args().skip(1).peekable();
    let action = input
        .peek()
        .filter(|s| !s.starts_with("--") || matches!(s.as_str(), "--help" | "--version"))
        .cloned()
        .unwrap_or_default();
    if !action.is_empty() {
        input.next();
    }
    let executable = env::current_exe().map_err(|e| e.to_string())?;
    let default = if action != "migrate"
        && executable
            .file_name()
            .is_some_and(|n| n == "inkscape-mcp" || n == "inkscape-mcp-launcher")
        && executable
            .parent()
            .is_some_and(|p| p.file_name().is_some_and(|n| n == "bin"))
    {
        executable.parent().unwrap().parent().unwrap().to_path_buf()
    } else {
        PathBuf::from(env::var_os("HOME").ok_or("HOME missing")?)
            .join("Library/Application Support/inkscape-mcp")
    };
    let mut options = Options {
        action,
        client: None,
        root: default,
        source: None,
        preparation: None,
        parent_pid: None,
        channel: None,
        release: None,
        component: Component::Both,
        check: false,
        json: false,
    };
    let mut seen = std::collections::BTreeSet::new();
    let mut component = false;
    while let Some(key) = input.next() {
        if !seen.insert(key.clone()) {
            return Err(format!("duplicate option: {key}"));
        }
        match key.as_str() {
            "--install-dir" | "--source" | "--channel" | "--release" | "--client"
            | "--preparation" | "--parent-pid" => {
                let value = input
                    .next()
                    .filter(|v| !v.starts_with("--"))
                    .ok_or_else(|| format!("missing value for {key}"))?;
                match key.as_str() {
                    "--install-dir" => options.root = value.into(),
                    "--client" => options.client = Some(value),
                    "--source" => options.source = Some(value.into()),
                    "--preparation" => options.preparation = Some(value),
                    "--parent-pid" => {
                        options.parent_pid = Some(value.parse().map_err(|_| "invalid parent PID")?)
                    }
                    "--channel" => options.channel = Some(value),
                    _ => options.release = Some(value),
                }
            }
            "--instructions" | "--runtime" => {
                if component {
                    return Err("choose one component, or omit both for a combined update".into());
                }
                component = true;
                options.component = if key == "--runtime" {
                    Component::Runtime
                } else {
                    Component::Instructions
                };
            }
            "--check" => options.check = true,
            "--json" => options.json = true,
            _ => return Err(format!("unknown option: {key}; see --help")),
        }
    }
    if !matches!(
        options.action.as_str(),
        "" | "--help"
            | "--version"
            | "update"
            | "rollback"
            | "migrate"
            | "disconnect"
            | "uninstall"
            | "install-inspect"
            | "install-prepare"
            | "install-activate"
            | "install-recover"
            | "manager-prepare"
            | "manager-activate"
    ) || !options.root.is_absolute()
        || options
            .channel
            .as_ref()
            .is_some_and(|c| !matches!(c.as_str(), "stable" | "prerelease"))
    {
        return Err("invalid command, channel or installation directory; see --help".into());
    }
    if options.action != "update" && (component || options.check || options.release.is_some())
        || !matches!(options.action.as_str(), "migrate" | "install-inspect")
            && options.source.is_some()
        || !matches!(options.action.as_str(), "update" | "migrate") && options.channel.is_some()
    {
        return Err("option does not apply to this command".into());
    }
    if options.preparation.is_some()
        && !matches!(
            options.action.as_str(),
            "install-activate" | "manager-activate"
        )
    {
        return Err("--preparation only applies to install-activate".into());
    }
    if options.parent_pid.is_some() && options.action != "manager-activate" {
        return Err("--parent-pid only applies to manager-activate".into());
    }
    Ok(options)
}
fn os_major() -> Result<u32> {
    let output = std::process::Command::new("/usr/bin/sw_vers")
        .arg("-productVersion")
        .output()
        .map_err(|e| e.to_string())?;
    String::from_utf8(output.stdout)
        .map_err(|e| e.to_string())?
        .split('.')
        .next()
        .ok_or("OS version missing")?
        .parse()
        .map_err(|_| "OS version invalid".into())
}
fn run(options: &Options) -> Result<Value> {
    let root = &options.root;
    use inkscape_mcp_rust::update::installer;
    match options.action.as_str() {
        "manager-prepare" => return inkscape_mcp_rust::update::manager_app::prepare(root),
        "manager-activate" => {
            return inkscape_mcp_rust::update::manager_app::activate(
                root,
                options
                    .preparation
                    .as_deref()
                    .ok_or("Manager activation requires --preparation")?,
                options
                    .parent_pid
                    .ok_or("Manager activation requires --parent-pid")?,
            );
        }
        "install-inspect" => {
            return match &options.source {
                Some(source) => installer::inspect_legacy(root, source),
                None => installer::inspect(root),
            };
        }
        "install-recover" => return installer::recover(root),
        "install-activate" => {
            return installer::activate(
                root,
                options
                    .preparation
                    .as_deref()
                    .ok_or("activation requires --preparation")?,
            );
        }
        "install-prepare" => {
            if std::env::consts::OS != "macos"
                || std::env::consts::ARCH != "aarch64"
                || os_major()? < 15
            {
                return Err("native installation requires Apple Silicon macOS 15 or newer".into());
            }
            use std::io::Read;
            let mut bytes = Vec::new();
            std::io::stdin()
                .take(16385)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() > 16384 {
                return Err("installation request exceeds 16 KiB".into());
            }
            let request = serde_json::from_slice(&bytes)
                .map_err(|e| format!("invalid installation request: {e}"))?;
            return installer::prepare(root, request);
        }
        _ => {}
    }
    if options.action == "disconnect" {
        inkscape_mcp_rust::client_management::disconnect_binding(
            options
                .client
                .as_deref()
                .ok_or("disconnect requires --client codex|claude")?,
            &root.join("bin/inkscape-mcp-launcher"),
        )
        .map_err(|e| e.to_string())?;
        return Ok(json!({"disconnected":true,"client_reconnect_required":true}));
    }
    if options.client.is_some() {
        return Err("--client only applies to disconnect".into());
    }

    if options.action == "--help" {
        println!("{HELP}");
        return Ok(Value::Null);
    }
    if options.action == "migrate" {
        return manager::migrate(
            root,
            options
                .source
                .as_deref()
                .ok_or("migrate requires --source")?,
            options.channel.as_deref().unwrap_or("stable"),
        );
    }
    if options.action == "update" {
        return manager::update(
            root,
            &Https::new()?,
            options.component,
            options.check,
            options.channel.as_deref(),
            options.release.as_deref(),
            os_major()?,
        );
    }
    if options.action == "--version" {
        return Ok(report(&selector(root)?));
    }
    let lock = if options.action.is_empty() {
        Lock::acquire_wait(root, std::time::Duration::from_secs(2))?
    } else {
        Lock::acquire(root)?
    };
    installer::recover_before_launch(root)?;
    let settings: Settings = inkscape_mcp_rust::update::storage::json(&root.join("settings.json"))?;
    settings.validate()?;
    if transaction::recover(root, &settings)? {
        eprintln!("Restored the preceding installation after an interrupted update.");
    }
    if options.action == "uninstall" {
        let clients: Vec<String> = inkscape_mcp_rust::update::storage::json(
            &root.join("backups/legacy-configuration/clients.json"),
        )?;
        inkscape_mcp_rust::client_management::disconnect_owned_bindings(
            &clients,
            &root.join("bin/inkscape-mcp-launcher"),
        )
        .map_err(|e| e.to_string())?;
        for (index, skill) in settings.skills.iter().enumerate() {
            if !skill.exists() {
                continue;
            }
            let owner = String::from_utf8(inkscape_mcp_rust::update::instructions::read(
                &skill.join(".inkscape-mcp-owner"),
                8192,
            )?)
            .map_err(|e| e.to_string())?;
            if owner.trim() == root.to_string_lossy()
                || owner.trim() == settings.legacy_skill_owner.to_string_lossy()
            {
                inkscape_mcp_rust::update::storage::inventory(skill, 16 * 1024 * 1024)?;
                std::fs::rename(
                    skill,
                    root.join("backups")
                        .join(format!("removed-skill-{index}-{}", uuid::Uuid::new_v4())),
                )
                .map_err(|e| e.to_string())?;
            }
        }
        let backup = root
            .parent()
            .ok_or("missing installation parent")?
            .join(format!("inkscape-mcp-removed-{}", uuid::Uuid::new_v4()));
        std::fs::rename(root, &backup).map_err(|e| e.to_string())?;
        sync_dir(backup.parent().unwrap())?;
        return Ok(json!({"uninstalled":true,"archive":backup,"client_reconnect_required":true}));
    }
    if options.action == "rollback" {
        if let Some(result) = installer::rollback_if_selected(root)? {
            return Ok(result);
        }
        let activation = transaction::rollback(root, &settings)?;
        let changed = activation.changed;
        let mut result = report(&selector(root)?);
        result["changed"] = json!(changed);
        result["client_reconnect_required"] = json!(changed);
        result["skill_reload_required"] = json!(activation.skills_changed);
        result["skipped_skills"] = json!(activation.skipped_skills);
        return Ok(result);
    }
    let pair = selector(root)?.current;
    // Recovery and the selector/settings snapshot require serialization. Runtime
    // and instruction trees are immutable; hashing them must not starve launches.
    drop(lock);
    let mut command = command(root, &pair, &settings)?;
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec().to_string())
    }
    #[cfg(not(unix))]
    {
        let status = command.status().map_err(|e| e.to_string())?;
        std::process::exit(status.code().unwrap_or(1));
    }
}
fn main() {
    let options = match parse() {
        Ok(o) => o,
        Err(error) => {
            if env::args().any(|a| a == "--json") {
                println!("{}", json!({"ok":false,"error":error,"exit_code":2}));
            } else {
                eprintln!("{error}");
            }

            std::process::exit(2);
        }
    };
    match run(&options) {
        Ok(value) if value.is_null() => {}
        Ok(value) => {
            if options.json {
                println!("{value}");
            } else {
                println!("{}", serde_json::to_string_pretty(&value).unwrap());
                if value["skipped_skills"]
                    .as_array()
                    .is_some_and(|paths| !paths.is_empty())
                {
                    eprintln!(
                        "Skipped removed or unchanged skill destinations: {}",
                        value["skipped_skills"]
                    );
                }
                if value["client_reconnect_required"] == true {
                    eprintln!(
                        "Activated. Reconnect your MCP client; reload its skill when indicated."
                    );
                }
            }
        }
        Err(error) => {
            if options.json {
                println!("{}", json!({"ok":false,"error":error}));
            } else {
                eprintln!("{error}");
            }
            std::process::exit(1);
        }
    }
}
