//! Isolated native installer acceptance. Client profiles are synthetic; MCP/doctor are real.
use crate::common::*;
use inkscape_mcp_rust::update::{instructions, storage};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
fn invoke(
    launcher: &Path,
    home: &Path,
    root: &Path,
    args: &[&str],
    request: Option<&Value>,
) -> Result<Value> {
    let mut command = Command::new(launcher);
    command
        .args(args)
        .args(["--install-dir", root.to_str().unwrap(), "--json"])
        .env_clear()
        .env("HOME", home)
        .env("CODEX_HOME", home.join(".codex"))
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", home.join("bin").display()),
        );
    if let Some(request) = request {
        let file = home.join("request.json");
        write_json(&file, request)?;
        command.stdin(Stdio::from(fs::File::open(file)?));
    } else {
        command.stdin(Stdio::null());
    }
    let (status, stdout, _stderr) = crate::acceptance::capture_deadline(&mut command, 120)?;
    ensure(
        stdout.len() <= 1024 * 1024,
        "installer reply exceeds byte budget",
    )?;
    let reply: Value = serde_json::from_str(&stdout)?;
    ensure(
        status == 0 && reply["ok"] != false,
        reply["error"]
            .as_str()
            .unwrap_or("installer invocation failed"),
    )?;
    Ok(reply)
}
fn refuse(
    launcher: &Path,
    home: &Path,
    root: &Path,
    request: &Value,
    expected: &str,
) -> Result<()> {
    let error = invoke(launcher, home, root, &["install-prepare"], Some(request))
        .err()
        .ok_or("installation refusal unexpectedly succeeded")?;
    ensure(
        error.to_string().contains(expected),
        "installation failed for an unrelated reason",
    )
}
fn custom_engine_acceptance(
    launcher: &Path,
    output_dir: &Path,
    request: &Value,
    inkscape: &Path,
) -> Result<()> {
    let home = output_dir.join("custom-engine-home");
    fs::create_dir_all(home.join(".codex"))?;
    fs::create_dir(home.join("bin"))?;
    for client in ["codex", "claude"] {
        fs::copy(
            output_dir.join("home/bin").join(client),
            home.join("bin").join(client),
        )?;
    }
    let root = home.join("installation");
    let workspace = home.join("drawings");
    fs::create_dir(&workspace)?;
    fs::write(
        workspace.join("original.svg"),
        b"preserved custom engine drawing",
    )?;
    let old = home.join("custom engine");
    let new = home.join("moved engine");
    let relative_engine = if cfg!(target_os = "macos") {
        Path::new("Contents/MacOS/inkscape")
    } else {
        Path::new("inkscape")
    };
    let executable = old.join(relative_engine);
    fs::create_dir_all(executable.parent().unwrap())?;
    // Fixed compiled adapter to the actual approved headless engine. This tests
    // saved-path semantics, not another copy's quarantine/first-open acceptance.
    let source = home.join("engine-adapter.c");
    let original = serde_json::to_string(inkscape.to_str().ok_or("invalid engine path")?)?;
    fs::write(
        &source,
        format!(
            "#include <unistd.h>\nint main(int argc,char **argv){{(void)argc;argv[0]={original};execv(argv[0],argv);return 127;}}\n"
        ),
    )?;
    output(
        Command::new(tool("cc")?)
            .arg(&source)
            .arg("-o")
            .arg(&executable),
    )?;
    if cfg!(target_os = "macos") {
        let resources = inkscape
            .parent()
            .and_then(Path::parent)
            .ok_or("Inkscape application layout missing")?
            .join("Resources");
        let gtk = old.join("Contents/Resources/lib/libgtk-3.0.dylib");
        fs::create_dir_all(gtk.parent().unwrap())?;
        fs::copy(resources.join("lib/libgtk-3.0.dylib"), gtk)?;
    }
    let mut selected = request.clone();
    selected["workspace"] = json!(workspace);
    selected["inkscape"] = json!(old.join(relative_engine));
    let prepared = invoke(
        launcher,
        &home,
        &root,
        &["install-prepare"],
        Some(&selected),
    )?;
    invoke(
        launcher,
        &home,
        &root,
        &[
            "install-activate",
            "--preparation",
            prepared["preparation_id"].as_str().unwrap(),
        ],
        None,
    )?;
    let observed = invoke(launcher, &home, &root, &["install-inspect"], None)?;
    ensure(
        observed["inkscape"] == json!(old.join(relative_engine)),
        "custom engine inspection used a generic system path",
    )?;
    let before: inkscape_mcp_rust::update::install::Settings =
        storage::json(&root.join("settings.json"))?;
    let prepared = invoke(
        launcher,
        &home,
        &root,
        &["install-prepare"],
        Some(&selected),
    )?;
    let noop = invoke(
        launcher,
        &home,
        &root,
        &[
            "install-activate",
            "--preparation",
            prepared["preparation_id"].as_str().unwrap(),
        ],
        None,
    )?;
    ensure(
        noop["changed"] == false,
        "upgrade replaced a usable saved custom engine",
    )?;
    let active = fs::read(root.join("active.json"))?;
    let config = fs::read(home.join(".codex/config.toml"))?;
    fs::rename(&old, &new)?;
    selected["inkscape"] = json!(new.join(relative_engine));
    let prepared = invoke(
        launcher,
        &home,
        &root,
        &["install-prepare"],
        Some(&selected),
    )?;
    invoke(
        launcher,
        &home,
        &root,
        &[
            "install-activate",
            "--preparation",
            prepared["preparation_id"].as_str().unwrap(),
        ],
        None,
    )?;
    let repaired: inkscape_mcp_rust::update::install::Settings =
        storage::json(&root.join("settings.json"))?;
    ensure(
        std::env::split_paths(std::ffi::OsStr::new(&repaired.environment["PATH"])).next()
            == Some(new.join(relative_engine).parent().unwrap().to_owned()),
        "missing engine selection did not repair saved PATH",
    )?;
    let mut expected = serde_json::to_value(&before)?;
    expected["environment"]["PATH"] = json!(repaired.environment["PATH"]);
    ensure(
        serde_json::to_value(&repaired)? == expected
            && fs::read(root.join("active.json"))? == active
            && fs::read(home.join(".codex/config.toml"))? == config
            && fs::read(workspace.join("original.svg"))? == b"preserved custom engine drawing",
        "engine repair changed other settings, selection, client or artwork",
    )?;
    ensure(
        invoke(launcher, &home, &root, &["install-inspect"], None)?["inkscape"]
            == json!(new.join(relative_engine)),
        "repaired engine is not visible on reopen",
    )
}
fn legacy_acceptance(
    launcher: &Path,
    output_dir: &Path,
    request: &Value,
    inkscape: &Path,
) -> Result<()> {
    let home = output_dir.join("legacy-home");
    fs::create_dir_all(home.join(".codex"))?;
    fs::create_dir(home.join("bin"))?;
    for client in ["codex", "claude"] {
        fs::copy(
            output_dir.join("home/bin").join(client),
            home.join("bin").join(client),
        )?;
    }
    let source = output_dir.join("legacy source");
    fs::create_dir_all(source.join(".inkscape-mcp-local"))?;
    fs::create_dir(source.join("bin"))?;
    let old_command = source.join("run-mcp.sh");
    fs::write(
        &old_command,
        b"unverified legacy launcher; must never execute\n",
    )?;
    fs::write(source.join("bin/inkscape-mcp"), b"unverified old binary\n")?;
    let saved_workspace = output_dir.join("legacy drawings");
    fs::create_dir(&saved_workspace)?;
    fs::write(
        saved_workspace.join("original.svg"),
        b"preserved legacy drawing",
    )?;
    let setup = format!(
        "inkscape-mcp-setup-v1\n{}\n{}\n{}\nfalse\nper_call\n",
        source.join("bin/inkscape-mcp").display(),
        inkscape.parent().unwrap().display(),
        saved_workspace.display()
    );
    fs::write(source.join(".inkscape-mcp-local/setup.conf"), &setup)?;
    fs::write(
        home.join(".codex/config.toml"),
        format!(
            "# legacy preference\nmodel = 'kept'\n[mcp_servers.inkscape]\ncommand = {}\nargs = []\n",
            serde_json::to_string(&old_command)?
        ),
    )?;
    write_json(
        &home.join(".claude.json"),
        &json!({"theme":"legacy-kept","mcpServers":{"inkscape":{"command":old_command,"args":[]}}}),
    )?;
    let root = output_dir.join("custom permanent root");
    let discovery = invoke(launcher, &home, &root, &["install-inspect"], None)?;
    ensure(
        discovery["state"] == "legacy_found" && discovery["candidates"] == json!([source]),
        "legacy discovery failed or executed the old launcher",
    )?;
    let mut migration = request.clone();
    migration["legacy_source"] = json!(source);
    let before_codex = fs::read(home.join(".codex/config.toml"))?;
    let before_claude = fs::read(home.join(".claude.json"))?;
    let other_source = output_dir.join("second legacy source");
    fs::create_dir_all(other_source.join(".inkscape-mcp-local"))?;
    fs::write(other_source.join(".inkscape-mcp-local/setup.conf"), &setup)?;
    fs::write(
        other_source.join("run-mcp.sh"),
        b"second unverified source\n",
    )?;
    write_json(
        &home.join(".claude.json"),
        &json!({"mcpServers":{"inkscape":{"command":other_source.join("run-mcp.sh"),"args":[]}}}),
    )?;
    let multiple = invoke(launcher, &home, &root, &["install-inspect"], None)?;
    ensure(
        multiple["choice_required"] == true
            && multiple["candidates"].as_array().unwrap().len() == 2,
        "multiple legacy candidates were silently adopted",
    )?;
    fs::write(home.join(".claude.json"), &before_claude)?;
    let prepared = invoke(
        launcher,
        &home,
        &root,
        &["install-prepare"],
        Some(&migration),
    )?;
    ensure(
        fs::read(home.join(".codex/config.toml"))? == before_codex
            && fs::read(home.join(".claude.json"))? == before_claude,
        "legacy preparation switched bindings before activation",
    )?;
    invoke(
        launcher,
        &home,
        &root,
        &[
            "install-activate",
            "--preparation",
            prepared["preparation_id"].as_str().unwrap(),
        ],
        None,
    )?;
    let settings: inkscape_mcp_rust::update::install::Settings =
        storage::json(&root.join("settings.json"))?;
    ensure(
        settings.environment["INKSCAPE_MCP_WORKSPACE_ROOTS"] == saved_workspace.to_str().unwrap()
            && settings.environment["INKSCAPE_MCP_LIVE_ENABLED"] == "false"
            && fs::read_to_string(source.join(".inkscape-mcp-local/setup.conf"))? == setup
            && fs::read(&old_command)? == b"unverified legacy launcher; must never execute\n"
            && fs::read(saved_workspace.join("original.svg"))? == b"preserved legacy drawing",
        "legacy settings/source/drawing preservation failed",
    )?;
    let inspect = invoke(launcher, &home, &root, &["install-inspect"], None)?;
    ensure(
        inspect["state"] == "installed" && inspect["candidates"] == json!([root]),
        "custom-root migration did not switch both bindings",
    )?;
    ensure(
        json(&home.join(".claude.json"))?["theme"] == "legacy-kept"
            && fs::read_to_string(home.join(".codex/config.toml"))?.contains("# legacy preference"),
        "legacy client preferences lost",
    )?;
    Ok(())
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--package", "--output", "--manager-app"])?;
    let package = args.required("--package")?.canonicalize()?;
    let output_dir = args.required("--output")?;
    fs::create_dir(&output_dir)?;
    let output_dir = output_dir.canonicalize()?;
    let home = output_dir.join("home");
    fs::create_dir_all(home.join("bin"))?;
    let root = output_dir.join("installation");
    let workspace = output_dir.join("drawings");
    fs::create_dir(&workspace)?;
    fs::write(workspace.join("original.svg"), "preserved original")?;
    for name in ["codex", "claude"] {
        use std::os::unix::fs::PermissionsExt;
        fs::write(
            home.join("bin").join(name),
            "synthetic presence fixture; never executed",
        )?;
        fs::set_permissions(
            home.join("bin").join(name),
            fs::Permissions::from_mode(0o700),
        )?;
    }
    let instruction_dir = output_dir.join("instructions");
    fs::create_dir(&instruction_dir)?;
    let files = instructions::default_files();
    let manifest = instructions::manifest("installer-test", &files);
    for (name, bytes) in files {
        let path = instruction_dir.join(name);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, bytes)?;
    }
    write_json(
        &instruction_dir.join("manifest.json"),
        &serde_json::to_value(manifest)?,
    )?;
    let inkscape = if cfg!(target_os = "macos") {
        PathBuf::from("/Applications/Inkscape.app/Contents/MacOS/inkscape")
    } else {
        tool("inkscape")?
    };
    let runtime_launcher = inkscape_mcp_rust::runtime_layout::binary(
        &inkscape_mcp_rust::runtime_layout::library(&package),
        "inkscape-mcp-launcher",
    );
    let launcher = if let Some(app) = args.values.get("--manager-app") {
        let app = Path::new(app).canonicalize()?;
        let helper = app.join("Contents/Helpers/inkscape-mcp-launcher");
        let team = inkscape_mcp_rust::client_management::signature_team(&std::env::current_exe()?)
            .map_err(|e| e.to_string())?;
        ensure(
            team.is_some()
                && inkscape_mcp_rust::client_management::signature_team(&helper)
                    .map_err(|e| e.to_string())?
                    == team,
            "Manager acceptance requires a verified matching Developer ID helper",
        )?;
        output(
            Command::new("/usr/bin/codesign")
                .args(["--verify", "--deep", "--strict"])
                .arg(&app),
        )?;
        fs::remove_dir_all(&instruction_dir)?;
        inkscape_mcp_rust::update::storage::copy_tree(
            &app.join("Contents/Resources/instructions"),
            &instruction_dir,
            32 * 1024 * 1024,
        )?;
        helper
    } else {
        runtime_launcher
    };
    let request = json!({"package":package,"instructions":instruction_dir,"workspace":workspace,"inkscape":inkscape,"clients":["codex","claude"],"install_skills":true,"channel":"prerelease"});
    ensure(
        invoke(&launcher, &home, &root, &["install-inspect"], None)?["state"] == "not_installed",
        "fresh state differs",
    )?;
    fs::create_dir(home.join(".codex"))?;
    fs::write(
        home.join(".codex/config.toml"),
        "# preserved comment\nmodel = 'fixture'\n[mcp_servers.other]\ncommand = '/preserved'\n",
    )?;
    fs::write(
        home.join(".claude.json"),
        "{\"theme\":\"preserved\",\"mcpServers\":{\"other\":{\"command\":\"/preserved\"}}}",
    )?;
    let codex_before = fs::read(home.join(".codex/config.toml"))?;
    let claude_before = fs::read(home.join(".claude.json"))?;
    let ownership_refusal_exercised = unsafe { libc::geteuid() } != 0;
    if ownership_refusal_exercised {
        let foreign_workspace = Path::new("/etc").canonicalize()?;
        let mut foreign = request.clone();
        foreign["workspace"] = json!(foreign_workspace);
        refuse(
            &launcher,
            &home,
            &root,
            &foreign,
            "managed file must be user-owned",
        )?;
    }
    let mut overlapping = request.clone();
    overlapping["workspace"] = json!(output_dir);
    refuse(
        &launcher,
        &home,
        &root,
        &overlapping,
        "installation overlaps package, instructions, workspace or Inkscape",
    )?;
    fs::rename(home.join("bin/claude"), home.join("bin/claude-unavailable"))?;
    let missing = invoke(&launcher, &home, &root, &["install-inspect"], None)?;
    let missing_client_exercised = missing["clients"]
        .as_array()
        .unwrap()
        .iter()
        .any(|client| client["name"] == "claude" && client["available"] == false);
    if missing_client_exercised {
        refuse(
            &launcher,
            &home,
            &root,
            &request,
            "claude client is missing",
        )?;
        ensure(
            !root.join("active.json").exists()
                && fs::read(home.join(".codex/config.toml"))? == codex_before
                && fs::read(home.join(".claude.json"))? == claude_before,
            "missing client refusal changed bindings",
        )?;
    }
    fs::rename(home.join("bin/claude-unavailable"), home.join("bin/claude"))?;
    let mut foreign: Value = serde_json::from_slice(&claude_before)?;
    foreign["mcpServers"]["inkscape"] = json!({"command":"/foreign/inkscape","args":[]});
    write_json(&home.join(".claude.json"), &foreign)?;
    let foreign_bytes = fs::read(home.join(".claude.json"))?;
    let discovered = invoke(&launcher, &home, &root, &["install-inspect"], None)?;
    ensure(
        discovered["clients"]
            .as_array()
            .unwrap()
            .iter()
            .any(|client| client["name"] == "claude" && client["foreign_binding"] == true),
        "foreign client binding was adopted during discovery",
    )?;
    refuse(
        &launcher,
        &home,
        &root,
        &request,
        "client binding belongs to another installation",
    )?;
    ensure(
        fs::read(home.join(".claude.json"))? == foreign_bytes && !root.join("active.json").exists(),
        "foreign binding refusal changed activation",
    )?;
    fs::write(home.join(".claude.json"), &claude_before)?;
    let mut unknown = request.clone();
    unknown["execute"] = json!("forbidden");
    ensure(
        invoke(
            &launcher,
            &home,
            &root,
            &["install-prepare"],
            Some(&unknown),
        )
        .is_err(),
        "unknown installation action accepted",
    )?;
    let linked_workspace = output_dir.join("linked-workspace");
    std::os::unix::fs::symlink(&workspace, &linked_workspace)?;
    let mut linked = request.clone();
    linked["workspace"] = json!(linked_workspace);
    ensure(
        invoke(&launcher, &home, &root, &["install-prepare"], Some(&linked)).is_err(),
        "symlink workspace accepted",
    )?;
    ensure(
        !root.join("active.json").exists()
            && fs::read(home.join(".codex/config.toml"))? == codex_before
            && fs::read(home.join(".claude.json"))? == claude_before,
        "refusal changed client bindings",
    )?;
    let prepared = invoke(
        &launcher,
        &home,
        &root,
        &["install-prepare"],
        Some(&request),
    )?;
    ensure(
        !root.join("active.json").exists()
            && fs::read(home.join(".codex/config.toml"))? == codex_before
            && fs::read(home.join(".claude.json"))? == claude_before,
        "preparation modified activation/configuration",
    )?;
    let plan = json(&root.join("installation-prepared.json"))?;
    let changes = plan["changes"].as_array().ok_or("changes missing")?;
    let base = root.join("backups").join(plan["id"].as_str().unwrap());
    // Simulate process termination after every atomic file-publication boundary.
    // Recovery uses the real native CLI with this isolated home, twice per boundary.
    for boundary in 0..=changes.len() {
        write_json(&root.join("installation-journal.json"), &plan)?;
        for (i, change) in changes.iter().enumerate().take(boundary) {
            let destination = Path::new(change["destination"].as_str().unwrap());
            fs::create_dir_all(destination.parent().unwrap())?;
            fs::copy(base.join(format!("after-{i}")), destination)?;
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                destination,
                fs::Permissions::from_mode(change["mode"].as_u64().unwrap() as u32),
            )?;
        }
        ensure(
            invoke(&launcher, &home, &root, &["install-recover"], None)?["changed"] == true,
            "recovery did not run",
        )?;
        ensure(
            invoke(&launcher, &home, &root, &["install-recover"], None)?["changed"] == false,
            "recovery is not idempotent",
        )?;
        ensure(
            !root.join("active.json").exists()
                && fs::read(home.join(".codex/config.toml"))? == codex_before
                && fs::read(home.join(".claude.json"))? == claude_before,
            "interruption failed to preserve original configuration",
        )?;
    }
    let id = prepared["preparation_id"].as_str().unwrap();
    let activated = invoke(
        &launcher,
        &home,
        &root,
        &["install-activate", "--preparation", id],
        None,
    )?;
    ensure(
        activated["state"] == "completed",
        "fresh activation did not complete",
    )?;
    let inspected = invoke(&launcher, &home, &root, &["install-inspect"], None)?;
    ensure(
        inspected["state"] == "installed"
            && inspected["installed"]["launcher_build"] == activated["launcher_build"],
        "reopened state differs",
    )?;
    // A native picker displays one existing root while upgrades retain every saved root.
    let settings_path = root.join("settings.json");
    let saved_settings = fs::read(&settings_path)?;
    let second_workspace = output_dir.join("additional drawings");
    fs::create_dir(&second_workspace)?;
    let mut multi_root = json(&settings_path)?;
    multi_root["environment"]["INKSCAPE_MCP_WORKSPACE_ROOTS"] = json!(
        std::env::join_paths([&workspace, &second_workspace])?
            .to_str()
            .ok_or("invalid workspace list")?
    );
    write_json(&settings_path, &multi_root)?;
    let multi_root_bytes = fs::read(&settings_path)?;
    let multi_inspected = invoke(&launcher, &home, &root, &["install-inspect"], None)?;
    ensure(
        multi_inspected["workspace"] == json!(workspace)
            && fs::read(&settings_path)? == multi_root_bytes,
        "inspection must display the first workspace without changing saved roots",
    )?;
    fs::write(&settings_path, saved_settings)?;
    let mut failed_probe_exercised = false;
    if !args.values.contains_key("--manager-app")
        && inkscape_mcp_rust::client_management::signature_team(&launcher)
            .map_err(|e| e.to_string())?
            .is_none()
    {
        // Deliberately mismatched development metadata reaches the real version
        // probe. Never mutate a Developer ID artifact to manufacture this fixture.
        let bad_package = output_dir.join("probe-mismatch-package");
        storage::copy_tree(&package, &bad_package, 1024 * 1024 * 1024)?;
        let metadata_path =
            inkscape_mcp_rust::runtime_layout::library(&bad_package).join("package.json");
        let mut metadata = json(&metadata_path)?;
        metadata["build_info"]["build_id"] = json!("probe-mismatch");
        write_json(&metadata_path, &metadata)?;
        let app = bad_package.join(inkscape_mcp_rust::runtime_layout::RUNTIME_APP);
        if app.exists() {
            output(
                Command::new("/usr/bin/codesign")
                    .args([
                        "--force",
                        "--sign",
                        "-",
                        "--timestamp=none",
                        "--options",
                        "runtime",
                    ])
                    .arg(app),
            )?;
        }
        let mut inventory = storage::inventory(&bad_package, 1024 * 1024 * 1024)?;
        inventory.remove("FILES.json");
        write_json(&bad_package.join("FILES.json"), &json!(inventory))?;
        let mut mismatch = request.clone();
        mismatch["package"] = json!(bad_package);
        let selected = fs::read(root.join("active.json"))?;
        let codex = fs::read(home.join(".codex/config.toml"))?;
        let claude = fs::read(home.join(".claude.json"))?;
        refuse(
            &launcher,
            &home,
            &root,
            &mismatch,
            "candidate executable identity differs from manifest",
        )?;
        ensure(
            fs::read(root.join("active.json"))? == selected
                && fs::read(home.join(".codex/config.toml"))? == codex
                && fs::read(home.join(".claude.json"))? == claude,
            "failed probe changed selection or client bindings",
        )?;
        failed_probe_exercised = true;
    }
    let before_component = json(&root.join("active.json"))?;
    let mut interrupted_component = before_component.clone();
    interrupted_component["current"]["tag"] = json!("interrupted-fixture");
    let component_backup = root.join("backups/component-fixture");
    fs::create_dir(&component_backup)?;
    write_json(
        &component_backup.join("selector-before.json"),
        &before_component,
    )?;
    write_json(
        &root.join("transaction.json"),
        &json!({"format":1,"id":"component-fixture","before":before_component,"after":interrupted_component,"skills":[]}),
    )?;
    write_json(&root.join("active.json"), &interrupted_component)?;
    ensure(
        invoke(&launcher, &home, &root, &["install-inspect"], None)?["recovery_required"] == true,
        "component interruption is hidden from Manager discovery",
    )?;
    ensure(
        invoke(&launcher, &home, &root, &["install-recover"], None)?["changed"] == true
            && invoke(&launcher, &home, &root, &["install-recover"], None)?["changed"] == false
            && json(&root.join("active.json"))? == before_component,
        "Manager recovery failed to restore interrupted component selection idempotently",
    )?;
    let codex = fs::read_to_string(home.join(".codex/config.toml"))?;
    ensure(
        codex.contains("preserved comment") && codex.contains("/preserved"),
        "Codex preferences/comments lost",
    )?;
    ensure(
        json(&home.join(".claude.json"))?["theme"] == "preserved",
        "Claude preference lost",
    )?;
    let skill = home.join(".codex/skills/inkscape-mcp");
    let selector_before_conflict = fs::read(root.join("active.json"))?;
    let local_skill = fs::read(skill.join("SKILL.md"))?;
    let incoming_skill = instruction_dir.join("skills/inkscape-mcp/SKILL.md");
    let incoming_before = fs::read(&incoming_skill)?;
    let instructions_before = fs::read(instruction_dir.join("manifest.json"))?;
    fs::write(skill.join("SKILL.md"), b"user changed this entire skill\n")?;
    fs::write(&incoming_skill, b"upstream changed this entire skill\n")?;
    let mut changed: inkscape_mcp_rust::update::manifests::InstructionManifest =
        serde_json::from_slice(&instructions_before)?;
    changed.version = "conflicting-skill".into();
    changed.files.insert(
        "skills/inkscape-mcp/SKILL.md".into(),
        inkscape_mcp_rust::update::manifests::FileIdentity::of(&fs::read(&incoming_skill)?),
    );
    changed.content_id = inkscape_mcp_rust::update::manifests::content_id(&changed.files);
    write_json(&instruction_dir.join("manifest.json"), &json!(changed))?;
    refuse(&launcher, &home, &root, &request, "skill merge conflict")?;
    ensure(
        fs::read(root.join("active.json"))? == selector_before_conflict
            && fs::read(skill.join("SKILL.md"))? == b"user changed this entire skill\n",
        "skill conflict changed the selected installation or user customization",
    )?;
    fs::write(skill.join("SKILL.md"), local_skill)?;
    fs::write(incoming_skill, incoming_before)?;
    fs::write(instruction_dir.join("manifest.json"), instructions_before)?;
    fs::write(skill.join("extra.txt"), "customized")?;
    fs::remove_dir_all(home.join(".claude/skills/inkscape-mcp"))?;
    let mut retry = request.clone();
    let mut relabeled: inkscape_mcp_rust::update::manifests::InstructionManifest =
        storage::json(&instruction_dir.join("manifest.json"))?;
    relabeled.version = "installer-retry".into();
    write_json(&instruction_dir.join("manifest.json"), &json!(relabeled))?;
    retry["install_skills"] = json!(false);
    retry["channel"] = json!("stable");
    let prepared = invoke(&launcher, &home, &root, &["install-prepare"], Some(&retry))?;
    invoke(
        &launcher,
        &home,
        &root,
        &[
            "install-activate",
            "--preparation",
            prepared["preparation_id"].as_str().unwrap(),
        ],
        None,
    )?;
    ensure(
        fs::read_to_string(skill.join("extra.txt"))? == "customized"
            && !home.join(".claude/skills/inkscape-mcp").exists(),
        "retry lost customization or recreated removed skill",
    )?;
    ensure(
        invoke(&launcher, &home, &root, &["--version"], None)?["channel"] == "prerelease",
        "upgrade changed saved channel",
    )?;
    ensure(
        invoke(&launcher, &home, &root, &["install-inspect"], None)?["installed"]["instructions_version"]
            == "installer-retry",
        "equal-content instruction version upgrade failed",
    )?;
    ensure(
        fs::read_to_string(workspace.join("original.svg"))? == "preserved original",
        "original drawing changed",
    )?;
    let last_before_noop = fs::read(root.join("installation-last.json"))?;
    let active_before_noop = fs::read(root.join("active.json"))?;
    let noop = invoke(&launcher, &home, &root, &["install-prepare"], Some(&retry))?;
    ensure(
        noop["changes"].as_array().unwrap().is_empty(),
        "same installation produced nonempty changes",
    )?;
    let noop = invoke(
        &launcher,
        &home,
        &root,
        &[
            "install-activate",
            "--preparation",
            noop["preparation_id"].as_str().unwrap(),
        ],
        None,
    )?;
    ensure(
        noop["changed"] == false
            && noop["client_reconnect_required"] == false
            && fs::read(root.join("installation-last.json"))? == last_before_noop
            && fs::read(root.join("active.json"))? == active_before_noop,
        "no-op activation replaced rollback history or selected state",
    )?;
    legacy_acceptance(&launcher, &output_dir, &request, &inkscape)?;
    custom_engine_acceptance(&launcher, &output_dir, &request, &inkscape)?;
    let report = json!({"passed":true,"custom_engine_reused":true,"missing_engine_picker_repair":true,"publication_boundaries":changes.len()+1,"fresh_clients":["codex","claude"],"client_profiles":"synthetic","mcp_doctor_stdio":"real native package","native_gui":false,"installed_user_runtime_changed":false,"missing_client_exercised":missing_client_exercised,"failed_probe_exercised":failed_probe_exercised,"ownership_refusal_exercised":ownership_refusal_exercised,"foreign_binding_refused":true,"skill_conflict_preserved":true,"legacy_custom_root_migration":true,"component_recovery":true,"multi_root_inspection_preserved":true,"no_op_preserves_rollback":true,"build_info":json(&inkscape_mcp_rust::runtime_layout::library(&package).join("package.json"))?["build_info"],"results":["read-only discovery","unknown request, symlink workspace and overlap refused","foreign binding refused","skill conflict preserves selection/customization","multiple legacy candidates require choice","legacy migration into custom root","preparation preserves bindings","every file boundary recovers twice","fresh activation","byte-bound bootstrap discovery","component recovery through Manager operation","preferences/comments","customized/removed skills","equal-content instruction version upgrade","no-op activation preserves rollback history","retry","saved channel","original drawing preservation"]});
    // Ensure the final selector is verified independently of GUI status labels.
    let settings = storage::json(&root.join("settings.json"))?;
    inkscape_mcp_rust::update::install::probe(
        &root,
        &inkscape_mcp_rust::update::install::selector(&root)?.current,
        &settings,
    )?;
    write_json(&output_dir.join("report.json"), &report)?;
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}
