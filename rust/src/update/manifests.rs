use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub type Result<T> = std::result::Result<T, String>;
pub const FORMAT: u32 = 1;
pub const TEXT_INTERFACE: u32 = 1;
pub const HELPER_PROTOCOL: u32 = 5;
pub const LAUNCHER_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FileIdentity {
    pub bytes: u64,
    pub sha256: String,
}
impl FileIdentity {
    pub fn of(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.len() as u64,
            sha256: format!("{:x}", Sha256::digest(bytes)),
        }
    }
    pub fn validate(&self, limit: u64) -> Result<()> {
        if self.bytes == 0 || self.bytes > limit || !hash(&self.sha256) {
            return Err("invalid asset length or SHA-256".into());
        }
        Ok(())
    }
}
pub fn hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
pub fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        && s != "."
        && s != ".."
}
pub fn relative(s: &str) -> bool {
    !s.is_empty() && s.len() <= 240 && s.split('/').all(identifier) && !s.contains('\\')
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub name: String,
    #[serde(flatten)]
    pub identity: FileIdentity,
}
impl Asset {
    pub fn validate(&self, limit: u64) -> Result<()> {
        if !identifier(&self.name) {
            return Err("invalid release asset name".into());
        }
        self.identity.validate(limit)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeManifest {
    pub format: u32,
    pub distribution_tag: String,
    pub build_id: String,
    pub source_revision: String,
    pub os: String,
    pub architecture: String,
    pub minimum_os_major: u32,
    pub text_interface: u32,
    pub helper_protocol: u32,
    pub launcher_minimum: u32,
    pub asset: Asset,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstructionManifest {
    pub format: u32,
    pub version: String,
    pub content_id: String,
    pub text_interface: u32,
    pub launcher_minimum: u32,
    pub files: BTreeMap<String, FileIdentity>,
}
impl InstructionManifest {
    pub fn validate(&self) -> Result<()> {
        if self.format != FORMAT
            || self.text_interface != TEXT_INTERFACE
            || self.launcher_minimum > LAUNCHER_VERSION
        {
            return Err(
                "unsupported instruction format/interface; upgrade the launcher manually".into(),
            );
        }
        if !identifier(&self.version)
            || !hash(&self.content_id)
            || self.files.len() > 64
            || self.files.is_empty()
        {
            return Err("invalid instruction identity".into());
        }
        let mut total = 0;
        for (name, file) in &self.files {
            if !relative(name) {
                return Err("unsafe instruction path".into());
            }
            file.validate(256 * 1024)?;
            total += file.bytes;
        }
        if total > 2 * 1024 * 1024 || self.content_id != content_id(&self.files) {
            return Err("instruction content identity/size mismatch".into());
        }
        Ok(())
    }
}
pub fn content_id(files: &BTreeMap<String, FileIdentity>) -> String {
    // Fixed struct field order, sorted paths, UTF-8 JSON, no whitespace.
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(files).expect("file identities serialize"))
    )
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    pub format: u32,
    pub tag: String,
    pub prerelease: bool,
    pub runtime: RuntimeManifest,
    pub instructions: InstructionManifest,
    pub instruction_asset: Asset,
    pub launcher_asset: Asset,
}
impl ReleaseManifest {
    pub fn compatible(&self, os: &str, architecture: &str, os_major: u32) -> Result<()> {
        self.instructions.validate()?;
        let r = &self.runtime;
        if self.format != FORMAT
            || r.format != FORMAT
            || !identifier(&self.tag)
            || !identifier(&r.distribution_tag)
            || !identifier(&r.build_id)
            || !identifier(&r.source_revision)
        {
            return Err("unsupported release format or identity".into());
        }
        if r.launcher_minimum > LAUNCHER_VERSION {
            return Err("launcher upgrade required; install a newer launcher manually".into());
        }
        if r.os != os
            || r.architecture != architecture
            || os_major < r.minimum_os_major
            || os != "macos"
            || architecture != "aarch64"
        {
            return Err("unsupported update target; requires Apple Silicon macOS 15+".into());
        }
        if r.minimum_os_major < 15
            || r.text_interface != self.instructions.text_interface
            || r.helper_protocol != HELPER_PROTOCOL
        {
            return Err("incompatible runtime, instructions or native-helper protocol".into());
        }
        r.asset.validate(512 * 1024 * 1024)?;
        self.instruction_asset.validate(4 * 1024 * 1024)?;
        self.launcher_asset.validate(128 * 1024 * 1024)?;
        Ok(())
    }
}
