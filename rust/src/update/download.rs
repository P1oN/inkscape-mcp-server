//! Fixed repository discovery; HTTPS and explicit bounded redirects only.
use super::manifests::*;
use serde_json::Value;
use std::{io::Read, time::Duration};
const REPO: &str = "P1oN/inkscape-mcp-server";
pub const MANIFEST_ASSET: &str = "inkscape-mcp-update.json";
pub trait Transport {
    fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>>;
}
pub struct Https(reqwest::blocking::Client);
impl Https {
    pub fn new() -> Result<Self> {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("inkscape-mcp-launcher/1")
            .build()
            .map(Self)
            .map_err(|e| e.to_string())
    }
}
pub fn allowed(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url).map_err(|e| e.to_string())?;
    let host = parsed.host_str().ok_or("download host missing")?;
    if parsed.scheme() != "https"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.port().is_some_and(|p| p != 443)
        || parsed.fragment().is_some()
        || !matches!(
            host,
            "api.github.com"
                | "github.com"
                | "release-assets.githubusercontent.com"
                | "objects.githubusercontent.com"
        )
    {
        return Err("download redirect is outside approved GitHub HTTPS hosts".into());
    }
    Ok(())
}
impl Transport for Https {
    fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>> {
        let mut url = url.to_string();
        for _ in 0..6 {
            allowed(&url)?;
            let response = self
                .0
                .get(&url)
                .send()
                .map_err(|_| "Unable to reach GitHub. Check your internet connection or proxy settings and retry.".to_string())?;
            if response.status().is_redirection() {
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .ok_or("redirect has no location")?
                    .to_str()
                    .map_err(|e| e.to_string())?;
                url = reqwest::Url::parse(&url)
                    .unwrap()
                    .join(location)
                    .map_err(|e| e.to_string())?
                    .to_string();
                continue;
            }
            if !response.status().is_success() {
                return Err(format!(
                    "GitHub returned {}; check connectivity/API rate limits and retry",
                    response.status()
                ));
            }
            if response.content_length().is_some_and(|n| n > limit) {
                return Err("download exceeds declared byte limit".into());
            }
            let total = response.content_length();
            let mut response = response.take(limit + 1);
            let mut bytes = Vec::new();
            let mut chunk = [0u8; 65536];
            let mut next_report = 1024 * 1024;
            loop {
                let count = response.read(&mut chunk).map_err(|e| e.to_string())?;
                if count == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..count]);
                if bytes.len() as u64 >= next_report {
                    super::progress::download(bytes.len() as u64, total);
                    next_report += 1024 * 1024;
                }
            }
            super::progress::download(bytes.len() as u64, total);
            if bytes.len() as u64 > limit {
                return Err("download exceeds byte limit".into());
            }
            return Ok(bytes);
        }
        Err("too many download redirects".into())
    }
}
pub fn asset_url(tag: &str, name: &str) -> Result<String> {
    if !identifier(tag) || !identifier(name) {
        return Err("unsafe release tag/asset".into());
    }
    Ok(format!(
        "https://github.com/{REPO}/releases/download/{tag}/{name}"
    ))
}
pub fn asset(transport: &impl Transport, tag: &str, asset: &Asset) -> Result<Vec<u8>> {
    let bytes = transport.get(&asset_url(tag, &asset.name)?, asset.identity.bytes)?;
    if FileIdentity::of(&bytes) != asset.identity {
        return Err(format!("download checksum/length mismatch: {}", asset.name));
    }
    Ok(bytes)
}
pub fn discover(
    transport: &impl Transport,
    channel: &str,
    version: Option<&str>,
) -> Result<ReleaseManifest> {
    if !matches!(channel, "stable" | "prerelease") {
        return Err("channel must be stable or prerelease".into());
    }
    if version.is_some_and(|v| !identifier(v)) {
        return Err("invalid requested release version".into());
    }
    for page in 1..=10 {
        let bytes = transport.get(
            &format!("https://api.github.com/repos/{REPO}/releases?per_page=100&page={page}"),
            4 * 1024 * 1024,
        )?;
        let releases: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let releases = releases.as_array().ok_or("invalid GitHub release list")?;
        for release in releases {
            let tag = release["tag_name"].as_str().ok_or("release tag missing")?;
            let prerelease = release["prerelease"]
                .as_bool()
                .ok_or("release channel missing")?;
            if release["draft"] != false
                || (channel == "stable" && prerelease)
                || version.is_some_and(|v| v != tag)
            {
                continue;
            }
            // Old distributions lack update contracts: skip during discovery, explicit requests explain why.
            if !release["assets"]
                .as_array()
                .ok_or("release assets missing")?
                .iter()
                .any(|a| a["name"] == MANIFEST_ASSET)
            {
                if version == Some(tag) {
                    return Err("selected old release has no update manifest; use its documented manual installation".into());
                }
                continue;
            }
            let bytes = transport.get(&asset_url(tag, MANIFEST_ASSET)?, 128 * 1024)?;
            let manifest: ReleaseManifest = serde_json::from_slice(&bytes)
                .map_err(|e| format!("invalid update manifest: {e}"))?;
            if manifest.tag != tag || manifest.prerelease != prerelease {
                return Err("release manifest identity/channel mismatch".into());
            }
            return Ok(manifest);
        }
        if releases.len() < 100 {
            break;
        }
    }
    Err("no compatible update release found on the selected channel".into())
}
