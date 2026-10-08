use inkscape_mcp_rust::update::{instructions::*, manifests::*};
use serde_json::json;
use std::{collections::BTreeMap, fs};

fn files() -> BTreeMap<String, Vec<u8>> {
    default_files()
}
#[test]
fn embedded_text_and_content_identity_are_deterministic() {
    let files = files();
    let m = manifest("v1", &files);
    assert_eq!(m, manifest("v1", &files));
    let bundle = Instructions::from_files(m, files).unwrap();
    let init = bundle.expand(&bundle.initialization);
    let compose = bundle.expand(
        bundle.prompts["compose_artwork"]["messages"][0]["content"]["text"]
            .as_str()
            .unwrap(),
    );
    assert_eq!(init.matches(bundle.guidance.trim_end()).count(), 1);
    assert_eq!(compose.matches(bundle.guidance.trim_end()).count(), 1);
    assert!(!init.contains(GUIDANCE_SLOT));
}
#[test]
fn text_changes_need_no_interface_change() {
    let mut files = files();
    files
        .get_mut("initialize.txt")
        .unwrap()
        .extend_from_slice(b"\nUpdated orientation.");
    let before = manifest("v1", &default_files());
    let after = manifest("v2", &files);
    assert_ne!(before.content_id, after.content_id);
    assert!(
        Instructions::from_files(after, files)
            .unwrap()
            .initialization
            .contains("Updated orientation.")
    );
}
#[test]
fn refuses_corrupt_unknown_and_oversized_bundles() {
    let original = files();
    let m = manifest("v1", &original);
    let mut corrupt = original.clone();
    corrupt.get_mut("initialize.txt").unwrap().push(b'x');
    assert!(Instructions::from_files(m.clone(), corrupt).is_err());
    for key in [
        "../escape",
        "/absolute",
        "schemas.json",
        "skills/inkscape-mcp/run.sh",
    ] {
        let mut changed = original.clone();
        changed.insert(key.into(), b"extra".to_vec());
        assert!(Instructions::from_files(manifest("v1", &changed), changed).is_err());
    }
    let mut changed = original.clone();
    changed.insert("initialize.txt".into(), vec![b'x'; 256 * 1024 + 1]);
    assert!(Instructions::from_files(manifest("v1", &changed), changed).is_err());
    let mut future = m.clone();
    future.format = 2;
    assert!(future.validate().is_err());
    let mut future = m.clone();
    future.launcher_minimum = LAUNCHER_VERSION + 1;
    assert!(future.validate().is_err());
    let mut future = m;
    future.text_interface = 2;
    assert!(future.validate().is_err());
}
#[test]
fn prompt_arguments_roles_types_names_and_placeholders_stay_compiled() {
    for mutation in 0..7 {
        let mut files = files();
        let mut prompts: serde_json::Value =
            serde_json::from_slice(&files["prompts.json"]).unwrap();
        match mutation {
            0 => prompts["unknown"] = prompts["compose_artwork"].clone(),
            1 => prompts["compose_artwork"]["messages"][0]["role"] = json!("assistant"),
            2 => prompts["compose_artwork"]["messages"][0]["content"]["type"] = json!("image"),
            3 => {
                prompts["compose_artwork"]["messages"][0]["content"]["text"] =
                    json!("missing slots")
            }
            4 => {
                prompts["prepare_web_export"]["messages"][0]["content"]["text"] =
                    json!("__UNKNOWN_SLOT__")
            }
            5 => prompts["prepare_web_export"]["messages"][0]["content"]["text"] = json!(GOAL_SLOT),
            6 => {
                files.insert("prompt-interfaces.json".into(), b"[]".to_vec());
            }
            _ => unreachable!(),
        }
        files.insert("prompts.json".into(), serde_json::to_vec(&prompts).unwrap());
        assert!(
            Instructions::from_files(manifest("v1", &files), files).is_err(),
            "mutation {mutation}"
        );
    }
}
#[test]
fn loader_checks_symlinks_permissions_and_exact_hashes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap().join("bundle");
    fs::create_dir(&root).unwrap();
    let files = files();
    let m = manifest("v1", &files);
    fs::write(root.join("manifest.json"), serde_json::to_vec(&m).unwrap()).unwrap();
    for (name, bytes) in &files {
        let path = root.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    assert!(Instructions::load(&root).is_ok());
    fs::write(root.join("initialize.txt"), "corrupt").unwrap();
    assert!(Instructions::load(&root).is_err());
    #[cfg(unix)]
    {
        use std::os::unix::fs::{PermissionsExt, symlink};
        fs::remove_file(root.join("initialize.txt")).unwrap();
        fs::write(temp.path().join("outside"), &files["initialize.txt"]).unwrap();
        symlink(temp.path().join("outside"), root.join("initialize.txt")).unwrap();
        assert!(Instructions::load(&root).is_err());
        fs::remove_file(root.join("initialize.txt")).unwrap();
        fs::write(root.join("initialize.txt"), &files["initialize.txt"]).unwrap();
        fs::set_permissions(
            root.join("initialize.txt"),
            fs::Permissions::from_mode(0o666),
        )
        .unwrap();
        assert!(Instructions::load(&root).is_err());
    }
}
#[test]
fn compatibility_rejects_unknown_formats_targets_and_protocols() {
    let asset = Asset {
        name: "runtime.tar.gz".into(),
        identity: FileIdentity::of(b"archive"),
    };
    let release = ReleaseManifest {
        format: 1,
        tag: "v1".into(),
        prerelease: true,
        runtime: RuntimeManifest {
            format: 1,
            distribution_tag: "v1".into(),
            build_id: "build1".into(),
            source_revision: "revision1".into(),
            os: "macos".into(),
            architecture: "aarch64".into(),
            minimum_os_major: 15,
            text_interface: 1,
            helper_protocol: 5,
            launcher_minimum: 1,
            asset: asset.clone(),
        },
        instructions: manifest("v1", &files()),
        instruction_asset: asset.clone(),
        launcher_asset: asset,
    };
    assert!(release.compatible("macos", "aarch64", 15).is_ok());
    assert!(release.compatible("macos", "aarch64", 14).is_err());
    assert!(release.compatible("linux", "aarch64", 15).is_err());
    assert!(release.compatible("macos", "x86_64", 15).is_err());
    let mut incompatible = release.clone();
    incompatible.runtime.helper_protocol = 2;
    assert!(incompatible.compatible("macos", "aarch64", 15).is_err());
    let mut incompatible = release.clone();
    incompatible.format = 2;
    assert!(incompatible.compatible("macos", "aarch64", 15).is_err());
    let mut incompatible = release;
    incompatible.runtime.text_interface = 2;
    assert!(incompatible.compatible("macos", "aarch64", 15).is_err());
}
