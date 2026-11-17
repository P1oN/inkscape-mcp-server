use crate::{archive, common::*};
use inkscape_mcp_rust::update::instructions;
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--output", "--version", "--archive"])?;
    let root = args.required("--output")?;
    let version = args
        .values
        .get("--version")
        .ok_or("--version is required")?;
    let files = instructions::default_files();
    let manifest = instructions::manifest(version, &files);
    instructions::Instructions::from_files(manifest.clone(), files.clone())?;
    std::fs::create_dir(&root)?;
    for (name, bytes) in files {
        let target = root.join(name);
        std::fs::create_dir_all(target.parent().unwrap())?;
        std::fs::write(target, bytes)?;
    }
    write_json(
        &root.join("manifest.json"),
        &serde_json::to_value(&manifest)?,
    )?;
    if args.flag("--archive") {
        archive::pack(&root, &args.required("--archive")?)?;
    }
    println!("{}", serde_json::to_string(&manifest)?);
    Ok(())
}

pub fn distribution(args: &Args) -> Result<()> {
    use inkscape_mcp_rust::update::manifests::{
        Asset, FORMAT, FileIdentity, HELPER_PROTOCOL, ReleaseManifest, RuntimeManifest,
        TEXT_INTERFACE,
    };
    args.check(&[
        "--output",
        "--runtime",
        "--instructions",
        "--launcher",
        "--tag",
    ])?;
    let runtime = args.required("--runtime")?;
    let instructions = args.required("--instructions")?;
    let launcher = args.required("--launcher")?;
    let tag = args.values.get("--tag").ok_or("--tag required")?;
    let stage = tempfile::tempdir()?;
    let stage_root = stage.path().canonicalize()?;
    archive::extract(
        &runtime,
        &stage_root,
        std::path::Path::new("inkscape-mcp-macos-arm64"),
    )?;
    let metadata = json(
        &inkscape_mcp_rust::runtime_layout::library(&stage_root.join("inkscape-mcp-macos-arm64"))
            .join("package.json"),
    )?;
    archive::extract(
        &instructions,
        &stage_root,
        std::path::Path::new("inkscape-mcp-instructions"),
    )?;
    let instruction_bundle = inkscape_mcp_rust::update::instructions::Instructions::load(
        &stage_root.join("inkscape-mcp-instructions"),
    )?;
    let asset = |path: &std::path::Path| -> Result<Asset> {
        Ok(Asset {
            name: path
                .file_name()
                .unwrap()
                .to_str()
                .ok_or("invalid asset name")?
                .into(),
            identity: FileIdentity {
                bytes: std::fs::metadata(path)?.len(),
                sha256: hash(path)?,
            },
        })
    };
    let manifest = ReleaseManifest {
        format: FORMAT,
        tag: tag.clone(),
        prerelease: true,
        runtime: RuntimeManifest {
            format: FORMAT,
            distribution_tag: tag.clone(),
            build_id: metadata["build_info"]["build_id"]
                .as_str()
                .ok_or("missing build")?
                .into(),
            source_revision: metadata["build_info"]["revision"]
                .as_str()
                .ok_or("missing revision")?
                .into(),
            os: "macos".into(),
            architecture: "aarch64".into(),
            minimum_os_major: 15,
            text_interface: TEXT_INTERFACE,
            helper_protocol: HELPER_PROTOCOL,
            launcher_minimum: metadata["update_contract"]["launcher_minimum"]
                .as_u64()
                .ok_or("missing launcher compatibility")?
                .try_into()?,
            asset: asset(&runtime)?,
        },
        instructions: instruction_bundle.manifest,
        instruction_asset: asset(&instructions)?,
        launcher_asset: asset(&launcher)?,
    };
    manifest.compatible("macos", "aarch64", 15)?;
    inkscape_mcp_rust::update::install::verify_runtime(
        &stage_root.join("inkscape-mcp-macos-arm64"),
        &manifest.runtime,
    )?;
    write_json(
        &args.required("--output")?,
        &serde_json::to_value(manifest)?,
    )?;
    Ok(())
}

/// A separate versioned asset keeps deny_unknown_fields clients compatible.
pub fn manager(args: &Args) -> Result<()> {
    use inkscape_mcp_rust::update::{
        manager_release as r,
        manifests::{Asset, FileIdentity},
        storage,
    };
    args.check(&["--output", "--launcher", "--tag", "--channel"])?;
    let archive = args.required("--launcher")?;
    let stage = tempfile::tempdir()?;
    let root = stage.path().canonicalize()?;
    archive::extract(
        &archive,
        &root,
        std::path::Path::new("inkscape-mcp-launcher"),
    )?;
    let app = root.join("inkscape-mcp-launcher").join(r::APP);
    let info = r::info(&app)?;
    // No code from an archive may run until its fixed publisher is verified.
    for path in [app.clone(), app.join(r::HELPER), app.join(r::EXECUTABLE)] {
        ensure(
            inkscape_mcp_rust::client_management::signature_team(&path)
                .map_err(|e| e.to_string())?
                .as_deref()
                == Some(r::PUBLISHER),
            "Manager manifest requires trusted Developer ID signing",
        )?;
    }
    output(
        std::process::Command::new("/usr/bin/codesign")
            .args(["--verify", "--deep", "--strict"])
            .arg(&app),
    )?;
    let compiled = r::bootstrap_info(&app.join(r::HELPER))?;
    let channel = args
        .values
        .get("--channel")
        .map(String::as_str)
        .unwrap_or("prerelease");
    ensure(
        matches!(channel, "stable" | "prerelease"),
        "invalid Manager channel",
    )?;
    let field = |name: &str| -> Result<String> {
        Ok(info.get(name).ok_or("Manager identity missing")?.clone())
    };
    let manifest = r::Manifest {
        format: 1,
        tag: args.values.get("--tag").ok_or("tag required")?.clone(),
        prerelease: channel == "prerelease",
        os: "macos".into(),
        architecture: "aarch64".into(),
        minimum_os_major: 15,
        bundle_identifier: r::BUNDLE.into(),
        publisher: r::PUBLISHER.into(),
        manager_build: field("InkscapeMCPBuildID")?,
        manager_revision: field("InkscapeMCPRevision")?,
        bootstrap: serde_json::from_value(compiled["bootstrap"].clone())?,
        launcher_version: compiled["launcher_version"]
            .as_u64()
            .ok_or("launcher version missing")?
            .try_into()?,
        component_format: compiled["component_format"]
            .as_u64()
            .ok_or("component format missing")?
            .try_into()?,
        text_interface: compiled["text_interface"]
            .as_u64()
            .ok_or("text interface missing")?
            .try_into()?,
        helper_protocol: compiled["helper_protocol"]
            .as_u64()
            .ok_or("helper protocol missing")?
            .try_into()?,
        asset: Asset {
            name: archive
                .file_name()
                .ok_or("asset name missing")?
                .to_str()
                .ok_or("invalid asset")?
                .into(),
            identity: FileIdentity {
                bytes: std::fs::metadata(&archive)?.len(),
                sha256: hash(&archive)?,
            },
        },
        files: storage::inventory(&app, 512 * 1024 * 1024)?,
    };
    r::verify(&app, &manifest)?;
    ensure(
        FileIdentity::of(&inkscape_mcp_rust::update::instructions::read(
            &root.join("inkscape-mcp-launcher/bin/inkscape-mcp-launcher"),
            128 * 1024 * 1024,
        )?) == manifest.bootstrap.executable,
        "standalone bootstrap differs",
    )?;
    storage::write_json(&args.required("--output")?, &manifest)?;
    Ok(())
}

pub fn verify_manager(args: &Args) -> Result<()> {
    use inkscape_mcp_rust::update::{download, manager_release as r, manifests::FileIdentity};
    args.check(&["--directory", "--output"])?;
    let root = args.required("--directory")?.canonicalize()?;
    let manifest_bytes =
        inkscape_mcp_rust::update::instructions::read(&root.join(r::ASSET), 2 * 1024 * 1024)?;
    let manifest: r::Manifest = serde_json::from_slice(&manifest_bytes)?;
    manifest.validate()?;
    let archive = root.join(&manifest.asset.name);
    ensure(
        FileIdentity::of(&inkscape_mcp_rust::update::instructions::read(
            &archive,
            512 * 1024 * 1024,
        )?) == manifest.asset.identity,
        "final Manager archive differs",
    )?;
    // Also validate fixed-repository URL construction; no network is used here.
    download::asset_url(&manifest.tag, &manifest.asset.name)?;
    let stage = tempfile::tempdir()?;
    let stage = stage.path().canonicalize()?;
    crate::archive::extract(
        &archive,
        &stage,
        std::path::Path::new("inkscape-mcp-launcher"),
    )?;
    let payload = stage.join("inkscape-mcp-launcher");
    r::verify(&payload.join(r::APP), &manifest)?;
    ensure(
        FileIdentity::of(&inkscape_mcp_rust::update::instructions::read(
            &payload.join("bin/inkscape-mcp-launcher"),
            128 * 1024 * 1024,
        )?) == manifest.bootstrap.executable,
        "final standalone bootstrap differs",
    )?;
    let output = args.required("--output")?;
    std::fs::create_dir_all(&output)?;
    write_json(
        &output.join("report.json"),
        &serde_json::json!({"passed":true,"manifest_sha256":FileIdentity::of(&manifest_bytes).sha256,"archive":manifest.asset,"manager_build":manifest.manager_build,"bootstrap":manifest.bootstrap}),
    )?;
    Ok(())
}
