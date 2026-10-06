use crate::{acceptance::capture_full, common::*};
use serde_json::json;
use std::{
    fs,
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::PermissionsExt,
    },
    path::Path,
    process::Command,
    time::{Duration, Instant},
};
const DSN: &str = "https://0123456789abcdef0123456789abcdef@sentry.invalid/123";
pub fn fixture() {
    fs::write(std::env::var_os("IMCP_CONFIG_MARKER").unwrap(), "started").unwrap();
    println!(
        "{}",
        json!({"doctor":std::env::args().any(|a|a=="--doctor"),"dsn":std::env::var("SENTRY_DSN").ok(),"environment":std::env::var("SENTRY_ENVIRONMENT").ok()})
    );
}
fn script(path: &Path, text: &str) -> Result<()> {
    fs::write(path, text)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}
fn check(checks: &mut Vec<String>, ok: bool, label: &str) -> Result<()> {
    ensure(ok, label)?;
    checks.push(label.into());
    Ok(())
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--output"])?;
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let temporary = tempfile::tempdir()?;
    let root = temporary.path().canonicalize()?;
    let checkout = root.join("checkout with spaces");
    fs::create_dir(&checkout)?;
    for name in [
        "setup.sh",
        "run-mcp.sh",
        "uninstall.sh",
        "scripts/mcp-client.sh",
        "scripts/install-skill.sh",
        "skills/inkscape-mcp/SKILL.md",
        "skills/inkscape-mcp/agents/openai.yaml",
    ] {
        copy(Path::new(name), &checkout.join(name))?;
    }
    let package = root.join("package");
    fs::create_dir_all(package.join("libexec/inkscape-mcp"))?;
    fs::write(package.join("libexec/inkscape-mcp/package.json"), "{}")?;
    let binary = package.join("bin/inkscape-mcp");
    copy(&std::env::current_exe()?, &binary)?;
    for name in ["client", "supervisor", "inx", "live"] {
        copy(&binary, &package.join(format!("bin/inkscape-mcp-{name}")))?;
    }
    let vendor = root.join("vendor");
    fs::create_dir(&vendor)?;
    let engine = vendor.join("inkscape");
    script(&engine, "#!/bin/bash\nexit 0\n")?;
    let workspace = root.join("workspace");
    fs::create_dir(&workspace)?;
    let home = root.join("home");
    let codex_home = root.join("custom codex home");
    let marker = root.join("server-started");
    let env = [
        ("SENTRY_DSN", "ambient-value"),
        ("SENTRY_ENVIRONMENT", "ambient"),
        ("HOME", home.to_str().unwrap()),
        ("CODEX_HOME", codex_home.to_str().unwrap()),
        ("INKSCAPE_MCP_TOOLING_FIXTURE", "config"),
        ("IMCP_CONFIG_MARKER", marker.to_str().unwrap()),
    ];
    let base = [
        "--package",
        package.to_str().unwrap(),
        "--workspace",
        workspace.to_str().unwrap(),
        "--inkscape",
        engine.to_str().unwrap(),
    ];
    let invoke = |name: &str, arguments: &[&str]| {
        capture_full(
            Command::new("/bin/bash")
                .arg(checkout.join(name))
                .args(arguments)
                .envs(env),
        )
    };
    let setup = |prefix: &[&str], extra: &[&str]| {
        let options: Vec<_> = prefix.iter().chain(extra).copied().collect();
        invoke("setup.sh", &options)
    };
    let mut checks = vec![];
    check(
        &mut checks,
        setup(&base, &[])?.0 == 0,
        "legacy noninteractive setup",
    )?;
    check(
        &mut checks,
        !marker.exists(),
        "default setup does not execute package code",
    )?;
    let client_script = checkout.join("scripts/mcp-client.sh");
    let saved_client = fs::read(&client_script)?;
    script(
        &client_script,
        "#!/bin/bash\necho 'injected connection failure' >&2\nexit 1\n",
    )?;
    let result = setup(&base, &["--connect-client", "codex"])?;
    let setup_config = checkout.join(".inkscape-mcp-local/setup.conf");
    check(
        &mut checks,
        result.0 != 0
            && setup_config.is_file()
            && result
                .2
                .contains("MCP settings were saved, but client connection failed")
            && result
                .2
                .contains("scripts/mcp-client.sh --client codex upgrade"),
        "failed client connection retains settings and explains standalone retry",
    )?;
    fs::write(client_script, saved_client)?;
    let skill = codex_home.join("skills/inkscape-mcp");
    check(&mut checks, !skill.exists(), "skill installation is opt-in")?;
    check(
        &mut checks,
        setup(&base, &["--install-skill", "codex"])?.0 == 0
            && fs::read(skill.join("SKILL.md"))?
                == fs::read(checkout.join("skills/inkscape-mcp/SKILL.md"))?
            && skill.join("agents/openai.yaml").is_file()
            && !marker.exists(),
        "packaged setup installs skill in custom CODEX_HOME without runtime execution",
    )?;
    check(
        &mut checks,
        setup(&base, &["--install-skill", "codex"])?.0 == 0,
        "identical skill reinstall succeeds",
    )?;
    let saved_skill = b"user-customized skill\n";
    fs::write(skill.join("SKILL.md"), saved_skill)?;
    check(
        &mut checks,
        setup(&base, &["--install-skill", "codex"])?.0 == 0
            && fs::read(skill.join("SKILL.md"))?
                == fs::read(checkout.join("skills/inkscape-mcp/SKILL.md"))?
            && fs::read_dir(codex_home.join("inkscape-mcp-backups"))?
                .filter_map(std::result::Result::ok)
                .any(|p| fs::read(p.path().join("SKILL.md")).is_ok_and(|v| v == saved_skill)),
        "different existing skill replaced and archived outside discovery",
    )?;
    fs::remove_file(skill.join("SKILL.md"))?;
    let sentinel = root.join("skill-sentinel");
    fs::write(&sentinel, saved_skill)?;
    std::os::unix::fs::symlink(&sentinel, skill.join("SKILL.md"))?;
    check(
        &mut checks,
        invoke("scripts/install-skill.sh", &["--client", "codex"])?.0 != 0
            && fs::read(sentinel)? == saved_skill,
        "existing skill file symlink refused without touching target",
    )?;
    fs::remove_file(skill.join("SKILL.md"))?;
    fs::write(skill.join("SKILL.md"), saved_skill)?;
    fs::write(skill.join("obsolete.txt"), "stale extra")?;
    check(
        &mut checks,
        setup(&base, &[])?.0 == 0
            && fs::read(skill.join("SKILL.md"))?
                == fs::read(checkout.join("skills/inkscape-mcp/SKILL.md"))?
            && !skill.join("obsolete.txt").exists(),
        "ordinary setup replaces installed skill and removes stale files",
    )?;
    check(
        &mut checks,
        setup(&base, &["--install-skill", "claude"])?.0 == 0
            && home.join(".claude/skills/inkscape-mcp/SKILL.md").is_file()
            && !marker.exists(),
        "Claude skill installation uses isolated home without runtime execution",
    )?;
    let custom = root.join("project skills");
    check(
        &mut checks,
        invoke(
            "scripts/install-skill.sh",
            &["--destination", custom.to_str().unwrap()],
        )?
        .0 == 0
            && custom.join("inkscape-mcp/SKILL.md").is_file(),
        "standalone installation supports explicit skills directory",
    )?;
    let linked = root.join("linked skills");
    std::os::unix::fs::symlink(&custom, &linked)?;
    check(
        &mut checks,
        invoke(
            "scripts/install-skill.sh",
            &["--destination", linked.to_str().unwrap()],
        )?
        .0 != 0,
        "symlink destination refused",
    )?;
    check(
        &mut checks,
        setup(&base, &["--check"])?.0 == 0 && marker.exists(),
        "explicit setup check executes doctor",
    )?;
    fs::remove_file(&marker)?;
    fs::create_dir(checkout.join("rust"))?;
    fs::write(
        checkout.join("rust/Cargo.toml"),
        "synthetic source checkout",
    )?;
    check(
        &mut checks,
        setup(&base[2..], &[])?.0 != 0 && !marker.exists(),
        "unidentified sources require a builder instead of reusing stale runtime",
    )?;
    let build_marker = root.join("builder-called");
    for (name, mode, expected) in [
        ("bootstrap-local-package.sh", "auto", "false"),
        ("build-local-package.sh", "local", "true"),
    ] {
        script(
            &checkout.join("scripts").join(name),
            &format!(
                "#!/bin/bash\nset -eu\n[ \"$INKSCAPE_MCP_BUILD_LOCAL_TOOLS_ONLY\" = {expected} ]\nprintf '%s' {mode} > {}\nprintf '%s\\n' {}\n",
                shell_quote(&build_marker),
                shell_quote(&package)
            ),
        )?;
    }
    fs::remove_file(&setup_config)?;
    check(
        &mut checks,
        setup(&base[2..], &[])?.0 == 0
            && fs::read_to_string(&build_marker)? == "auto"
            && !marker.exists(),
        "fresh source setup automatically chooses provisioning without starting runtime",
    )?;
    fs::remove_file(&build_marker)?;
    check(
        &mut checks,
        setup(&base[2..], &[])?.0 == 0 && fs::read_to_string(&build_marker)? == "auto",
        "unidentified source rerun rebuilds automatically",
    )?;
    fs::remove_file(&build_marker)?;
    write_json(
        &package.join("libexec/inkscape-mcp/package.json"),
        &json!({"source_head":"1".repeat(40)}),
    )?;
    fs::write(
        checkout.join("SOURCE_REVISION"),
        format!("inkscape-mcp-source-v1\n{}\n", "1".repeat(40)),
    )?;
    check(
        &mut checks,
        setup(&base[2..], &["--local-tools"])?.0 == 0
            && fs::read_to_string(&build_marker)? == "local",
        "local-tools explicitly rebuilds with offline tools-only policy",
    )?;
    fs::remove_file(&build_marker)?;
    check(
        &mut checks,
        setup(&base, &[])?.0 == 0 && !build_marker.exists(),
        "explicit ready package skips source builders",
    )?;
    let saved_setup = fs::read(&setup_config)?;
    check(
        &mut checks,
        setup(&base, &["--local-tools"])?.0 != 0
            && setup(&base[2..], &["--local-tools", "--bootstrap"])?.0 != 0
            && fs::read(&setup_config)? == saved_setup
            && !build_marker.exists(),
        "conflicting build and package options fail before mutation",
    )?;
    for (option, expected) in [
        ("--build", "local"),
        ("--bootstrap", "auto"),
        ("--rebuild", "auto"),
    ] {
        check(
            &mut checks,
            setup(&base[2..], &[option])?.0 == 0 && fs::read_to_string(&build_marker)? == expected,
            &format!("legacy build option remains compatible: {option}"),
        )?;
    }
    fs::remove_file(&build_marker)?;
    check(
        &mut checks,
        invoke("setup.sh", &["--live", "false", "--engine", "shell"])?.0 == 0
            && invoke("setup.sh", &[])?.0 == 0
            && fs::read_to_string(&setup_config)?
                .lines()
                .skip(4)
                .take(2)
                .collect::<Vec<_>>()
                == ["false", "shell"]
            && !build_marker.exists(),
        "plain rerun preserves workspace Inkscape live and engine settings",
    )?;
    check(
        &mut checks,
        invoke("setup.sh", &["--live", "true", "--engine", "per_call"])?.0 == 0
            && fs::read_to_string(&setup_config)?
                .lines()
                .skip(4)
                .take(2)
                .collect::<Vec<_>>()
                == ["true", "per_call"],
        "explicit options override saved settings",
    )?;
    let saved_setup = fs::read(&setup_config)?;
    let invalid = root.join("bad-dsn-early");
    fs::write(&invalid, "not-a-dsn\n")?;
    for options in [
        vec!["--sentry", "invalid"],
        vec!["--sentry-environment", "bad\nlabel"],
        vec!["--sentry-dsn-file", invalid.to_str().unwrap()],
        vec!["--sentry", "false", "--sentry-environment", "wife"],
        vec!["--engine", ""],
    ] {
        let mut arguments = vec!["--bootstrap"];
        arguments.extend(options.clone());
        check(
            &mut checks,
            invoke("setup.sh", &arguments)?.0 != 0
                && !build_marker.exists()
                && fs::read(&setup_config)? == saved_setup,
            &format!("invalid telemetry fails before build: {}", options[0]),
        )?;
    }
    fs::remove_file(&setup_config)?;
    fs::create_dir(&setup_config)?;
    check(
        &mut checks,
        setup(&base, &[])?.0 != 0
            && fs::read_dir(&setup_config)?.count() == 0
            && !build_marker.exists(),
        "directory configuration refuses without false success or build",
    )?;
    fs::remove_dir(&setup_config)?;
    fs::write(&setup_config, &saved_setup)?;
    let manifest = package.join("libexec/inkscape-mcp/package.json");
    write_json(&manifest, &json!({"source_head":"1".repeat(40)}))?;
    let revision = checkout.join("SOURCE_REVISION");
    fs::write(
        &revision,
        format!("inkscape-mcp-source-v1\n{}\n", "1".repeat(40)),
    )?;
    check(
        &mut checks,
        invoke("setup.sh", &[])?.0 == 0 && !build_marker.exists(),
        "same source revision reuses installed runtime",
    )?;
    fs::write(
        &revision,
        format!("inkscape-mcp-source-v1\n{}\n", "2".repeat(40)),
    )?;
    check(
        &mut checks,
        invoke("setup.sh", &[])?.0 == 0 && fs::read_to_string(&build_marker)? == "auto",
        "changed committed source revision triggers automatic rebuild",
    )?;
    fs::remove_file(&build_marker)?;
    check(
        &mut checks,
        invoke("setup.sh", &["--package", package.to_str().unwrap()])?.0 == 0
            && !build_marker.exists(),
        "explicit package wins over source revision mismatch",
    )?;
    script(
        &checkout.join("scripts/bootstrap-local-package.sh"),
        "#!/bin/bash\nexit 22\n",
    )?;
    let saved_setup = fs::read(&setup_config)?;
    check(
        &mut checks,
        setup(&base[2..], &["--bootstrap"])?.0 != 0 && fs::read(&setup_config)? == saved_setup,
        "failed build preserves saved configuration",
    )?;
    check(
        &mut checks,
        invoke("setup.sh", &[])?.0 != 0 && fs::read(&setup_config)? == saved_setup,
        "failed automatic update preserves saved runtime configuration",
    )?;
    fs::remove_file(revision)?;
    fs::write(&manifest, "{}")?;
    for name in [
        "inkscape-mcp",
        "inkscape-mcp-client",
        "inkscape-mcp-supervisor",
        "inkscape-mcp-inx",
        "inkscape-mcp-live",
    ] {
        copy(&binary, &checkout.join("bin").join(name))?;
    }
    fs::create_dir_all(checkout.join("libexec/inkscape-mcp"))?;
    fs::write(checkout.join("libexec/inkscape-mcp/package.json"), "{}")?;
    check(
        &mut checks,
        setup(&base[2..], &[])?.0 == 0
            && fs::read_to_string(&setup_config)?
                .contains(checkout.join("bin/inkscape-mcp").to_str().unwrap())
            && !build_marker.exists()
            && !marker.exists(),
        "unpacked ready package default setup skips all builds and runtime execution",
    )?;
    fs::remove_dir_all(checkout.join("bin"))?;
    fs::remove_dir_all(checkout.join("libexec"))?;
    fs::write(&setup_config, saved_setup)?;
    let emitted = |key: &str| -> Result<serde_json::Value> {
        let (_, stdout, _) = invoke("run-mcp.sh", &[])?;
        let value: serde_json::Value = serde_json::from_str(&stdout)?;
        Ok(value[key].clone())
    };
    check(
        &mut checks,
        emitted("dsn")? == "ambient-value",
        "legacy environment retained without local setting",
    )?;
    let config = checkout.join(".inkscape-mcp-local/sentry.conf");
    fs::write(&config, "inkscape-mcp-sentry-v1\nfalse\n")?;
    check(
        &mut checks,
        setup(&base, &[])?.0 == 0
            && fs::read_to_string(&config)? == "inkscape-mcp-sentry-v1\nfalse\n\nproduction\n",
        "setup supplements missing optional Sentry fields without enabling reporting",
    )?;
    fs::remove_file(&config)?;
    let input = root.join("dsn-input");
    fs::write(&input, format!("{DSN}\n"))?;
    let result = setup(
        &base,
        &[
            "--sentry",
            "true",
            "--sentry-dsn-file",
            input.to_str().unwrap(),
            "--sentry-environment",
            "wife",
        ],
    )?;
    check(
        &mut checks,
        result.0 == 0 && !result.1.contains(DSN) && !result.2.contains(DSN),
        "enable without DSN disclosure",
    )?;
    check(
        &mut checks,
        fs::metadata(&config)?.permissions().mode() & 0o777 == 0o600,
        "DSN file mode600",
    )?;
    output(
        Command::new("git")
            .arg("-c")
            .arg("init.templateDir=")
            .arg("-C")
            .arg(&checkout)
            .args(["init", "--quiet"]),
    )?;
    let (status, _, _) = capture_full(
        Command::new("git")
            .arg("-C")
            .arg(&checkout)
            .args(["check-ignore", ".inkscape-mcp-local/sentry.conf"]),
    )?;
    check(
        &mut checks,
        status == 0,
        "DSN ignored in a checkout without root gitignore",
    )?;
    check(
        &mut checks,
        emitted("environment")? == "wife",
        "wife environment and saved DSN override ambient",
    )?;
    check(
        &mut checks,
        emitted("dsn")? == DSN,
        "saved DSN forwarded as data",
    )?;
    let saved = fs::read(&config)?;
    check(
        &mut checks,
        setup(&base, &[])?.0 == 0 && fs::read(&config)? == saved,
        "unattended rerun preserves telemetry",
    )?;
    fs::write(&input, format!("{DSN}\nsecond line\n"))?;
    let before = fs::read(&setup_config)?;
    check(
        &mut checks,
        setup(&base, &["--sentry-dsn-file", input.to_str().unwrap()])?.0 != 0
            && fs::read(&config)? == saved
            && fs::read(&setup_config)? == before,
        "multiline DSN refuses before settings writes",
    )?;
    fs::write(&input, "$(touch SHOULD_NOT_EXIST)\n")?;
    check(
        &mut checks,
        setup(&base, &["--sentry-dsn-file", input.to_str().unwrap()])?.0 != 0
            && fs::read(&config)? == saved
            && !root.join("SHOULD_NOT_EXIST").exists(),
        "DSN injection refuses without execution or disclosure",
    )?;
    check(
        &mut checks,
        setup(&base, &["--sentry-environment", "wife\nprivate"])?.0 != 0
            && fs::read(&config)? == saved,
        "invalid environment preserves saved DSN",
    )?;
    let mut extra = saved.clone();
    extra.extend_from_slice(b"$(touch SHOULD_NOT_EXIST)\n");
    fs::write(&config, extra)?;
    let result = invoke("run-mcp.sh", &[])?;
    check(
        &mut checks,
        result.0 != 0 && result.1.is_empty(),
        "extra telemetry rows refuse without stdout",
    )?;
    fs::remove_file(&config)?;
    let target = root.join("external");
    fs::write(&target, &saved)?;
    std::os::unix::fs::symlink(&target, &config)?;
    check(
        &mut checks,
        invoke("run-mcp.sh", &[])?.0 != 0
            && setup(&base, &[])?.0 != 0
            && fs::read(target)? == saved,
        "linked telemetry refuses and preserves external target",
    )?;
    fs::remove_file(&config)?;
    fs::write(&config, saved)?;
    check(
        &mut checks,
        setup(&base, &["--sentry", "false"])?.0 == 0 && emitted("dsn")?.is_null(),
        "explicit opt-out overrides ambient DSN",
    )?;
    check(
        &mut checks,
        fs::read_to_string(&config)?.lines().nth(2) == Some(""),
        "opt-out removes stored DSN",
    )?;
    let mut command = Command::new("/bin/bash");
    command.arg(checkout.join("setup.sh")).args(base).envs(env);
    terminal(&mut command)?;
    check(
        &mut checks,
        true,
        "interactive DSN input is hidden and setup succeeds",
    )?;
    check(
        &mut checks,
        emitted("environment")? == "wife",
        "interactive label survives launcher",
    )?;
    write_json(
        &out.join("comparison.json"),
        &json!({"passed":true,"checks":checks,"scope":"Isolated launcher/TTY/privacy/refusal fixtures with inert native executable; no Sentry or GUI"}),
    )?;
    println!("Sentry/setup: {} checks passed", checks.len());
    Ok(())
}
fn terminal(command: &mut Command) -> Result<()> {
    let mut master = -1;
    let mut slave = -1;
    ensure(
        unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        } == 0,
        "PTY unavailable",
    )?;
    let mut master = unsafe { fs::File::from_raw_fd(master) };
    let slave = unsafe { fs::File::from_raw_fd(slave) };
    let flags = unsafe { libc::fcntl(master.as_raw_fd(), libc::F_GETFL) };
    ensure(
        flags >= 0
            && unsafe { libc::fcntl(master.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) }
                == 0,
        "PTY nonblocking setup failed",
    )?;
    let child = command
        .stdin(slave.try_clone()?)
        .stdout(slave.try_clone()?)
        .stderr(slave)
        .spawn()?;
    struct Owned(std::process::Child);
    impl Drop for Owned {
        fn drop(&mut self) {
            if self.0.try_wait().ok().flatten().is_none() {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
    }
    let mut child = Owned(child);
    let mut transcript = vec![];
    let drain = |master: &mut fs::File, transcript: &mut Vec<u8>| -> Result<()> {
        let mut buffer = [0; 65536];
        loop {
            match master.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    ensure(transcript.len() + n <= 1024 * 1024, "PTY transcript cap")?;
                    transcript.extend_from_slice(&buffer[..n]);
                }
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        || error.raw_os_error() == Some(libc::EIO) =>
                {
                    break;
                }
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    };
    for (prompt, answer) in [
        ("Enable Sentry error reporting?", "y"),
        ("Sentry DSN (hidden", DSN),
        ("Sentry environment label", "wife"),
    ] {
        let started = Instant::now();
        while !String::from_utf8_lossy(&transcript).contains(prompt) {
            ensure(
                started.elapsed() < Duration::from_secs(10),
                format!("terminal prompt timeout: {prompt}"),
            )?;
            drain(&mut master, &mut transcript)?;
            std::thread::sleep(Duration::from_millis(5));
        }
        if prompt.starts_with("Sentry DSN") {
            loop {
                let mut term = std::mem::MaybeUninit::<libc::termios>::uninit();
                ensure(
                    unsafe { libc::tcgetattr(master.as_raw_fd(), term.as_mut_ptr()) } == 0,
                    "PTY attributes unavailable",
                )?;
                if unsafe { term.assume_init() }.c_lflag & libc::ECHO == 0 {
                    break;
                }
                ensure(
                    started.elapsed() < Duration::from_secs(10),
                    "hidden input did not disable echo",
                )?;
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        master.write_all(format!("{answer}\n").as_bytes())?;
    }
    let started = Instant::now();
    let status = loop {
        drain(&mut master, &mut transcript)?;
        if let Some(status) = child.0.try_wait()? {
            break status;
        }
        ensure(
            started.elapsed() < Duration::from_secs(10),
            "terminal setup did not exit",
        )?;
        std::thread::sleep(Duration::from_millis(5));
    };
    drain(&mut master, &mut transcript)?;
    ensure(
        status.success() && !String::from_utf8_lossy(&transcript).contains(DSN),
        "interactive DSN echoed or setup failed",
    )
}
