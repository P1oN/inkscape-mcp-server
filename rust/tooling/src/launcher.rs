use crate::{
    acceptance::capture,
    common::*,
    wire::{Wire, data},
};
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};
fn invoke(root: &Path, source: &Path, script: &str, args: &[&str]) -> Result<Value> {
    let (exit, stdout) = capture(
        Command::new("/bin/bash")
            .arg(source.join(script))
            .args(args)
            .current_dir(root)
            .env("PATH", "")
            .env("HOME", root.join("home"))
            .env("CODEX_HOME", root.join("home/.codex"))
            .env("INKSCAPE_MCP_RAW_ACTION_ENABLED", "false")
            .env("INKSCAPE_MCP_TOOL_PROFILE", "full")
            .env("INKSCAPE_MCP_TOOL_DESC", "full")
            .env("SENTRY_DSN", ""),
    )?;
    Ok(json!({"exit":exit,"stdout":stdout}))
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--package", "--output"])?;
    let package = args.required("--package")?.canonicalize()?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let root = tempfile::tempdir()?;
    let root = root.path().canonicalize()?;
    let source = root.join("checkout with spaces");
    fs::create_dir(&source)?;
    for name in ["setup.sh", "run-mcp.sh"] {
        ensure(
            read(&package.join(name), 1024 * 1024)? == read(Path::new(name), 1024 * 1024)?,
            "stale packaged launcher",
        )?;
        copy(&package.join(name), &source.join(name))?;
    }
    let workspace = root.join("SVGs $(touch SHOULD_NOT_EXIST); 'quoted'");
    fs::create_dir(&workspace)?;
    let original =
        "<svg xmlns=\"http://www.w3.org/2000/svg\"><rect id=\"r\" width=\"2\" height=\"3\"/></svg>";
    fs::write(workspace.join("fixture.svg"), original)?;
    let missing = invoke(&root, &source, "run-mcp.sh", &[])?;
    ensure(
        missing["exit"] != 0 && missing["stdout"] == "",
        "missing config must fail silently",
    )?;
    let configured = invoke(
        &root,
        &source,
        "setup.sh",
        &[
            "--package",
            package.to_str().ok_or("package UTF-8 path")?,
            "--workspace",
            workspace.to_str().ok_or("workspace UTF-8 path")?,
        ],
    )?;
    write_json(&out.join("setup.json"), &configured)?;
    ensure(
        configured["exit"] == 0 && configured["stdout"] == "",
        "real package setup failed",
    )?;
    let config = source.join(".inkscape-mcp-local/setup.conf");
    let stored = read(&config, 1024 * 1024)?;
    let text = std::str::from_utf8(&stored)?;
    ensure(
        fs::metadata(&config)?.permissions().mode() & 0o777 == 0o600
            && text.contains(workspace.to_str().unwrap()),
        "config permissions/literal workspace drift",
    )?;
    let mut command = Command::new("/bin/bash");
    command
        .arg(source.join("run-mcp.sh"))
        .env("PATH", "")
        .env("INKSCAPE_MCP_RAW_ACTION_ENABLED", "false")
        .env("INKSCAPE_MCP_TOOL_PROFILE", "full")
        .env("INKSCAPE_MCP_TOOL_DESC", "full")
        .env("SENTRY_DSN", "");
    let mut wire = Wire::spawn(&mut command, &out.join("mcp.stderr.log"))?;
    let result = (|| -> Result<()> {
        wire.initialize()?;
        let expected = crate::common::json(Path::new(
            "migration/contracts/live-true_raw-false_full_full.json",
        ))?;
        ensure(
            wire.request("tools/list", None)?["result"]["tools"] == expected["tools/list"]["tools"],
            "default tools differ from frozen contract",
        )?;
        let document =
            data(wire.call("open_document", json!({"path":"fixture.svg"}))?)?["doc_id"].clone();
        data(wire.call("inspect_document", json!({"doc_id":document}))?)?;
        Ok(())
    })();
    wire.save_trace(&out.join("mcp.trace.json"))?;
    result?;
    wire.close();
    ensure(
        fs::read_to_string(workspace.join("fixture.svg"))? == original
            && !root.join("SHOULD_NOT_EXIST").exists(),
        "original changed/config interpreted as shell",
    )?;
    let lines: Vec<_> = text.lines().collect();
    ensure(lines.len() >= 6, "configuration line count")?;
    for (name, content) in [
        ("truncated", format!("{}\n", lines[..3].join("\n"))),
        ("trailing", format!("{text}$(touch SHOULD_NOT_EXIST)\n")),
        (
            "engine",
            format!("{}\n$(touch SHOULD_NOT_EXIST)\n", lines[..5].join("\n")),
        ),
        (
            "workspace",
            format!(
                "{}\n{}:/\n{}\n",
                lines[..3].join("\n"),
                root.display(),
                lines[4..].join("\n")
            ),
        ),
    ] {
        fs::write(&config, content)?;
        let reply = invoke(&root, &source, "run-mcp.sh", &[])?;
        write_json(&out.join(format!("refused-{name}.json")), &reply)?;
        ensure(
            reply["exit"] != 0 && reply["stdout"] == "" && !root.join("SHOULD_NOT_EXIST").exists(),
            format!("accepted/executed {name} config"),
        )?;
    }
    fs::write(&config, &stored)?;
    fs::write(&config, format!("{}\n", lines[..4].join("\n")))?;
    let upgraded = invoke(
        &root,
        &source,
        "setup.sh",
        &["--package", package.to_str().unwrap()],
    )?;
    ensure(
        upgraded["exit"] == 0 && fs::read(&config)? == stored,
        "setup failed to supplement missing live/engine defaults",
    )?;
    let colon = root.join("bad:workspace");
    fs::create_dir(&colon)?;
    let newline = format!("{}\n/", root.display());
    for (label, arguments) in [
        ("colon", vec!["--workspace", colon.to_str().unwrap()]),
        ("newline", vec!["--workspace", &newline]),
        (
            "invalid-mode",
            vec![
                "--workspace",
                workspace.to_str().unwrap(),
                "--engine",
                "invalid",
            ],
        ),
    ] {
        let mut options = vec!["--package", package.to_str().unwrap()];
        options.extend(arguments);
        let reply = invoke(&root, &source, "setup.sh", &options)?;
        ensure(
            reply["exit"] != 0 && fs::read(&config)? == stored,
            format!("setup damaged {label}"),
        )?;
    }
    fs::remove_file(&config)?;
    let sentinel = root.join("sentinel");
    fs::write(&sentinel, &stored)?;
    std::os::unix::fs::symlink(&sentinel, &config)?;
    let reply = invoke(&root, &source, "run-mcp.sh", &[])?;
    ensure(
        reply["exit"] != 0 && reply["stdout"] == "" && fs::read(sentinel)? == stored,
        "linked config accepted/modified",
    )?;
    write_json(
        &out.join("comparison.json"),
        &json!({"passed":true,"checks":11,"binary_sha256":hash(&package.join("bin/inkscape-mcp"))?,"scope":"actual package/CLI/STDIO; no native GUI/source build acceptance"}),
    )?;
    println!("Launcher: 11 checks passed");
    Ok(())
}
