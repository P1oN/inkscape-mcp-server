use crate::{
    acceptance::{capture_full, copy_tree},
    common::*,
};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
pub fn run(args: &Args) -> Result<()> {
    args.check(&[
        "--package",
        "--output",
        "--profile",
        "--previous-package",
        "--runtime-archive",
        "--runtime-package",
    ])?;
    let package = args.required("--package")?.canonicalize()?;
    let runtime_package = if args.flag("--runtime-package") {
        args.required("--runtime-package")?.canonicalize()?
    } else {
        package.clone()
    };
    let management_launcher = if args.flag("--runtime-package") {
        inkscape_mcp_rust::runtime_layout::binary(
            &inkscape_mcp_rust::runtime_layout::library(&runtime_package),
            "inkscape-mcp-launcher",
        )
    } else {
        package.join("bin/inkscape-mcp-launcher")
    };
    let previous_package = if args.flag("--previous-package") {
        args.required("--previous-package")?.canonicalize()?
    } else {
        package.clone()
    };
    let out = args.required("--output")?;
    fs::create_dir_all(&out)?;
    let temp = tempfile::tempdir()?;
    let base = if args.flag("--profile") {
        let path = args.required("--profile")?;
        fs::create_dir_all(&path)?;
        path.canonicalize()?
    } else {
        temp.path().canonicalize()?
    };
    let repo = base.join("old ready package");
    copy_tree(&package, &repo)?;
    let home = base.join("profile");
    fs::create_dir_all(home.join(".codex"))?;
    let workspace = base.join("drawings");
    fs::create_dir(&workspace)?;
    fs::write(
        workspace.join("original.svg"),
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    )?;
    let unrelated = base.join("unrelated");
    fs::create_dir(&unrelated)?;
    let codex_home = home.join(".codex");
    let environment = [
        ("HOME", home.to_str().unwrap()),
        ("CODEX_HOME", codex_home.to_str().unwrap()),
        ("PATH", "/opt/homebrew/bin:/usr/bin:/bin"),
    ];
    let execute = |program: &Path, arguments: &[&str], success: bool| -> Result<String> {
        let (status, stdout, stderr) = capture_full(
            Command::new(program)
                .args(arguments)
                .env_clear()
                .envs(environment)
                .current_dir(&unrelated),
        )?;
        ensure(
            (status == 0) == success,
            format!("unexpected launcher result: {stdout}\n{stderr}"),
        )?;
        Ok(stdout)
    };
    let inkscape = if Path::new("/Applications/Inkscape.app/Contents/MacOS/inkscape").exists() {
        "/Applications/Inkscape.app/Contents/MacOS/inkscape"
    } else {
        "/usr/bin/inkscape"
    };
    execute(
        &repo.join("setup.sh"),
        &[
            "--package",
            previous_package.to_str().unwrap(),
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
    execute(
        &repo.join("scripts/install-skill.sh"),
        &["--client", "codex"],
        true,
    )?;
    let skill = home.join(".codex/skills/inkscape-mcp/SKILL.md");
    let original = fs::read(&skill)?;
    let mut custom = b"Custom user orientation.\n".to_vec();
    custom.extend_from_slice(&original);
    fs::write(&skill, &custom)?;
    let config = home.join(".codex/config.toml");
    fs::write(
        &config,
        format!(
            "model = \"preserved\"\n[mcp_servers.inkscape]\ncommand = {}\nargs = []\n[mcp_servers.unrelated]\ncommand = \"keep\"\n",
            toml::Value::String(repo.join("run-mcp.sh").display().to_string())
        ),
    )?;
    fs::write(
        home.join(".claude.json"),
        serde_json::to_vec(
            &json!({"unrelated":{"keep":true},"mcpServers":{"inkscape":{"command":repo.join("run-mcp.sh"),"args":[]},"other":{"command":"keep"}}}),
        )?,
    )?;
    fs::write(
        repo.join(".inkscape-mcp-local/clients.json"),
        "[\"codex\",\"claude\"]",
    )?;
    let installation = base.join("permanent installation");
    let migration: Value = serde_json::from_str(&execute(
        &management_launcher,
        &[
            "migrate",
            "--source",
            repo.to_str().unwrap(),
            "--install-dir",
            installation.to_str().unwrap(),
            "--json",
        ],
        true,
    )?)?;
    let launcher = installation.join("bin/inkscape-mcp-launcher");
    let cli = installation.join("bin/inkscape-mcp");
    let selected: Value = serde_json::from_str(&execute(&cli, &["--version", "--json"], true)?)?;
    ensure(
        selected["runtime_build"] == migration["runtime_build"],
        "permanent CLI selected a different runtime",
    )?;
    let codex: toml::Value = toml::from_str(&fs::read_to_string(&config)?)?;
    ensure(
        codex["model"].as_str() == Some("preserved")
            && codex["mcp_servers"]["unrelated"]["command"].as_str() == Some("keep")
            && codex["mcp_servers"]["inkscape"]["command"].as_str() == launcher.to_str(),
        "Codex migration lost preferences/binding",
    )?;
    let claude = json(&home.join(".claude.json"))?;
    ensure(
        claude["unrelated"]["keep"] == true
            && claude["mcpServers"]["other"]["command"] == "keep"
            && claude["mcpServers"]["inkscape"]["command"] == launcher.to_str().unwrap(),
        "synthetic Claude migration lost settings",
    )?;
    ensure(
        fs::read(&skill)? == custom,
        "migration overwrote skill customization",
    )?;
    fs::rename(&repo, base.join("source moved away"))?;
    let settings: inkscape_mcp_rust::update::install::Settings =
        inkscape_mcp_rust::update::storage::json(&installation.join("settings.json"))?;
    let selector = inkscape_mcp_rust::update::install::selector(&installation)?;
    inkscape_mcp_rust::update::install::probe(&installation, &selector.current, &settings)?;
    let wire = inkscape_mcp_rust::client_management::probe(
        Command::new(&launcher).env_clear().envs(environment),
        std::time::Duration::from_secs(30),
    )
    .map_err(|e| e.to_string())?;
    ensure(
        execute(&cli, &["--help"], true)?.contains("update --instructions"),
        "CLI help missing component commands",
    )?;
    execute(&cli, &["update", "--runtime", "--instructions"], false)?;
    let mut runtime_exercised = false;
    if args.flag("--runtime-archive") {
        use inkscape_mcp_rust::update::{
            download::{self, Transport},
            instructions,
            manager::{self, Component},
            manifests::*,
            storage, transaction,
        };
        struct FixtureTransport(std::collections::BTreeMap<String, Vec<u8>>);
        impl Transport for FixtureTransport {
            fn get(&self, url: &str, limit: u64) -> std::result::Result<Vec<u8>, String> {
                let bytes = self.0.get(url).ok_or("isolated network failure")?.clone();
                if bytes.len() as u64 > limit {
                    return Err("fixture download cap".into());
                }
                Ok(bytes)
            }
        }
        let mut files = instructions::default_files();
        files
            .get_mut("initialize.txt")
            .unwrap()
            .extend_from_slice(b"\nIndependent downloaded update acceptance.");
        files
            .get_mut("skills/inkscape-mcp/SKILL.md")
            .unwrap()
            .extend_from_slice(b"\nUpstream download acceptance.\n");
        let instruction_manifest = instructions::manifest("v9.0.0", &files);
        let bundle = base.join("inkscape-mcp-instructions");
        storage::mkdir(&bundle)?;
        for (name, bytes) in files {
            let path = bundle.join(name);
            storage::mkdir(path.parent().unwrap())?;
            storage::write(&path, &bytes)?;
        }
        storage::write_json(&bundle.join("manifest.json"), &instruction_manifest)?;
        let instruction_archive = base.join("instructions.tar.gz");
        crate::archive::pack(&bundle, &instruction_archive)?;
        let instruction_bytes = fs::read(instruction_archive)?;
        let runtime_bytes = fs::read(args.required("--runtime-archive")?)?;
        let metadata = crate::common::json(
            &inkscape_mcp_rust::runtime_layout::library(&runtime_package).join("package.json"),
        )?;
        let release = ReleaseManifest {
            format: 1,
            tag: "v9.0.0".into(),
            prerelease: true,
            runtime: RuntimeManifest {
                format: 1,
                distribution_tag: "v9.0.0".into(),
                build_id: metadata["build_info"]["build_id"].as_str().unwrap().into(),
                source_revision: metadata["build_info"]["revision"].as_str().unwrap().into(),
                os: "macos".into(),
                architecture: "aarch64".into(),
                minimum_os_major: 15,
                text_interface: TEXT_INTERFACE,
                helper_protocol: HELPER_PROTOCOL,
                launcher_minimum: metadata["update_contract"]["launcher_minimum"]
                    .as_u64()
                    .ok_or("missing launcher compatibility")?
                    .try_into()?,
                asset: Asset {
                    name: "inkscape-mcp-macos-arm64.tar.gz".into(),
                    identity: FileIdentity::of(&runtime_bytes),
                },
            },
            instructions: instruction_manifest,
            instruction_asset: Asset {
                name: "instructions.tar.gz".into(),
                identity: FileIdentity::of(&instruction_bytes),
            },
            launcher_asset: Asset {
                name: "launcher.tar.gz".into(),
                identity: FileIdentity::of(b"unused launcher"),
            },
        };
        let transport=FixtureTransport(std::collections::BTreeMap::from([
            ("https://api.github.com/repos/P1oN/inkscape-mcp-server/releases?per_page=100&page=1".into(),serde_json::to_vec(&json!([{"tag_name":"v9.0.0","draft":false,"prerelease":true,"assets":[{"name":download::MANIFEST_ASSET}]}]))?),
            (download::asset_url("v9.0.0",download::MANIFEST_ASSET)?,serde_json::to_vec(&release)?),
            (download::asset_url("v9.0.0","instructions.tar.gz")?,instruction_bytes),
            (download::asset_url("v9.0.0","inkscape-mcp-macos-arm64.tar.gz")?,runtime_bytes),
        ]));
        let runtime_differs = release.runtime.build_id != selector.current.runtime.build_id;
        ensure(
            runtime_differs || !args.flag("--previous-package"),
            "explicit previous package must have a different runtime build ID",
        )?;
        let binding = fs::read(&config)?;
        for (label, component) in [
            ("instructions", Component::Instructions),
            ("runtime", Component::Runtime),
            ("combined", Component::Both),
        ] {
            let checked = manager::update(
                &installation,
                &transport,
                component,
                true,
                Some("prerelease"),
                None,
                15,
            )?;
            ensure(
                inkscape_mcp_rust::update::install::selector(&installation)?.current
                    == selector.current,
                "read-only check changed installation",
            )?;
            let updated = manager::update(
                &installation,
                &transport,
                component,
                false,
                Some("prerelease"),
                None,
                15,
            )?;
            let runtime_expected = runtime_differs && !matches!(component, Component::Instructions);
            let instructions_expected = !matches!(component, Component::Runtime);
            ensure(
                updated["changed"] == (runtime_expected || instructions_expected),
                format!("{label} update did not change the expected components"),
            )?;
            let active = inkscape_mcp_rust::update::install::selector(&installation)?;
            ensure(
                active.current.runtime.build_id.as_str()
                    == if runtime_expected {
                        release.runtime.build_id.as_str()
                    } else {
                        selector.current.runtime.build_id.as_str()
                    }
                    && active.current.instructions.content_id.as_str()
                        == if instructions_expected {
                            release.instructions.content_id.as_str()
                        } else {
                            selector.current.instructions.content_id.as_str()
                        },
                format!("{label} selected unexpected runtime/instructions"),
            )?;
            inkscape_mcp_rust::client_management::probe(
                Command::new(&launcher).env_clear().envs(environment),
                std::time::Duration::from_secs(30),
            )
            .map_err(|e| e.to_string())?;
            runtime_exercised |= runtime_expected;
            if updated["changed"] == true {
                transaction::rollback(&installation, &settings)?;
            }
            ensure(
                inkscape_mcp_rust::update::install::selector(&installation)?.current
                    == selector.current,
                "rollback changed original pair",
            )?;
            inkscape_mcp_rust::client_management::probe(
                Command::new(&launcher).env_clear().envs(environment),
                std::time::Duration::from_secs(30),
            )
            .map_err(|e| e.to_string())?;
            ensure(
                fs::read(&skill)? == custom && fs::read(&config)? == binding,
                "downloaded update/rollback changed customization or registration",
            )?;
            write_json(
                &out.join(format!("{label}.json")),
                &json!({"checked":checked,"updated":updated,"restored":true,"runtime_exercised":runtime_expected}),
            )?;
        }
    }
    ensure(
        fs::read_to_string(workspace.join("original.svg"))?.contains("xmlns"),
        "drawing changed",
    )?;
    write_json(
        &out.join("report.json"),
        &json!({"passed":true,"runtime_exercised":runtime_exercised,"migration":migration,"wire":wire,"checks":["relocated native package","permanent CLI from unrelated directory","source removal and offline STDIO","private settings preservation","Codex TOML and synthetic Claude JSON bindings","custom skill preservation","help and component-option refusal"],"scope":"Isolated profiles and actual native package/STDIO. Not real Claude, clean-machine or native GUI acceptance."}),
    )?;
    Ok(())
}
