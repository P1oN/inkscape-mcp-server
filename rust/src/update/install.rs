use super::{
    instructions::{Instructions, guarded, read},
    manifests::*,
    storage::*,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    pub runtime: RuntimeManifest,
    pub instructions: InstructionManifest,
    pub tag: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Selector {
    pub format: u32,
    pub current: Pair,
    pub previous: Option<Pair>,
    pub channel: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub legacy_skill_owner: PathBuf,
    pub format: u32,
    pub environment: BTreeMap<String, String>,
    pub skills: Vec<PathBuf>,
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        if self.format != FORMAT || self.skills.len() > 2 {
            return Err("unsupported settings format/client count".into());
        }
        let allowed = [
            "INKSCAPE_MCP_WORKSPACE_ROOTS",
            "INKSCAPE_MCP_LIVE_ENABLED",
            "INKSCAPE_MCP_ENGINE_MODE",
            "SENTRY_DSN",
            "SENTRY_ENVIRONMENT",
            "SENTRY_TRACES_SAMPLE_RATE",
            "PATH",
        ];
        for (name, value) in &self.environment {
            if !allowed.contains(&name.as_str())
                || value.len() > 8192
                || value.contains(['\n', '\r', '\0'])
            {
                return Err("invalid saved setting".into());
            }
        }
        for name in ["INKSCAPE_MCP_WORKSPACE_ROOTS", "PATH"] {
            if self
                .environment
                .get(name)
                .is_none_or(|v| v.is_empty() || !v.starts_with('/'))
            {
                return Err("missing/invalid workspace or Inkscape settings".into());
            }
        }
        if self
            .environment
            .get("INKSCAPE_MCP_LIVE_ENABLED")
            .is_none_or(|v| !matches!(v.as_str(), "true" | "false"))
            || self
                .environment
                .get("INKSCAPE_MCP_ENGINE_MODE")
                .is_none_or(|v| !matches!(v.as_str(), "per_call" | "shell"))
        {
            return Err("invalid live/engine settings".into());
        }
        if !self.legacy_skill_owner.is_absolute() {
            return Err("invalid legacy skill owner".into());
        }
        if self
            .skills
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != self.skills.len()
        {
            return Err("duplicate managed skill destinations".into());
        }
        for skill in &self.skills {
            guarded(skill.parent().ok_or("skill destination has no parent")?)?;
            if skill.exists() || skill.is_symlink() {
                guarded(skill)?;
            }
        }
        Ok(())
    }
}
pub fn runtime_path(root: &Path, pair: &Pair) -> Result<PathBuf> {
    if !identifier(&pair.runtime.build_id) {
        return Err("invalid runtime identity".into());
    }
    Ok(root.join("runtime").join(&pair.runtime.build_id))
}
pub fn instruction_path(root: &Path, pair: &Pair) -> Result<PathBuf> {
    instruction_manifest_path(root, &pair.instructions)
}
fn instruction_manifest_path(root: &Path, manifest: &InstructionManifest) -> Result<PathBuf> {
    manifest.validate()?;
    let legacy = root.join("instructions").join(&manifest.content_id);
    if legacy.exists() && Instructions::load(&legacy)?.manifest == *manifest {
        return Ok(legacy);
    }
    // Labels and compatibility metadata are part of the immutable selected bundle.
    // Equal text bytes across releases must not overwrite an older rollback bundle.
    let identity = FileIdentity::of(&serde_json::to_vec(manifest).map_err(|e| e.to_string())?);
    Ok(root
        .join("instructions")
        .join(format!("{}-{}", manifest.content_id, identity.sha256)))
}
pub fn validate_pair(root: &Path, pair: &Pair) -> Result<()> {
    let r = &pair.runtime;
    if r.format != FORMAT
        || r.text_interface != TEXT_INTERFACE
        || r.helper_protocol != HELPER_PROTOCOL
        || r.launcher_minimum > LAUNCHER_VERSION
    {
        return Err("incompatible selected runtime/text/helper protocol; recover using rollback or a compatible combined update".into());
    }
    let bundle = Instructions::load(&instruction_path(root, pair)?)?;
    if bundle.manifest != pair.instructions {
        return Err("selected instruction identity differs".into());
    }
    verify_runtime(&runtime_path(root, pair)?, r)
}
pub fn verify_runtime(path: &Path, runtime: &RuntimeManifest) -> Result<()> {
    let expected: BTreeMap<String, FileIdentity> = json(&path.join("FILES.json"))?;
    let mut actual = inventory_with_local(path, 1024 * 1024 * 1024, true)?;
    actual.remove("FILES.json");
    if actual != expected {
        return Err("runtime file inventory/checksums differ".into());
    }
    let library = crate::runtime_layout::library(path);
    let metadata: Value = json(&library.join("package.json"))?;
    if library.ends_with("Contents/Resources") {
        if metadata["layout"] != "macos-runtime-app-v2"
            || runtime.launcher_minimum < 2
            || path.join("libexec/inkscape-mcp/package.json").exists()
        {
            return Err("invalid or ambiguous runtime bundle layout/compatibility".into());
        }
        verify_bundle_signer(path, &metadata)?;
    }
    if metadata["build_info"]["build_id"] != runtime.build_id
        || metadata["build_info"]["revision"] != runtime.source_revision
    {
        return Err("runtime build/revision mismatch".into());
    }
    for name in [
        "inkscape-mcp",
        "inkscape-mcp-client",
        "inkscape-mcp-supervisor",
        "inkscape-mcp-inx",
        "inkscape-mcp-live",
    ] {
        guarded(&crate::runtime_layout::binary(&library, name))?;
    }
    Ok(())
}
fn verify_bundle_signer(path: &Path, metadata: &Value) -> Result<()> {
    if !cfg!(target_os = "macos") {
        return Err("macOS runtime bundle requires macOS".into());
    }
    let actor = std::env::current_exe().map_err(|e| e.to_string())?;
    let team = crate::client_management::signature_team(&actor).map_err(|e| e.to_string())?;
    if let Some(team) = team {
        let requirement =
            format!("=anchor apple generic and certificate leaf[subject.OU] = \"{team}\"");
        let code = metadata["code_inventory"]
            .as_array()
            .filter(|c| c.len() >= 9 && c.len() <= 64)
            .ok_or("runtime native code inventory missing")?;
        for name in code {
            let name = name.as_str().ok_or("invalid runtime code inventory")?;
            if !managed_relative(name)
                || !(name.starts_with("Inkscape MCP Runtime.app/Contents/MacOS/")
                    || name.starts_with("Inkscape MCP Runtime.app/Contents/Frameworks/"))
            {
                return Err("runtime code path is outside supported locations".into());
            }
            crate::client_management::bounded_output(
                Command::new("/usr/bin/codesign")
                    .args(["--verify", "--strict", "-R", &requirement])
                    .arg(path.join(name)),
                8192,
            )
            .map_err(|_| {
                "runtime code signer differs from the trusted installer; preserved".to_string()
            })?;
        }
        crate::client_management::bounded_output(
            Command::new("/usr/bin/codesign")
                .args(["--verify", "--deep", "--strict", "-R", &requirement])
                .arg(path.join(crate::runtime_layout::RUNTIME_APP)),
            8192,
        )
        .map_err(|_| "runtime bundle signer differs from the trusted installer".to_string())?;
    } else if metadata["release_signed"] == true {
        return Err("signed runtime requires its verified signed Manager/launcher; upgrade the Manager first".into());
    }
    Ok(())
}
pub fn command(root: &Path, pair: &Pair, settings: &Settings) -> Result<Command> {
    settings.validate()?;
    validate_pair(root, pair)?;
    let mut command = Command::new(crate::runtime_layout::binary(
        &crate::runtime_layout::library(&runtime_path(root, pair)?),
        "inkscape-mcp",
    ));
    command.envs(&settings.environment).env(
        "INKSCAPE_MCP_INSTRUCTION_BUNDLE",
        instruction_path(root, pair)?,
    );
    Ok(command)
}
pub fn probe(root: &Path, pair: &Pair, settings: &Settings) -> Result<()> {
    let mut version = command(root, pair, settings)?;
    version.arg("--version");
    let identity: Value = serde_json::from_slice(
        &crate::client_management::bounded_output(&mut version, 8192).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if identity["build_id"] != pair.runtime.build_id
        || identity["revision"] != pair.runtime.source_revision
    {
        return Err("candidate executable identity differs from manifest".into());
    }
    crate::client_management::probe(&mut command(root, pair, settings)?, Duration::from_secs(30))
        .map_err(|e| format!("candidate STDIO/workspace check failed: {e}"))?;
    Ok(())
}
pub fn doctor(root: &Path, pair: &Pair, settings: &Settings) -> Result<()> {
    let mut candidate = command(root, pair, settings)?;
    candidate.arg("--doctor");
    let bytes =
        crate::client_management::bounded_output(&mut candidate, 128 * 1024).map_err(|e| {
            format!("candidate doctor refused: {e}; check Inkscape/package prerequisites")
        })?;
    let report: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if report["ready"] != true {
        return Err("candidate doctor reports incomplete prerequisites".into());
    }
    Ok(())
}
pub fn selector(root: &Path) -> Result<Selector> {
    let active: Selector = json(&root.join("active.json"))?;
    if active.format != FORMAT || !matches!(active.channel.as_str(), "stable" | "prerelease") {
        return Err("unsupported active selector/channel".into());
    }
    Ok(active)
}
pub fn report(active: &Selector) -> Value {
    let actor_signing = if cfg!(target_os = "macos") {
        match std::env::current_exe()
            .map_err(|e| e.to_string())
            .and_then(|path| {
                crate::client_management::signature_team(&path).map_err(|e| e.to_string())
            }) {
            Ok(Some(team)) => json!({"status":"developer_id_verified","team_id":team}),
            Ok(None) => json!({"status":"ad_hoc","team_id":null}),
            Err(_) => json!({"status":"unverified","team_id":null}),
        }
    } else {
        json!({"status":"not_applicable","team_id":null})
    };
    json!({"launcher_version":LAUNCHER_VERSION,"launcher_build":env!("INKSCAPE_MCP_BUILD_ID"),"launcher_revision":env!("INKSCAPE_MCP_REVISION"),"channel":active.channel,"distribution":active.current.tag,"runtime_build":active.current.runtime.build_id,"runtime_revision":active.current.runtime.source_revision,"instructions_version":active.current.instructions.version,"instructions_content":active.current.instructions.content_id,"rollback_available":active.previous.is_some(),"manager_helper_signing":actor_signing,"notarization_assessment":"not_checked_on_this_host"})
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapIdentity {
    pub format: u32,
    pub build_id: String,
    pub source_revision: String,
    pub executable: FileIdentity,
}
pub fn bootstrap_identity(bytes: &[u8]) -> BootstrapIdentity {
    BootstrapIdentity {
        format: 1,
        build_id: env!("INKSCAPE_MCP_BUILD_ID").into(),
        source_revision: env!("INKSCAPE_MCP_REVISION").into(),
        executable: FileIdentity::of(bytes),
    }
}
/// Discovery reports saved bootstrap identity only after matching its actual bytes.
/// Never execute an old launcher merely to learn its version.
pub fn installation_report(root: &Path, active: &Selector) -> Value {
    let mut result = report(active);
    result["installer_build"] = result["launcher_build"].clone();
    result["launcher_build"] = Value::Null;
    result["launcher_revision"] = Value::Null;
    let identity = root.join("bootstrap.json");
    if identity.exists() || identity.is_symlink() {
        let verified = (|| -> Result<BootstrapIdentity> {
            let saved: BootstrapIdentity = super::storage::json(&identity)?;
            if saved.format != 1
                || !identifier(&saved.build_id)
                || saved.build_id.len() > 128
                || saved.executable.bytes > 128 * 1024 * 1024
                || !hash(&saved.executable.sha256)
                || !(saved.source_revision == "unknown"
                    || saved.source_revision.len() == 40
                        && saved.source_revision.bytes().all(|b| b.is_ascii_hexdigit()))
            {
                return Err("invalid bootstrap identity".into());
            }
            for name in ["bin/inkscape-mcp-launcher", "bin/inkscape-mcp"] {
                if FileIdentity::of(&read(&root.join(name), saved.executable.bytes)?)
                    != saved.executable
                {
                    return Err("bootstrap bytes changed".into());
                }
            }
            Ok(saved)
        })();
        match verified {
            Ok(saved) => {
                result["launcher_build"] = json!(saved.build_id);
                result["launcher_revision"] = json!(saved.source_revision);
            }
            Err(_) => result["bootstrap_damaged"] = json!(true),
        }
    } else if !root.join("bin/inkscape-mcp-launcher").is_file()
        || !root.join("bin/inkscape-mcp").is_file()
    {
        result["bootstrap_damaged"] = json!(true);
    }
    result
}
pub fn stage_instructions(
    root: &Path,
    source: &Path,
    manifest: &InstructionManifest,
) -> Result<()> {
    let loaded = Instructions::load(source)?;
    if &loaded.manifest != manifest {
        return Err("instruction artifact identity mismatch".into());
    }
    let target = instruction_manifest_path(root, manifest)?;
    if target.exists() {
        if Instructions::load(&target)?.manifest != *manifest {
            return Err("occupied instruction identity".into());
        }
        return Ok(());
    }
    let stage = tempfile::Builder::new()
        .prefix("instructions-")
        .tempdir_in(root.join("staging"))
        .map_err(|e| e.to_string())?;
    let candidate = stage.path().join("bundle");
    copy_tree(source, &candidate, 2 * 1024 * 1024 + 64 * 1024)?;
    if Instructions::load(&candidate)?.manifest != *manifest {
        return Err("staged instruction identity changed".into());
    }
    fs::rename(candidate, &target).map_err(|e| e.to_string())?;
    sync_dir(target.parent().unwrap())
}
pub fn stage_runtime(root: &Path, source: &Path, manifest: &RuntimeManifest) -> Result<()> {
    verify_runtime(source, manifest)?;
    let target = root.join("runtime").join(&manifest.build_id);
    if target.exists() {
        return verify_runtime(&target, manifest);
    }
    let stage = tempfile::Builder::new()
        .prefix("runtime-")
        .tempdir_in(root.join("staging"))
        .map_err(|e| e.to_string())?;
    let candidate = stage.path().join("runtime");
    mkdir(&candidate)?;
    let files = inventory_with_local(source, 1024 * 1024 * 1024, true)?;
    for (name, id) in files {
        let target = candidate.join(&name);
        mkdir(target.parent().unwrap())?;
        write(&target, &read(&source.join(&name), id.bytes)?)?;
        fs::set_permissions(
            &target,
            fs::metadata(source.join(name))
                .map_err(|e| e.to_string())?
                .permissions(),
        )
        .map_err(|e| e.to_string())?;
    }

    verify_runtime(&candidate, manifest)?;
    fs::rename(candidate, &target).map_err(|e| e.to_string())?;
    sync_dir(target.parent().unwrap())
}
pub fn import_settings(repo: &Path) -> Result<(PathBuf, Settings)> {
    let config = String::from_utf8(read(&repo.join(".inkscape-mcp-local/setup.conf"), 8192)?)
        .map_err(|e| e.to_string())?;
    let lines: Vec<_> = config.lines().collect();
    if lines.len() != 6 || lines[0] != "inkscape-mcp-setup-v1" {
        return Err("unsupported legacy setup configuration; rerun source setup first".into());
    }
    let binary = Path::new(lines[1]);
    if !binary.is_absolute()
        || binary.file_name().is_none_or(|n| n != "inkscape-mcp")
        || binary
            .parent()
            .and_then(Path::file_name)
            .is_none_or(|n| n != "bin")
    {
        return Err("invalid configured runtime path".into());
    }
    let mut environment = BTreeMap::from([
        ("INKSCAPE_MCP_WORKSPACE_ROOTS".into(), lines[3].into()),
        ("INKSCAPE_MCP_LIVE_ENABLED".into(), lines[4].into()),
        ("INKSCAPE_MCP_ENGINE_MODE".into(), lines[5].into()),
        ("PATH".into(), format!("{}:/usr/bin:/bin", lines[2])),
    ]);
    let sentry = repo.join(".inkscape-mcp-local/sentry.conf");
    if sentry.exists() || sentry.is_symlink() {
        let text = String::from_utf8(read(&sentry, 4096)?).map_err(|e| e.to_string())?;
        let data: Vec<_> = text.lines().collect();
        if data.len() != 4 || data[0] != "inkscape-mcp-sentry-v1" {
            return Err("invalid saved telemetry configuration".into());
        }
        match data[1] {
            "true" => {
                environment.insert("SENTRY_DSN".into(), data[2].into());
                environment.insert("SENTRY_ENVIRONMENT".into(), data[3].into());
            }
            "false" if data[2].is_empty() => {
                environment.insert("SENTRY_TRACES_SAMPLE_RATE".into(), "0".into());
            }
            _ => return Err("invalid saved telemetry enablement".into()),
        }
    }
    let home = PathBuf::from(std::env::var_os("HOME").ok_or("HOME missing")?);
    let codex = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".codex"));
    let skills: std::collections::BTreeSet<_> = [
        codex.join("skills/inkscape-mcp"),
        home.join(".claude/skills/inkscape-mcp"),
    ]
    .into_iter()
    .filter(|p| p.exists())
    .filter(|p| {
        fs::read_to_string(p.join(".inkscape-mcp-owner"))
            .is_ok_and(|s| s.trim() == repo.to_string_lossy())
    })
    .collect();
    let settings = Settings {
        format: FORMAT,
        legacy_skill_owner: repo.into(),
        environment,
        skills: skills.into_iter().collect(),
    };
    settings.validate()?;
    Ok((binary.parent().unwrap().parent().unwrap().into(), settings))
}
