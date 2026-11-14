//! Independent Manager contract. Never extend the strict legacy runtime manifest.
use super::{
    download::{self, Transport},
    manifests::*,
    storage::*,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub const ASSET: &str = "inkscape-mcp-manager-update-v1.json";
pub const BUNDLE: &str = "org.inkscape-mcp.manager";
pub const PUBLISHER: &str = "DN263AX69U";
pub const APP: &str = "Inkscape MCP Manager.app";
pub const HELPER: &str = "Contents/Helpers/inkscape-mcp-launcher";
pub const EXECUTABLE: &str = "Contents/MacOS/inkscape-mcp-manager";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub tag: String,
    pub prerelease: bool,
    pub os: String,
    pub architecture: String,
    pub minimum_os_major: u32,
    pub bundle_identifier: String,
    pub publisher: String,
    pub manager_build: String,
    pub manager_revision: String,
    pub bootstrap: super::install::BootstrapIdentity,
    pub launcher_version: u32,
    pub component_format: u32,
    pub text_interface: u32,
    pub helper_protocol: u32,
    pub asset: Asset,
    pub files: BTreeMap<String, FileIdentity>,
}
impl Manifest {
    pub fn validate(&self) -> Result<()> {
        if self.format != 1
            || !identifier(&self.tag)
            || self.bundle_identifier != BUNDLE
            || self.publisher != PUBLISHER
            || !identifier(&self.manager_build)
            || !identifier(&self.manager_revision)
            || self.bootstrap.format != 1
            || !identifier(&self.bootstrap.build_id)
            || !identifier(&self.bootstrap.source_revision)
            || self.launcher_version < 2
            || self.component_format == 0
            || self.text_interface == 0
            || self.helper_protocol == 0
            || self.minimum_os_major < 15
            || self.files.is_empty()
            || self.files.len() > 50000
        {
            return Err("invalid Manager update identity/contract".into());
        }
        self.asset.validate(512 * 1024 * 1024)?;
        self.bootstrap.executable.validate(128 * 1024 * 1024)?;
        let mut total = 0u64;
        for (name, file) in &self.files {
            if !managed_relative(name) {
                return Err("unsafe Manager inventory path".into());
            }
            // Empty sealed resources are permitted; the archive itself must not be empty.
            if !hash(&file.sha256) {
                return Err("invalid Manager inventory hash".into());
            }
            total = total
                .checked_add(file.bytes)
                .ok_or("Manager inventory overflow")?;
            if total > 512 * 1024 * 1024 {
                return Err("Manager inventory exceeds limit".into());
            }
        }
        if self.files.get(HELPER) != Some(&self.bootstrap.executable)
            || !self.files.contains_key(EXECUTABLE)
            || !self.files.contains_key("Contents/Info.plist")
        {
            return Err("incomplete Manager inventory".into());
        }
        Ok(())
    }
    pub fn compatible(&self, os: &str, architecture: &str, major: u32) -> bool {
        self.os == os
            && self.architecture == architecture
            && os == "macos"
            && architecture == "aarch64"
            && major >= self.minimum_os_major
    }
    pub fn preserves_pair(&self, pair: &super::install::Pair) -> Result<()> {
        if pair.runtime.format != self.component_format
            || pair.instructions.format != self.component_format
            || pair.runtime.text_interface != self.text_interface
            || pair.instructions.text_interface != self.text_interface
            || pair.runtime.helper_protocol != self.helper_protocol
            || pair.runtime.launcher_minimum > self.launcher_version
            || pair.instructions.launcher_minimum > self.launcher_version
        {
            return Err("The target Manager/bootstrap cannot preserve the selected runtime/text or rollback pair. Choose a compatible Manager release; no components were changed.".into());
        }
        Ok(())
    }
}

/// No filesystem mutation, runtime compatibility filtering or settings dependency.
pub fn discover(transport: &impl Transport, channel: &str, major: u32) -> Result<Option<Manifest>> {
    discover_inner(
        transport,
        channel,
        (std::env::consts::OS, std::env::consts::ARCH, major),
        None,
    )
}
pub fn discover_newer(
    transport: &impl Transport,
    channel: &str,
    major: u32,
    installed: &BTreeMap<String, String>,
) -> Result<Option<Manifest>> {
    discover_inner(
        transport,
        channel,
        (std::env::consts::OS, std::env::consts::ARCH, major),
        Some(installed),
    )
}
fn discover_inner(
    transport: &impl Transport,
    channel: &str,
    target: (&str, &str, u32),
    installed: Option<&BTreeMap<String, String>>,
) -> Result<Option<Manifest>> {
    if !matches!(channel, "stable" | "prerelease") {
        return Err("invalid Manager channel".into());
    }
    for page in 1..=10 {
        let bytes = transport.get(&format!("https://api.github.com/repos/P1oN/inkscape-mcp-server/releases?per_page=100&page={page}"), 4 * 1024 * 1024)?;
        let releases: Vec<Value> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        for release in &releases {
            let tag = release["tag_name"].as_str().ok_or("release tag missing")?;
            let prerelease = release["prerelease"]
                .as_bool()
                .ok_or("release channel missing")?;
            if release["draft"] != false
                || release["immutable"] != true
                || channel == "stable" && prerelease
            {
                continue;
            }
            if !release["assets"]
                .as_array()
                .ok_or("release assets missing")?
                .iter()
                .any(|a| a["name"] == ASSET)
            {
                continue;
            }
            let bytes = transport.get(&download::asset_url(tag, ASSET)?, 2 * 1024 * 1024)?;
            let manifest: Manifest = serde_json::from_slice(&bytes)
                .map_err(|e| format!("invalid Manager manifest: {e}"))?;
            manifest.validate()?;
            if manifest.tag != tag || manifest.prerelease != prerelease {
                return Err("Manager manifest release/channel mismatch".into());
            }
            if manifest.compatible(target.0, target.1, target.2)
                && installed.is_none_or(|baseline| eligible(&manifest, baseline))
            {
                return Ok(Some(manifest));
            }
        }
        if releases.len() < 100 {
            break;
        }
    }
    Ok(None)
}

pub fn info(app: &Path) -> Result<BTreeMap<String, String>> {
    let bytes = super::instructions::read(&app.join("Contents/Info.plist"), 1024 * 1024)?;
    let plist =
        plist::Value::from_reader(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let dict = plist.as_dictionary().ok_or("invalid Manager Info.plist")?;
    let mut result: BTreeMap<String, String> = dict
        .iter()
        .filter_map(|(k, v)| v.as_string().map(|s| (k.clone(), s.to_owned())))
        .collect();
    result.insert(
        "VerifiedHelperSHA256".into(),
        FileIdentity::of(&super::instructions::read(
            &app.join(HELPER),
            128 * 1024 * 1024,
        )?)
        .sha256,
    );
    Ok(result)
}
pub fn eligible(manifest: &Manifest, installed: &BTreeMap<String, String>) -> bool {
    let newer = installed
        .get("InkscapeMCPDistribution")
        .and_then(|tag| semver::Version::parse(tag.trim_start_matches('v')).ok());
    let target = semver::Version::parse(manifest.tag.trim_start_matches('v')).ok();
    (installed.get("InkscapeMCPBuildID") != Some(&manifest.manager_build)
        || installed.get("VerifiedHelperSHA256") != Some(&manifest.bootstrap.executable.sha256))
        && matches!((target, newer), (Some(target), Some(old)) if target > old)
}
pub fn check(app: &Path, transport: &impl Transport, channel: &str, major: u32) -> Result<Value> {
    let installed = info(app)?;
    let available = discover_newer(transport, channel, major, &installed)?;
    Ok(
        json!({"manager_update_available":available.as_ref().is_some_and(|m| eligible(m,&installed)),"manager_available":available,"manager_installed":installed}),
    )
}

/// Verify every byte and the fixed trusted publisher before any downloaded code runs.
pub fn verify(app: &Path, manifest: &Manifest) -> Result<()> {
    manifest.validate()?;
    if inventory(app, 512 * 1024 * 1024)? != manifest.files {
        return Err("Manager complete inventory differs".into());
    }
    let info = info(app)?;
    for (key, expected) in [
        ("CFBundleIdentifier", BUNDLE),
        ("CFBundleExecutable", "inkscape-mcp-manager"),
        ("InkscapeMCPBuildID", &manifest.manager_build),
        ("InkscapeMCPDistribution", &manifest.tag),
        ("InkscapeMCPRevision", &manifest.manager_revision),
    ] {
        if info.get(key).map(String::as_str) != Some(expected) {
            return Err(format!("Manager {key} differs from manifest"));
        }
    }
    for path in [app.to_owned(), app.join(EXECUTABLE), app.join(HELPER)] {
        if crate::client_management::signature_team(&path)
            .map_err(|e| e.to_string())?
            .as_deref()
            != Some(PUBLISHER)
        {
            return Err("Manager publisher is not the trusted Developer ID publisher".into());
        }
    }
    crate::client_management::bounded_output(
        std::process::Command::new("/usr/bin/codesign")
            .args(["--verify", "--deep", "--strict"])
            .arg(app),
        8192,
    )
    .map_err(|_| "Manager signature verification failed".to_string())?;
    let actual = bootstrap_info(&app.join(HELPER))?;
    if actual["component_format"] != manifest.component_format
        || actual["text_interface"] != manifest.text_interface
        || actual["helper_protocol"] != manifest.helper_protocol
        || actual["bootstrap"]
            != serde_json::to_value(&manifest.bootstrap).map_err(|e| e.to_string())?
        || actual["launcher_version"] != manifest.launcher_version
    {
        return Err("Manager helper compiled identity differs from manifest".into());
    }
    Ok(())
}
/// Caller must verify the publisher and enclosing app before invoking this.
pub fn bootstrap_info(helper: &Path) -> Result<Value> {
    let bytes = crate::client_management::bounded_output(
        std::process::Command::new(helper)
            .env_clear()
            .env("HOME", "/")
            .args(["bootstrap-info", "--install-dir", "/", "--json"]),
        8192,
    )
    .map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
pub(super) fn fixture_manifest() -> Manifest {
    let helper = FileIdentity::of(b"new bootstrap");
    Manifest {
        format: 1,
        tag: "v2.0.0".into(),
        prerelease: false,
        os: "macos".into(),
        architecture: "aarch64".into(),
        minimum_os_major: 15,
        bundle_identifier: BUNDLE.into(),
        publisher: PUBLISHER.into(),
        manager_build: "manager-two".into(),
        manager_revision: "a".repeat(40),
        bootstrap: super::install::BootstrapIdentity {
            format: 1,
            build_id: "bootstrap-two".into(),
            source_revision: "b".repeat(40),
            executable: helper.clone(),
        },
        launcher_version: 3,
        component_format: FORMAT,
        text_interface: TEXT_INTERFACE,
        helper_protocol: HELPER_PROTOCOL,
        asset: Asset {
            name: "inkscape-mcp-launcher.tar.gz".into(),
            identity: FileIdentity::of(b"archive"),
        },
        files: BTreeMap::from([
            (HELPER.into(), helper),
            (EXECUTABLE.into(), FileIdentity::of(b"manager")),
            ("Contents/Info.plist".into(), FileIdentity::of(b"plist")),
        ]),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    // Discovery fixtures describe the supported macOS target on every test host.
    fn discover(t: &impl Transport, channel: &str, major: u32) -> Result<Option<Manifest>> {
        discover_inner(t, channel, ("macos", "aarch64", major), None)
    }
    fn discover_newer(
        t: &impl Transport,
        channel: &str,
        major: u32,
        installed: &BTreeMap<String, String>,
    ) -> Result<Option<Manifest>> {
        discover_inner(t, channel, ("macos", "aarch64", major), Some(installed))
    }
    struct Network {
        manifests: Vec<Manifest>,
        calls: RefCell<Vec<String>>,
        error: Option<&'static str>,
    }
    impl Transport for Network {
        fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>> {
            self.calls.borrow_mut().push(url.into());
            if let Some(error) = self.error {
                return Err(error.into());
            }
            let bytes = if url.contains("api.github.com") {
                serde_json::to_vec(&self.manifests.iter().map(|m|json!({"tag_name":m.tag,"prerelease":m.prerelease,"draft":false,"immutable":true,"assets":[{"name":ASSET},{"name":download::MANIFEST_ASSET}]})).collect::<Vec<_>>()).unwrap()
            } else {
                let m = self
                    .manifests
                    .iter()
                    .find(|m| url == download::asset_url(&m.tag, ASSET).unwrap())
                    .expect("runtime manifest was incorrectly requested");
                serde_json::to_vec(m).unwrap()
            };
            assert!(bytes.len() as u64 <= limit);
            Ok(bytes)
        }
    }
    fn network(manifests: Vec<Manifest>) -> Network {
        Network {
            manifests,
            calls: RefCell::new(vec![]),
            error: None,
        }
    }
    fn app(root: &Path) -> std::path::PathBuf {
        let app = root.join(APP);
        mkdir(&app.join("Contents/Helpers")).unwrap();
        write(&app.join(HELPER), b"old bootstrap").unwrap();
        write(&app.join("Contents/Info.plist"),br#"<plist version="1.0"><dict><key>CFBundleIdentifier</key><string>org.inkscape-mcp.manager</string><key>InkscapeMCPDistribution</key><string>v1.0.0</string><key>InkscapeMCPBuildID</key><string>manager-one</string></dict></plist>"#).unwrap();
        app
    }
    #[test]
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    fn independent_discovery_ignores_incompatible_runtime_and_never_creates_settings() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let app = app(&root);
        let before = inventory(&root, 1024 * 1024).unwrap();
        let n = network(vec![fixture_manifest()]); // launcher_version 3, current launcher 2
        let result = check(&app, &n, "stable", 15).unwrap();
        assert_eq!(result["manager_update_available"], true);
        assert_eq!(inventory(&root, 1024 * 1024).unwrap(), before);
        assert!(!root.join("settings.json").exists());
        assert_eq!(n.calls.borrow().len(), 2);
        assert!(n.calls.borrow()[1].ends_with(ASSET));
    }
    #[test]
    fn discovery_channels_target_and_network_refusals() {
        let stable = fixture_manifest();
        let mut preview = stable.clone();
        preview.tag = "v3.0.0-beta.1".into();
        preview.prerelease = true;
        let n = network(vec![preview.clone(), stable.clone()]);
        assert_eq!(discover(&n, "stable", 15).unwrap(), Some(stable.clone()));
        assert_eq!(discover(&n, "prerelease", 15).unwrap(), Some(preview));
        let mut wrong = stable.clone();
        wrong.architecture = "x86_64".into();
        assert_eq!(discover(&network(vec![wrong]), "stable", 15).unwrap(), None);
        assert_eq!(
            discover(&network(vec![stable]), "stable", 14).unwrap(),
            None
        );
        assert_eq!(discover(&network(vec![]), "stable", 15).unwrap(), None);
        for error in ["offline", "GitHub 429", "timeout"] {
            let mut n = network(vec![]);
            n.error = Some(error);
            assert_eq!(discover(&n, "stable", 15).unwrap_err(), error);
        }
        assert!(discover(&network(vec![]), "other", 15).is_err());
    }
    #[test]
    fn mutable_releases_are_skipped_and_an_old_release_does_not_hide_a_newer_build() {
        struct Mutable(Network);
        impl Transport for Mutable {
            fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>> {
                assert!(
                    url.contains("api.github.com"),
                    "mutable asset was requested"
                );
                let bytes = self.0.get(url, limit)?;
                let mut releases: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
                for release in &mut releases {
                    release["immutable"] = json!(false);
                }
                Ok(serde_json::to_vec(&releases).unwrap())
            }
        }
        assert_eq!(
            discover(&Mutable(network(vec![fixture_manifest()])), "stable", 15).unwrap(),
            None
        );
        let target = fixture_manifest();
        let mut old = target.clone();
        old.tag = "v0.9.0".into();
        let baseline = BTreeMap::from([
            ("InkscapeMCPDistribution".into(), "v1.0.0".into()),
            ("InkscapeMCPBuildID".into(), "old-manager".into()),
        ]);
        assert_eq!(
            discover_newer(&network(vec![old, target.clone()]), "stable", 15, &baseline).unwrap(),
            Some(target)
        );
    }
    #[test]
    fn eligibility_uses_real_manager_and_bootstrap_ids_with_monotonic_versions() {
        let mut installed = BTreeMap::from([
            ("InkscapeMCPDistribution".into(), "v1.0.0".into()),
            ("InkscapeMCPBuildID".into(), "manager-one".into()),
            (
                "VerifiedHelperSHA256".into(),
                FileIdentity::of(b"old").sha256,
            ),
        ]);
        let mut target = fixture_manifest();
        assert!(eligible(&target, &installed));
        for version in ["v0.9.0", "v1.0.0"] {
            target.tag = version.into();
            assert!(!eligible(&target, &installed));
        }
        target.tag = "v2.0.0".into();
        installed.insert("InkscapeMCPBuildID".into(), target.manager_build.clone());
        assert!(
            eligible(&target, &installed),
            "a bootstrap-only change must be offered"
        );
        installed.insert(
            "VerifiedHelperSHA256".into(),
            target.bootstrap.executable.sha256.clone(),
        );
        assert!(!eligible(&target, &installed));
    }
    #[test]
    fn manifest_and_legacy_contracts_are_independent_and_strict() {
        let valid = fixture_manifest();
        valid.validate().unwrap();
        for field in [
            "format",
            "publisher",
            "bundle_identifier",
            "bootstrap",
            "files",
        ] {
            let mut value = serde_json::to_value(&valid).unwrap();
            value[field] = Value::Null;
            assert!(serde_json::from_value::<Manifest>(value).is_err());
        }
        for (key, bad) in [
            ("publisher", "FOREIGN123"),
            ("bundle_identifier", "other.app"),
            ("manager_build", "../bad"),
        ] {
            let mut value = serde_json::to_value(&valid).unwrap();
            value[key] = json!(bad);
            assert!(
                serde_json::from_value::<Manifest>(value)
                    .unwrap()
                    .validate()
                    .is_err()
            );
        }
        let mut unsafe_manifest = valid.clone();
        unsafe_manifest
            .files
            .insert("../escape".into(), FileIdentity::of(b"x"));
        assert!(unsafe_manifest.validate().is_err());
        let text = super::super::instructions::manifest(
            "v2",
            &super::super::instructions::default_files(),
        );
        let legacy_manifest = ReleaseManifest {
            format: 1,
            tag: valid.tag.clone(),
            prerelease: false,
            runtime: RuntimeManifest {
                format: 1,
                distribution_tag: valid.tag.clone(),
                build_id: "runtime-different".into(),
                source_revision: "c".repeat(40),
                os: "macos".into(),
                architecture: "aarch64".into(),
                minimum_os_major: 15,
                text_interface: 1,
                helper_protocol: HELPER_PROTOCOL,
                launcher_minimum: LAUNCHER_VERSION + 1,
                asset: valid.asset.clone(),
            },
            instructions: text,
            instruction_asset: valid.asset.clone(),
            launcher_asset: valid.asset.clone(),
        };
        assert!(
            legacy_manifest
                .compatible("macos", "aarch64", 15)
                .unwrap_err()
                .contains("launcher upgrade required")
        );
        assert_eq!(
            discover(&network(vec![valid.clone()]), "stable", 15).unwrap(),
            Some(valid.clone())
        );
        let mut legacy = serde_json::to_value(legacy_manifest).unwrap();
        serde_json::from_value::<ReleaseManifest>(legacy.clone()).unwrap();
        // Manager fields cannot be parsed as, or accidentally added to, the old contract.
        assert!(
            serde_json::from_value::<ReleaseManifest>(serde_json::to_value(&valid).unwrap())
                .is_err()
        );
        legacy["manager"] = json!(valid);
        assert!(serde_json::from_value::<ReleaseManifest>(legacy).is_err());
    }
    #[test]
    fn signature_and_content_refusals_do_not_execute_unsigned_helpers() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let app = app(&root);
        mkdir(&app.join("Contents/MacOS")).unwrap();
        write(&app.join(EXECUTABLE), b"not signed code").unwrap();
        let mut m = fixture_manifest();
        m.files = inventory(&app, 1024 * 1024).unwrap();
        m.bootstrap.executable = m.files[HELPER].clone();
        assert!(verify(&app, &m).is_err());
        write(&app.join(HELPER), b"changed").unwrap();
        assert_eq!(
            verify(&app, &m).unwrap_err(),
            "Manager complete inventory differs"
        );
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn intact_ad_hoc_bundle_is_rejected_before_helper_execution() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let app = root.join(APP);
        mkdir(&app.join("Contents/MacOS")).unwrap();
        mkdir(&app.join("Contents/Helpers")).unwrap();
        std::fs::copy("/usr/bin/true", app.join(EXECUTABLE)).unwrap();
        std::fs::copy("/usr/bin/true", app.join(HELPER)).unwrap();
        write(&app.join("Contents/Info.plist"),format!(r#"<plist version="1.0"><dict><key>CFBundleIdentifier</key><string>{BUNDLE}</string><key>CFBundleExecutable</key><string>inkscape-mcp-manager</string><key>CFBundlePackageType</key><string>APPL</string><key>InkscapeMCPDistribution</key><string>v2.0.0</string><key>InkscapeMCPBuildID</key><string>manager-two</string><key>InkscapeMCPRevision</key><string>{}</string></dict></plist>"#,"a".repeat(40)).as_bytes()).unwrap();
        crate::client_management::bounded_output(
            std::process::Command::new("/usr/bin/codesign")
                .args(["--force", "--sign", "-"])
                .arg(&app),
            8192,
        )
        .unwrap();
        let mut m = fixture_manifest();
        m.files = inventory(&app, 1024 * 1024).unwrap();
        m.bootstrap.executable = m.files[HELPER].clone();
        assert_eq!(
            verify(&app, &m).unwrap_err(),
            "Manager publisher is not the trusted Developer ID publisher"
        );
    }
}
