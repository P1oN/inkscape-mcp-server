use crate::{
    acceptance::{capture_full, copy_tree},
    common::*,
};
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};
const CLAUDE: &str = r##"#!/bin/sh
if [ "$1 $2" = "mcp add" ]; then
 [ "$3 $4 $5 $6 $7 $8" = "--transport stdio --scope user inkscape --" ] || exit 2
 printf '{"unrelated":{"keep":true},"mcpServers":{"inkscape":{"type":"stdio","command":"%s","args":[]}}}\n' "$9" > "$HOME/.claude.json"
elif [ "$*" = "mcp remove inkscape --scope user" ]; then
 printf '{"unrelated":{"keep":true},"mcpServers":{}}\n' > "$HOME/.claude.json"
else exit 2; fi
"##;
fn run_command(command: &mut Command, success: bool) -> Result<String> {
    let (status, stdout, stderr) = capture_full(command)?;
    ensure(
        (status == 0) == success,
        format!("client acceptance status differs: {stdout}\n{stderr}"),
    )?;
    Ok(stdout)
}
fn script(path: &Path, text: &str) -> Result<()> {
    fs::write(path, text)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--package", "--output"])?;
    let package = args.required("--package")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let codex = tool("codex")?;
    let inkscape = [
        "/Applications/Inkscape.app/Contents/MacOS/inkscape",
        "/usr/bin/inkscape",
    ]
    .iter()
    .find(|p| Path::new(p).is_file())
    .ok_or("installed Inkscape required (not executed by client checks)")?;
    let root = tempfile::tempdir()?;
    let root = root.path().canonicalize()?;
    let repo = root.join("ready package space");
    copy_tree(&package, &repo)?;
    let home = root.join("home");
    fs::create_dir(&home)?;
    let codex_home = root.join("custom codex home");
    fs::create_dir(&codex_home)?;
    let vendor = root.join("bin");
    fs::create_dir(&vendor)?;
    for name in ["dirname", "sed"] {
        std::os::unix::fs::symlink(tool(name)?, vendor.join(name))?;
    }
    std::os::unix::fs::symlink(codex, vendor.join("codex"))?;
    script(&vendor.join("claude"), CLAUDE)?;
    let env = [
        ("HOME", home.to_str().unwrap()),
        ("CODEX_HOME", codex_home.to_str().unwrap()),
        ("PATH", vendor.to_str().unwrap()),
        ("SENTRY_DSN", ""),
        ("SENTRY_TRACES_SAMPLE_RATE", "0"),
    ];
    let execute = |program: &Path, arguments: &[&str], success: bool| -> Result<String> {
        run_command(
            Command::new(program).args(arguments).env_clear().envs(env),
            success,
        )
    };
    let workspace = root.join("drawings");
    fs::create_dir(&workspace)?;
    let drawing = workspace.join("existing.svg");
    let original = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><rect id=\"original\"/></svg>";
    fs::write(&drawing, original)?;
    execute(
        &repo.join("setup.sh"),
        &[
            "--workspace",
            workspace.to_str().unwrap(),
            "--inkscape",
            inkscape,
            "--live",
            "false",
            "--sentry",
            "false",
        ],
        true,
    )?;
    let runtime = repo.join("libexec/inkscape-mcp");
    ensure(
        !runtime.join("python").exists(),
        "ready package bundles Python",
    )?;
    fs::write(runtime.join("python"), "damaged helper runtime")?;
    fs::write(
        repo.join("bin/inkscape-mcp-supervisor"),
        "damaged supervisor",
    )?;
    ensure(
        !repo.join("scripts/mcp_client.py").exists(),
        "Python manager bundled",
    )?;
    let manager = repo.join("scripts/mcp-client.sh");
    let configurations = [
        ("codex", codex_home.join("config.toml")),
        ("claude", home.join(".claude.json")),
    ];
    let client = |name: &str, action: &str, success: bool| {
        execute(&manager, &["--client", name, action], success)
    };
    for (name, path) in &configurations {
        fs::write(
            path,
            if *name == "codex" {
                "model = \"user-model\"\n"
            } else {
                "{\"unrelated\":{\"keep\":true}}"
            },
        )?;
        let before = fs::read(path)?;
        let snippet = client(name, "config", true)?;
        ensure(
            snippet.contains(repo.join("run-mcp.sh").to_str().unwrap())
                && fs::read(path)? == before,
            "snippet mutated settings",
        )?;
        for action in ["check", "connect"] {
            client(name, action, true)?;
        }
        let installed = fs::read(path)?;
        client(name, "connect", true)?;
        ensure(
            fs::read(path)? == installed,
            "idempotent connect mutated config",
        )?;
        ensure(
            if *name == "codex" {
                fs::read_to_string(path)?.contains("user-model")
            } else {
                crate::common::json(path)?["unrelated"]["keep"] == true
            },
            "unrelated settings lost",
        )?;
        fs::write(
            path,
            fs::read_to_string(path)?.replace(
                repo.join("run-mcp.sh").to_str().unwrap(),
                "/foreign/run-mcp.sh",
            ),
        )?;
        let foreign = fs::read(path)?;
        for action in ["connect", "disconnect", "uninstall"] {
            client(name, action, false)?;
        }
        ensure(fs::read(path)? == foreign, "foreign entry modified")?;
        execute(&repo.join("setup.sh"), &[], true)?;
        let updated = fs::read_to_string(path)?;
        ensure(
            updated.contains(repo.join("run-mcp.sh").to_str().unwrap())
                && !updated.contains("/foreign/run-mcp.sh"),
            "setup did not replace old registration",
        )?;
        ensure(
            if *name == "codex" {
                updated.contains("user-model")
            } else {
                crate::common::json(path)?["unrelated"]["keep"] == true
            },
            "setup replaced unrelated client settings",
        )?;
        fs::write(path, installed)?;
        client(name, "disconnect", true)?;
        client(name, "disconnect", true)?;
    }
    // Exercise the linked client record and failed handshake before either client is registered.
    let record = repo.join(".inkscape-mcp-local/clients.json");
    fs::remove_file(&record)?;
    let external = root.join("external.json");
    fs::write(&external, "[]")?;
    std::os::unix::fs::symlink(&external, &record)?;
    let before = fs::read(&configurations[0].1)?;
    client("codex", "connect", false)?;
    ensure(
        fs::read_to_string(&external)? == "[]" && fs::read(&configurations[0].1)? == before,
        "linked record mutated",
    )?;
    fs::remove_file(record)?;
    let server = repo.join("bin/inkscape-mcp");
    let held = root.join("held-server");
    fs::rename(&server, &held)?;
    script(&server, "#!/bin/sh\nexit 0\n")?;
    client("codex", "connect", false)?;
    ensure(
        fs::read(&configurations[0].1)? == before,
        "failed handshake changed config",
    )?;
    fs::remove_file(&server)?;
    fs::rename(held, server)?;
    for (name, _) in &configurations {
        client(name, "connect", true)?;
    }
    let local = repo.join(".inkscape-mcp-local");
    let saved = fs::read(local.join("setup.conf"))?;
    execute(&repo.join("uninstall.sh"), &["--client", "codex"], false)?;
    ensure(
        fs::read(local.join("setup.conf"))? == saved
            && fs::read_to_string(&configurations[0].1)?.contains("inkscape"),
        "blocked uninstall removed settings/client",
    )?;
    client("codex", "disconnect", true)?;
    fs::write(
        &configurations[1].1,
        fs::read_to_string(&configurations[1].1)?.replace(
            repo.join("run-mcp.sh").to_str().unwrap(),
            "/other/run-mcp.sh",
        ),
    )?;
    let switched = fs::read(&configurations[1].1)?;
    let skill = codex_home.join("skills/inkscape-mcp");
    fs::create_dir_all(&skill)?;
    fs::write(
        skill.join(".inkscape-mcp-owner"),
        format!("{}\n", repo.display()),
    )?;
    fs::write(skill.join("SKILL.md"), "user customizations")?;
    execute(&repo.join("uninstall.sh"), &["--client", "codex"], true)?;
    let backups: Vec<_> = fs::read_dir(&repo)?
        .filter_map(|row| row.ok())
        .map(|row| row.path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".inkscape-mcp-backup-")
        })
        .collect();
    ensure(
        backups.len() == 1 && !local.exists(),
        "uninstall failed to archive",
    )?;
    ensure(
        fs::read(backups[0].join("setup.conf"))? == saved
            && fs::read_to_string(backups[0].join("removed-skill/SKILL.md"))?
                == "user customizations"
            && fs::read(&configurations[1].1)? == switched,
        "archived skill/settings lost or switched client modified",
    )?;
    fs::rename(&backups[0], &local)?;
    fs::write(&configurations[1].1, "{\"unrelated\":{\"keep\":true}}")?;
    client("claude", "connect", true)?;
    execute(&repo.join("uninstall.sh"), &["--client", "claude"], true)?;
    ensure(fs::read(drawing)? == original, "existing drawing changed")?;
    write_json(
        &out.join("comparison.json"),
        &json!({"passed":true,"checks":8,"python_on_client_path":false,"helper_runtime":"damaged","codex_cli":"real isolated CODEX_HOME","claude_cli":"synthetic shell","native_GUI":false,"linked_record_refusal":true,"failed_handshake_refusal":true}),
    )?;
    println!("Client package lifecycle and source-management guards passed");
    Ok(())
}
