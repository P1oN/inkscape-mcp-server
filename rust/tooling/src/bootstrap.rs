use crate::{archive, common::*};
use serde_json::json;
use std::{
    fs,
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::Path,
    process::{Command, Stdio},
};
fn download(url: &str, destination: &Path, checksum: &str, token: &str) -> Result<()> {
    ensure(
        url.starts_with("https://"),
        "only HTTPS build inputs allowed",
    )?;
    ensure(!destination.exists(), "download output must be new")?;
    // Curl removes Authorization on redirects to other hosts by default. Keep bearer
    // tokens off argv/logs and reject curl-config metacharacters in all inputs.
    ensure(
        [url, token, &destination.to_string_lossy()]
            .iter()
            .all(|s| !s.contains(['\n', '\r', '"', '\\'])),
        "invalid download config",
    )?;
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(destination)?;
    let mut child = Command::new(tool("curl")?)
        .args([
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--tlsv1.2",
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--connect-timeout",
            "30",
            "--max-time",
            "120",
            "--max-filesize",
            "134217728",
            "--config",
            "-",
        ])
        .stdout(Stdio::from(file))
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let config = format!("url = \"{url}\"\nheader = \"Authorization: Bearer {token}\"\n");
    child
        .stdin
        .take()
        .ok_or("curl stdin unavailable")?
        .write_all(config.as_bytes())?;
    let result = child.wait_with_output()?;
    ensure(
        result.status.success(),
        "pinned native input download failed",
    )?;
    ensure(
        fs::metadata(destination)?.len() <= 128 * 1024 * 1024 && hash(destination)? == checksum,
        "native archive checksum/cap differs",
    )
}
pub fn run(args: &Args) -> Result<()> {
    args.check(&["--lock", "--output"])?;
    prepare(&args.required("--lock")?, &args.required("--output")?)
}
pub fn prepare(lock: &Path, output_dir: &Path) -> Result<()> {
    let metadata = crate::common::json(lock)?;
    ensure(
        metadata["target"] == "macos-arm64"
            && metadata["bottles"].as_array().is_some_and(|a| a.len() == 6),
        "unexpected native input lock",
    )?;
    let mut names = std::collections::HashSet::new();
    for row in metadata["bottles"].as_array().unwrap() {
        let name = row["name"].as_str().ok_or("invalid formula name")?;
        let checksum = row["sha256"].as_str().ok_or("invalid checksum")?;
        let version = row["version"].as_str().ok_or("invalid formula version")?;
        ensure(
            ["glib", "dbus", "gettext", "pcre2", "json-c", "libunistring"].contains(&name)
                && names.insert(name),
            "unexpected/duplicate formula",
        )?;
        ensure(
            checksum.len() == 64
                && checksum
                    .bytes()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "invalid checksum",
        )?;
        ensure(
            !version.is_empty()
                && version
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c)),
            "invalid formula version",
        )?;
        ensure(
            row["url"]
                == format!("https://ghcr.io/v2/homebrew/core/{name}/blobs/sha256:{checksum}"),
            "unexpected native input URL",
        )?;
    }
    if let Some(parent) = output_dir.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(output_dir)?;
    for row in metadata["bottles"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let url = format!(
            "https://ghcr.io/token?service=ghcr.io&scope=repository%3Ahomebrew%2Fcore%2F{name}%3Apull"
        );
        let response = output(Command::new(tool("curl")?).args([
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--tlsv1.2",
            "--fail",
            "--silent",
            "--show-error",
            "--max-time",
            "30",
            "--max-filesize",
            "1048576",
            &url,
        ]))?;
        let token: serde_json::Value = serde_json::from_str(&response)?;
        let token = token["token"]
            .as_str()
            .ok_or("native input token unavailable")?;
        println!("Preparing pinned native input: {name}");
        let file = output_dir.join(format!("{name}.tar.gz"));
        download(
            row["url"].as_str().unwrap(),
            &file,
            row["sha256"].as_str().unwrap(),
            token,
        )?;
        archive::extract(
            &file,
            output_dir,
            &Path::new(name).join(row["version"].as_str().unwrap()),
        )?;
        fs::remove_file(file)?;
    }
    write_json(&output_dir.join("inputs.json"), &json!(metadata))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_scheme_credentials_and_existing_destinations_refuse_before_download() {
        let root = tempfile::tempdir().unwrap();
        let output = root.path().join("never");
        assert!(download("file:///secret", &output, &"0".repeat(64), "").is_err());
        assert!(!output.exists());
        assert!(
            download(
                "https://ghcr.io/input",
                &output,
                &"0".repeat(64),
                "secret\ninjected"
            )
            .is_err()
        );
        assert!(!output.exists());
        fs::write(&output, "preserve").unwrap();
        assert!(download("https://ghcr.io/input", &output, &"0".repeat(64), "").is_err());
        assert_eq!(fs::read_to_string(&output).unwrap(), "preserve");
    }
}
