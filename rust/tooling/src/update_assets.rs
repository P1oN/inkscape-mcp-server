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
