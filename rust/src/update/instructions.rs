//! Text only. Schemas, prompt arguments and executable authority stay compiled.
use super::manifests::*;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Component, Path},
    sync::OnceLock,
};

pub const GUIDANCE_MARKER: &str = "\n\nEditable vector authoring quality:\n";
pub const GUIDANCE_SLOT: &str = "__INKSCAPE_AUTHORING_GUIDANCE_V1__";
pub const GOAL_SLOT: &str = "__MIGRATION_GOAL_SLOT_9a6c0__";
const CONTRACT: &str =
    include_str!("../../../migration/contracts/live-true_raw-false_full_full.json");
const PROMPTS: &str = include_str!("../../../migration/contracts/prompt-messages.json");
const GUIDANCE: &str = include_str!("../../../migration/contracts/authoring-guidance.txt");

#[derive(Clone, Debug)]
pub struct Instructions {
    pub manifest: InstructionManifest,
    pub initialization: String,
    pub guidance: String,
    pub prompts: Value,
}
pub fn prompt_interfaces() -> Value {
    serde_json::from_str::<Value>(CONTRACT).unwrap()["prompts/list"]["prompts"].clone()
}
pub fn default_files() -> BTreeMap<String, Vec<u8>> {
    let contract: Value = serde_json::from_str(CONTRACT).unwrap();
    let mut prompts: Value = serde_json::from_str(PROMPTS).unwrap();
    let strip = |s: &str| {
        format!(
            "{}{GUIDANCE_MARKER}{GUIDANCE_SLOT}",
            s.split_once(GUIDANCE_MARKER).unwrap().0
        )
    };
    prompts["compose_artwork"]["messages"][0]["content"]["text"] = Value::String(strip(
        prompts["compose_artwork"]["messages"][0]["content"]["text"]
            .as_str()
            .unwrap(),
    ));
    BTreeMap::from([
        (
            "initialize.txt".into(),
            strip(contract["initialize"]["instructions"].as_str().unwrap()).into_bytes(),
        ),
        (
            "authoring-guidance.txt".into(),
            GUIDANCE.as_bytes().to_vec(),
        ),
        (
            "prompts.json".into(),
            serde_json::to_vec_pretty(&prompts).unwrap(),
        ),
        (
            "prompt-interfaces.json".into(),
            serde_json::to_vec(&prompt_interfaces()).unwrap(),
        ),
        (
            "skills/inkscape-mcp/SKILL.md".into(),
            include_bytes!("../../../skills/inkscape-mcp/SKILL.md").to_vec(),
        ),
        (
            "skills/inkscape-mcp/agents/openai.yaml".into(),
            include_bytes!("../../../skills/inkscape-mcp/agents/openai.yaml").to_vec(),
        ),
    ])
}
pub fn manifest(version: &str, files: &BTreeMap<String, Vec<u8>>) -> InstructionManifest {
    let files = files
        .iter()
        .map(|(k, v)| (k.clone(), FileIdentity::of(v)))
        .collect();
    InstructionManifest {
        format: FORMAT,
        version: version.into(),
        content_id: content_id(&files),
        text_interface: TEXT_INTERFACE,
        launcher_minimum: 1,
        files,
    }
}
// Refuse every symlink ancestor, foreign owned file and group/world writable bundle path.
pub fn guarded(path: &Path) -> Result<()> {
    guarded_with_source(path, None)
}
// Only immutable inputs belonging to this running Manager may cross the macOS
// root:admin Applications ancestor. Managed destinations always use guarded().
const SYSTEM_MANAGER_APP: &str = "/Applications/Inkscape MCP Manager.app";
fn manager_input(path: &Path, executable: &Path) -> bool {
    let app = Path::new(SYSTEM_MANAGER_APP).join("Contents");
    cfg!(target_os = "macos")
        && executable == app.join("Helpers/inkscape-mcp-launcher")
        && (path == executable
            || path == app.join("Info.plist")
            || path.starts_with(app.join("Resources")))
}
pub(super) fn bundled_manager_input(path: &Path) -> bool {
    std::env::current_exe().is_ok_and(|exe| manager_input(path, &exe))
}
pub(super) fn guarded_read(path: &Path) -> Result<()> {
    guarded_with_source(
        path,
        bundled_manager_input(path).then(|| Path::new(SYSTEM_MANAGER_APP)),
    )
}
fn guarded_with_source(path: &Path, manager_app: Option<&Path>) -> Result<()> {
    if !path.is_absolute() {
        return Err("managed path must be absolute".into());
    }
    let mut current = std::path::PathBuf::new();
    for part in path.components() {
        if matches!(part, Component::ParentDir | Component::CurDir) {
            return Err("unsafe managed path".into());
        }
        current.push(part);
        let metadata =
            fs::symlink_metadata(&current).map_err(|e| format!("{}: {e}", current.display()))?;
        if metadata.file_type().is_symlink() {
            return Err("managed path contains a symlink".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            // Managed state stays user-owned; the running app's read-only inputs
            // may also be root-owned after an administrator installed the app.
            if metadata.uid() != unsafe { libc::geteuid() } && metadata.uid() != 0 {
                return Err("managed path has foreign ownership".into());
            }
            let in_bundle = manager_app.is_some_and(|app| current.starts_with(app));
            let allow_writable_ancestor =
                if manager_app.is_some() && current == Path::new("/Applications") {
                    metadata.uid() == 0
                        && metadata.gid() == 80
                        && metadata.is_dir()
                        && metadata.mode() & 0o002 == 0
                } else {
                    // A sticky temporary ancestor is normal outside the app. Inside
                    // the signed bundle, every descendant must be immutable to others.
                    !in_bundle && metadata.mode() & 0o1000 != 0
                };
            if metadata.mode() & 0o022 != 0 && !allow_writable_ancestor {
                return Err(format!(
                    "{}: managed path is writable by other users",
                    current.display()
                ));
            }
            if current == path && metadata.uid() != unsafe { libc::geteuid() } && !in_bundle {
                return Err("managed file must be user-owned".into());
            }
        }
    }
    Ok(())
}
pub fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    guarded_read(path)?;
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("managed input must be a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("managed input exceeds byte limit".into());
    }
    Ok(bytes)
}
fn text(files: &BTreeMap<String, Vec<u8>>, key: &str) -> Result<String> {
    String::from_utf8(
        files
            .get(key)
            .ok_or_else(|| format!("missing instruction file: {key}"))?
            .clone(),
    )
    .map_err(|_| "instructions must be UTF-8".into())
}
impl Instructions {
    pub fn from_files(
        manifest: InstructionManifest,
        files: BTreeMap<String, Vec<u8>>,
    ) -> Result<Self> {
        manifest.validate()?;
        if files.keys().collect::<Vec<_>>() != manifest.files.keys().collect::<Vec<_>>() {
            return Err("instruction file set mismatch".into());
        }
        for (name, bytes) in &files {
            if manifest.files[name] != FileIdentity::of(bytes) {
                return Err(format!("instruction checksum mismatch: {name}"));
            }
        }
        let interfaces: Value = serde_json::from_str(&text(&files, "prompt-interfaces.json")?)
            .map_err(|e| e.to_string())?;
        if interfaces != prompt_interfaces() {
            return Err("compiled prompt interfaces differ".into());
        }
        let initialization = text(&files, "initialize.txt")?;
        let guidance = text(&files, "authoring-guidance.txt")?;
        if initialization.matches(GUIDANCE_SLOT).count() != 1
            || guidance.contains("__")
            || guidance.trim().is_empty()
        {
            return Err("invalid authoring guidance slot".into());
        }
        slots(&initialization, &[GUIDANCE_SLOT])?;
        let prompts: Value =
            serde_json::from_str(&text(&files, "prompts.json")?).map_err(|e| e.to_string())?;
        let defaults: Value = serde_json::from_str(PROMPTS).unwrap();
        if prompts
            .as_object()
            .ok_or("prompts must be an object")?
            .keys()
            .collect::<BTreeSet<_>>()
            != defaults
                .as_object()
                .unwrap()
                .keys()
                .collect::<BTreeSet<_>>()
        {
            return Err("prompt names differ from compiled interfaces".into());
        }
        for (name, default) in defaults.as_object().unwrap() {
            let prompt = &prompts[name];
            let mut shape = prompt.clone();
            let messages = shape["messages"]
                .as_array_mut()
                .ok_or("invalid prompt messages")?;
            if messages.len() != default["messages"].as_array().unwrap().len() {
                return Err("prompt message count differs".into());
            }
            for (index, message) in messages.iter_mut().enumerate() {
                let value = message["content"]["text"]
                    .as_str()
                    .ok_or("prompt must contain text")?;
                let goal = default["messages"][index]["content"]["text"]
                    .as_str()
                    .unwrap()
                    .contains(GOAL_SLOT);
                let authoring = name == "compose_artwork" && index == 0;
                if value.matches(GOAL_SLOT).count() != usize::from(goal)
                    || value.matches(GUIDANCE_SLOT).count() != usize::from(authoring)
                {
                    return Err("required prompt placeholders differ".into());
                }
                slots(value, &[GOAL_SLOT, GUIDANCE_SLOT])?;
                message["content"]["text"] = default["messages"][index]["content"]["text"].clone();
            }
            shape["description"] = default["description"].clone();
            if shape != *default || !prompt["description"].is_string() {
                return Err("prompt message interface differs".into());
            }
        }
        for name in [
            "skills/inkscape-mcp/SKILL.md",
            "skills/inkscape-mcp/agents/openai.yaml",
        ] {
            text(&files, name)?;
        }
        // Only these six text files are supported, never schemas or executables.
        if files.keys().collect::<Vec<_>>() != default_files().keys().collect::<Vec<_>>() {
            return Err("unsupported instruction files".into());
        }
        Ok(Self {
            manifest,
            initialization,
            guidance,
            prompts,
        })
    }
    pub fn load(root: &Path) -> Result<Self> {
        guarded_read(root)?;
        let manifest: InstructionManifest =
            serde_json::from_slice(&read(&root.join("manifest.json"), 64 * 1024)?)
                .map_err(|e| e.to_string())?;
        manifest.validate()?;
        let files = manifest
            .files
            .iter()
            .map(|(name, id)| Ok((name.clone(), read(&root.join(name), id.bytes)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let inventory = super::storage::inventory(root, 2 * 1024 * 1024 + 64 * 1024)?;
        let expected = manifest
            .files
            .keys()
            .map(String::as_str)
            .chain(std::iter::once("manifest.json"))
            .collect::<BTreeSet<_>>();
        if inventory
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != expected
        {
            return Err("instruction directory has unexpected files".into());
        }
        Self::from_files(manifest, files)
    }
    pub fn expand(&self, template: &str) -> String {
        template.replace(GUIDANCE_SLOT, self.guidance.trim_end())
    }
}
fn slots(text: &str, approved: &[&str]) -> Result<()> {
    let mut remainder = text.to_string();
    for slot in approved {
        remainder = remainder.replace(slot, "");
    }
    if remainder.contains("__") {
        return Err("unknown instruction placeholder".into());
    }
    Ok(())
}
static ACTIVE: OnceLock<Result<Instructions>> = OnceLock::new();
pub fn active() -> Result<&'static Instructions> {
    ACTIVE
        .get_or_init(|| {
            if let Some(path) = std::env::var_os("INKSCAPE_MCP_INSTRUCTION_BUNDLE") {
                Instructions::load(Path::new(&path))
            } else {
                let files = default_files();
                Instructions::from_files(manifest("embedded", &files), files)
            }
        })
        .as_ref()
        .map_err(Clone::clone)
}

#[cfg(test)]
mod source_guard_tests {
    use super::*;
    #[test]
    fn source_exception_is_confined_to_the_running_system_manager_inputs() {
        let exe = Path::new(
            "/Applications/Inkscape MCP Manager.app/Contents/Helpers/inkscape-mcp-launcher",
        );
        for input in [
            "Helpers/inkscape-mcp-launcher",
            "Info.plist",
            "Resources",
            "Resources/offline-runtime.tar.gz",
            "Resources/instructions/manifest.json",
        ] {
            assert_eq!(
                manager_input(&exe.parent().unwrap().parent().unwrap().join(input), exe),
                cfg!(target_os = "macos")
            );
        }
        for input in [
            "/Applications",
            "/Applications/Other.app/Contents/Resources",
            "/Applications/Inkscape MCP Manager.app/Contents/MacOS/inkscape-mcp-manager",
            "/Users/example/Library/Application Support/inkscape-mcp/settings.json",
        ] {
            assert!(!manager_input(Path::new(input), exe));
        }
        assert!(!manager_input(
            Path::new("/Applications/Inkscape MCP Manager.app/Contents/Resources"),
            Path::new("/Users/example/bin/inkscape-mcp-launcher")
        ));
    }
    #[test]
    fn source_reads_still_refuse_writable_descendants_and_symlinks() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let scratch = tempfile::tempdir().unwrap();
        let root = scratch.path().canonicalize().unwrap();
        let file = root.join("input");
        std::fs::write(&file, b"immutable").unwrap();
        guarded_with_source(&file, Some(&root)).unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o664)).unwrap();
        assert!(
            guarded_with_source(&file, Some(&root))
                .unwrap_err()
                .contains("writable by other users")
        );
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        let link = root.join("linked");
        symlink(&file, &link).unwrap();
        assert!(
            guarded_with_source(&link, Some(&root))
                .unwrap_err()
                .contains("symlink")
        );
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o775)).unwrap();
        assert!(guarded_with_source(&file, Some(&root)).is_err());
    }
}
