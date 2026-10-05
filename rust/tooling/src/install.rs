//! Git-free source installation in an isolated client profile; never launches the GUI.
use crate::{
    acceptance::capture_deadline,
    common::*,
    wire::{Wire, data},
};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--archive", "--output"])?;
    let archive = args.required("--archive")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let out = out.canonicalize()?;
    let extracted = out.join("extracted source");
    fs::create_dir(&extracted)?;
    crate::archive::extract(
        &archive,
        &extracted,
        Path::new("inkscape-mcp-source-bootstrap"),
    )?;
    let source = extracted.join("inkscape-mcp-source-bootstrap");
    ensure(
        !source.join(".git").exists() && !source.join(".inkscape-mcp-local").exists(),
        "source archive contains private state",
    )?;
    let old_home = std::env::var("HOME")?;
    let repo = std::env::current_dir()?;
    let home = out.join("home");
    let workspace = out.join("workspace");
    fs::create_dir(&home)?;
    fs::create_dir(&workspace)?;
    let codex = tool("codex")?;
    let vendor = out.join("client-bin");
    fs::create_dir(&vendor)?;
    std::os::unix::fs::symlink(codex, vendor.join("codex"))?;
    let env: Vec<(String, String)> = vec![
        ("HOME".into(), home.display().to_string()),
        (
            "CODEX_HOME".into(),
            home.join(".codex").display().to_string(),
        ),
        ("RUSTUP_HOME".into(), format!("{old_home}/.rustup")),
        ("CARGO_HOME".into(), format!("{old_home}/.cargo")),
        (
            "PATH".into(),
            format!(
                "{}:{old_home}/.cargo/bin:{}",
                vendor.display(),
                std::env::var("PATH")?
            ),
        ),
        (
            "INKSCAPE_MCP_BUILD_TARGET_DIR".into(),
            repo.join("rust/target").display().to_string(),
        ),
        ("SENTRY_DSN".into(), String::new()),
    ];
    let execute = |program: &Path, arguments: &[&str], success: bool| -> Result<(String, String)> {
        let (status, stdout, stderr) = capture_deadline(
            Command::new(program)
                .args(arguments)
                .current_dir(&source)
                .envs(env.clone()),
            600,
        )?;
        use std::io::Write;
        writeln!(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(out.join("commands.log"))?,
            "{program:?} {arguments:?}\n{stdout}{stderr}"
        )?;
        ensure(
            (status == 0) == success,
            format!("source install command status: {stdout}{stderr}"),
        )?;
        Ok((stdout, stderr))
    };
    let setup = source.join("setup.sh");
    let manager = source.join("scripts/mcp-client.sh");
    let engine = if cfg!(target_os = "macos") {
        "/Applications/Inkscape.app"
    } else {
        "/usr/bin/inkscape"
    };
    let options = [
        "--workspace",
        workspace.to_str().unwrap(),
        "--inkscape",
        engine,
        "--live",
        "false",
        "--sentry",
        "false",
        "--install-skill",
        "codex",
        "--connect-client",
        "codex",
    ];
    execute(&setup, &[&["--local-tools"][..], &options].concat(), true)?;
    let config = home.join(".codex/config.toml");
    ensure(
        fs::read_to_string(&config)?.contains(source.join("run-mcp.sh").to_str().unwrap()),
        "wrong registered launcher",
    )?;
    execute(&manager, &["--client", "codex", "connect"], true)?;
    let version = execute(&setup, &["--version"], true)?.0;
    ensure(
        version.contains("build_id") && version.contains("revision"),
        "installed identity missing",
    )?;
    let settings = fs::read_to_string(source.join(".inkscape-mcp-local/setup.conf"))?;
    let installed = Path::new(settings.lines().nth(1).ok_or("installed server missing")?);
    let build: Value = serde_json::from_str(&execute(installed, &["--version"], true)?.0)?;
    let package = installed
        .parent()
        .and_then(Path::parent)
        .ok_or("package path")?;
    ensure(
        build["revision"] == "unknown"
            && crate::common::json(&package.join("libexec/inkscape-mcp/package.json"))?["build_info"]
                == build,
        "unpublished source claims committed identity",
    )?;
    let skill = home.join(".codex/skills/inkscape-mcp");
    let skill_file = skill.join("SKILL.md");
    let upstream = source.join("skills/inkscape-mcp/SKILL.md");
    fs::write(
        &skill_file,
        format!(
            "{}\nUser-specific retained instruction.\n",
            fs::read_to_string(&skill_file)?
        ),
    )?;
    fs::write(
        &upstream,
        fs::read_to_string(&upstream)?.replacen("# Inkscape", "# Updated Inkscape", 1),
    )?;
    let installer = source.join("scripts/install-skill.sh");
    execute(&installer, &["--client", "codex", "--update"], true)?;
    let merged = fs::read_to_string(&skill_file)?;
    ensure(
        merged.contains("User-specific retained") && merged.contains("# Updated Inkscape"),
        "skill merge lost changes",
    )?;
    fs::write(
        &skill_file,
        merged.replacen("# Updated Inkscape", "# User Inkscape", 1),
    )?;
    fs::write(
        &upstream,
        fs::read_to_string(&upstream)?.replacen("# Updated Inkscape", "# Supplier Inkscape", 1),
    )?;
    let before = fs::read(&skill_file)?;
    ensure(
        fs::read_to_string(skill.join(".inkscape-mcp-upstream/SKILL.md"))?
            .contains("# Updated Inkscape"),
        "skill merge baseline missing",
    )?;
    let conflict = execute(&installer, &["--client", "codex", "--update"], false)?.1;
    ensure(
        fs::read(&skill_file)? == before,
        "conflict changed installed skill",
    )?;
    let proposal = Path::new(
        conflict
            .split_once("Review proposed files at ")
            .ok_or("proposal missing")?
            .1
            .trim()
            .trim_end_matches('.'),
    );
    ensure(
        !proposal.starts_with(skill.parent().unwrap())
            && fs::read_to_string(proposal.join("SKILL.md"))?.contains("<<<<<<<")
            && !fs::read_dir(skill.parent().unwrap())?.any(|p| {
                p.is_ok_and(|p| {
                    p.file_name()
                        .to_string_lossy()
                        .starts_with(".inkscape-mcp-update.")
                })
            }),
        "conflict leaked into skill discovery",
    )?;
    fs::remove_dir_all(proposal)?;
    let mut wire = Wire::spawn(
        Command::new(source.join("run-mcp.sh"))
            .current_dir(&source)
            .envs(env.clone()),
        &out.join("wire.log"),
    )?;
    let result = (|| -> Result<()> {
        wire.initialize()?;
        let doc =
            data(wire.call("create_document", json!({"width":64,"height":64}))?)?["doc_id"].clone();
        data(wire.call(
            "create_rect",
            json!({"doc_id":doc,"x":4,"y":4,"width":40,"height":40,"fill":"blue"}),
        )?)?;
        data(wire.call("render_preview", json!({"doc_id":doc}))?)?;
        data(wire.call(
            "save_document_as",
            json!({"doc_id":doc,"dest_path":"first.svg"}),
        )?)?;
        Ok(())
    })();
    wire.save_trace(&out.join("wire.trace.json"))?;
    wire.close();
    result?;
    let drawing = workspace.join("first.svg");
    let saved = fs::read(&drawing)?;
    execute(&source.join("uninstall.sh"), &["--client", "codex"], true)?;
    ensure(
        !source.join(".inkscape-mcp-local").exists()
            && !skill.exists()
            && fs::read(&drawing)? == saved,
        "uninstall left state or changed drawing",
    )?;
    let backups: Vec<_> = fs::read_dir(&source)?
        .filter_map(std::result::Result::ok)
        .filter(|p| {
            p.file_name()
                .to_string_lossy()
                .starts_with(".inkscape-mcp-backup-")
        })
        .map(|p| p.path())
        .collect();
    ensure(
        backups.len() == 1 && fs::read(backups[0].join("removed-skill/SKILL.md"))? == before,
        "customized skill not archived",
    )?;
    let backup_settings = fs::read_to_string(backups[0].join("setup.conf"))?;
    let prior = Path::new(
        backup_settings
            .lines()
            .nth(1)
            .ok_or("backup server missing")?,
    )
    .parent()
    .and_then(Path::parent)
    .ok_or("backup package missing")?;
    let package = backups[0].join(prior.strip_prefix(source.join(".inkscape-mcp-local"))?);
    execute(
        &setup,
        &[&["--package", package.to_str().unwrap()][..], &options].concat(),
        true,
    )?;
    write_json(
        &out.join("acceptance.json"),
        &json!({"passed":true,"checks":8,"source":source,"package":package,"native_GUI":false,"clean_machine":false,"source_state":"unpublished working tree"}),
    )?;
    println!("Source install: 8 checks passed");
    Ok(())
}
