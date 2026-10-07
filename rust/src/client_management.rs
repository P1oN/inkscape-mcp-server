//! Installation management only. Never invokes Inkscape or the helper runtime.
use serde_json::{Value, json};
use std::{
    env, fs,
    io::{BufRead, BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RESPONSE: usize = 8 * 1024 * 1024;
const NAME: &str = "inkscape";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Client {
    Codex,
    Claude,
}
impl Client {
    fn parse(s: &str) -> Result<Self> {
        match s {
            "codex" => Ok(Self::Codex),
            "claude" => Ok(Self::Claude),
            _ => Err("Client must be codex or claude".into()),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }
}
struct Profiles {
    home: PathBuf,
    codex: PathBuf,
}
impl Profiles {
    fn environment() -> Result<Self> {
        let home = PathBuf::from(env::var_os("HOME").ok_or("HOME is missing")?);
        let codex = env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".codex"));
        Ok(Self { home, codex })
    }
    fn config(&self, client: Client) -> PathBuf {
        match client {
            Client::Codex => self.codex.join("config.toml"),
            Client::Claude => self.home.join(".claude.json"),
        }
    }
    fn skill(&self, client: Client) -> PathBuf {
        match client {
            Client::Codex => self.codex.clone(),
            Client::Claude => self.home.join(".claude"),
        }
        .join("skills/inkscape-mcp")
    }
}
fn linked(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}
fn regular_optional(path: &Path) -> Result<bool> {
    if linked(path) || path.parent().is_some_and(linked) {
        return Err("Configuration must not be symlinked".into());
    }
    match fs::metadata(path) {
        Ok(m) if m.is_file() => Ok(true),
        Ok(_) => Err("Configuration must be a regular file".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}
fn entry(
    profiles: &Profiles,
    client: Client,
    launcher: &Path,
    require_match: bool,
) -> Result<Option<Value>> {
    let path = profiles.config(client);
    if !regular_optional(&path)? {
        return Ok(None);
    }
    if fs::metadata(&path)?.len() > 4 * 1024 * 1024 {
        return Err("Client configuration exceeds size limit".into());
    }
    let text = fs::read_to_string(path)?;
    let (data, key): (Value, _) = match client {
        Client::Codex => (
            serde_json::to_value(toml::from_str::<toml::Value>(&text)?)?,
            "mcp_servers",
        ),
        Client::Claude => (serde_json::from_str(&text)?, "mcpServers"),
    };
    if !data.is_object() {
        return Err("Invalid client configuration".into());
    }
    let entries = data.get(key).unwrap_or(&Value::Null);
    if !entries.is_null() && !entries.is_object() {
        return Err("Invalid client server table".into());
    }
    let result = entries.get(NAME).cloned();
    if let Some(ref item) = result {
        if !item.is_object() {
            return Err("Invalid inkscape entry".into());
        }
        if require_match
            && (item.get("command").and_then(Value::as_str) != launcher.to_str()
                || item.get("args").is_some_and(|v| v != &json!([]))
                || item.get("url").is_some_and(|v| !v.is_null() && v != ""))
        {
            return Err(
                "Existing inkscape entry differs; preserved. Remove or rename it first.".into(),
            );
        }
    }
    Ok(result)
}
// Setup owns the inkscape transport binding; all other client settings stay intact.
fn upgraded_config(text: &str, client: Client, launcher: &Path) -> Result<String> {
    let launcher = launcher.to_str().ok_or("Invalid launcher path")?;
    match client {
        Client::Codex => {
            let mut document = text.parse::<toml_edit::DocumentMut>()?;
            if !document.contains_key("mcp_servers") {
                document["mcp_servers"] = toml_edit::Item::Table(toml_edit::Table::new());
            }
            let servers = document["mcp_servers"]
                .as_table_like_mut()
                .ok_or("Invalid client server table")?;
            if !servers.contains_key(NAME) {
                servers.insert(NAME, toml_edit::Item::Table(toml_edit::Table::new()));
            }
            let server = servers
                .get_mut(NAME)
                .unwrap()
                .as_table_like_mut()
                .ok_or("Invalid inkscape entry")?;
            for key in [
                "url",
                "http_headers",
                "env_http_headers",
                "bearer_token_env_var",
                "type",
            ] {
                server.remove(key);
            }
            server.insert("command", toml_edit::value(launcher));
            server.insert("args", toml_edit::value(toml_edit::Array::new()));
            if !server.contains_key("env") {
                server.insert("env", toml_edit::value(toml_edit::InlineTable::new()));
            }
            Ok(document.to_string())
        }
        Client::Claude => {
            let mut document: Value = if text.is_empty() {
                json!({})
            } else {
                serde_json::from_str(text)?
            };
            let root = document
                .as_object_mut()
                .ok_or("Invalid client configuration")?;
            let servers = root
                .entry("mcpServers")
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .ok_or("Invalid client server table")?;
            let server = servers
                .entry(NAME)
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .ok_or("Invalid inkscape entry")?;
            for key in ["url", "headers"] {
                server.remove(key);
            }
            server.insert("type".into(), json!("stdio"));
            server.insert("command".into(), json!(launcher));
            server.insert("args".into(), json!([]));
            server.entry("env").or_insert_with(|| json!({}));
            Ok(serde_json::to_string_pretty(&document)? + "\n")
        }
    }
}
fn upgrade_registration(profiles: &Profiles, client: Client, launcher: &Path) -> Result<()> {
    let path = profiles.config(client);
    let existing = regular_optional(&path)?;
    if existing && fs::metadata(&path)?.len() > 4 * 1024 * 1024 {
        return Err("Client configuration exceeds size limit".into());
    }
    let before = if existing {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let after = upgraded_config(&before, client, launcher)?;
    if before == after {
        return Ok(());
    }
    let parent = path.parent().ok_or("Missing client profile")?;
    fs::create_dir_all(parent)?;
    let mut staged = tempfile::NamedTempFile::new_in(parent)?;
    staged.write_all(after.as_bytes())?;
    staged.flush()?;
    if existing {
        staged
            .as_file()
            .set_permissions(fs::metadata(&path)?.permissions())?;
    }
    if regular_optional(&path)? != existing || (existing && fs::read_to_string(&path)? != before) {
        return Err("Client configuration changed during setup; retry".into());
    }
    if existing {
        let mut backup = tempfile::Builder::new()
            .prefix("inkscape-config-backup.")
            .tempfile_in(parent)?;
        backup.write_all(before.as_bytes())?;
        backup.flush()?;
        let (_, saved) = backup.keep()?;
        eprintln!("Previous client configuration saved at {}", saved.display());
    }
    staged.persist(&path).map_err(|e| e.error)?;
    Ok(())
}
fn records(repo: &Path) -> Result<(PathBuf, Vec<String>)> {
    let local = repo.join(".inkscape-mcp-local");
    let path = local.join("clients.json");
    if linked(&local) || !local.is_dir() {
        return Err("Invalid installation/client record".into());
    }
    let clients: Vec<String> = if regular_optional(&path)? {
        serde_json::from_str(&fs::read_to_string(&path)?)?
    } else {
        vec![]
    };
    if clients.iter().any(|c| Client::parse(c).is_err()) {
        return Err("Invalid recorded clients".into());
    }
    Ok((path, clients))
}
fn save_clients(path: &Path, clients: &[String]) -> Result<()> {
    let mut file = tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing record parent")?)?;
    serde_json::to_writer(&mut file, clients)?;
    file.flush()?;
    file.persist(path).map_err(|e| e.error)?;
    Ok(())
}
fn wait_until(child: &mut Child, duration: Duration) -> Result<Option<std::process::ExitStatus>> {
    let deadline = Instant::now() + duration;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(10));
    }
}
// Drop always reaps the explicitly spawned process, including parse/timeout errors.
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        self.0.stdin.take();
        if matches!(wait_until(&mut self.0, Duration::from_secs(5)), Ok(Some(_))) {
            return;
        }
        #[cfg(unix)]
        unsafe {
            libc::kill(self.0.id() as libc::pid_t, libc::SIGTERM);
        }
        if matches!(wait_until(&mut self.0, Duration::from_secs(5)), Ok(Some(_))) {
            return;
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn run_cli(client: Client, args: &[&str]) -> Result<()> {
    // Anonymous files prevent noisy client diagnostics from blocking pipes or using memory.
    let output = tempfile::tempfile()?;
    let errors = tempfile::tempfile()?;
    let mut process = OwnedChild(
        Command::new(client.name())
            .args(args)
            .stdin(Stdio::null())
            .stdout(output)
            .stderr(errors)
            .spawn()
            .map_err(|e| {
                format!(
                    "Client CLI missing or unavailable: {e}; install it or use the config action"
                )
            })?,
    );
    match wait_until(&mut process.0, REQUEST_TIMEOUT)? {
        Some(status) if status.success() => Ok(()),
        Some(_) => Err(
            "Client command failed; installation retained; inspect client CLI diagnostics".into(),
        ),
        None => Err("Client command timed out; installation retained".into()),
    }
}
struct Wire {
    process: OwnedChild,
    messages: Receiver<Result<Value>>,
    sequence: u64,
}
impl Wire {
    fn send(
        &mut self,
        method: &str,
        params: Value,
        notification: bool,
        timeout: Duration,
    ) -> Result<Value> {
        self.sequence += 1;
        let mut request = json!({"jsonrpc":"2.0", "method":method,"params":params});
        if !notification {
            request["id"] = json!(self.sequence);
        }
        let stdin = self.process.0.stdin.as_mut().ok_or("MCP stdin closed")?;
        serde_json::to_writer(&mut *stdin, &request)?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;
        if notification {
            return Ok(Value::Null);
        }
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or_else(|| format!("MCP request timed out: {method}"))?;
            let reply = self
                .messages
                .recv_timeout(remaining)
                .map_err(|_| format!("MCP request timed out or STDIO closed: {method}"))??;
            if Instant::now() >= deadline {
                return Err(format!("MCP request timed out: {method}").into());
            }
            if reply.get("id") == Some(&json!(self.sequence)) {
                if reply.get("error").is_some() {
                    return Err(format!("MCP request failed: {method}").into());
                }
                return reply
                    .get("result")
                    .cloned()
                    .ok_or_else(|| "Missing MCP result".into());
            }
        }
    }
}
pub fn probe(command: &mut Command, timeout: Duration) -> Result<Value> {
    let errors = tempfile::tempfile()?;
    let mut process = OwnedChild(
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(errors)
            .spawn()?,
    );
    let stdout = process.0.stdout.take().ok_or("MCP stdout unavailable")?;
    let (tx, messages) = mpsc::sync_channel(16);
    let reader = thread::spawn(move || {
        let mut input = BufReader::new(stdout);
        loop {
            let mut bytes = vec![];
            let result: Result<Value> = (|| {
                let size = input
                    .by_ref()
                    .take((MAX_RESPONSE + 1) as u64)
                    .read_until(b'\n', &mut bytes)?;
                if size == 0 {
                    return Err("MCP server closed STDIO".into());
                }
                if size > MAX_RESPONSE {
                    return Err("Oversized MCP response".into());
                }
                Ok(serde_json::from_slice(&bytes)?)
            })();
            let failed = result.is_err();
            if tx.send(result).is_err() || failed {
                break;
            }
        }
    });
    let mut wire = Wire {
        process,
        messages,
        sequence: 0,
    };
    let result = (|| {
        let info = wire.send("initialize", json!({"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"inkscape-install-check","version":"1"}}), false, timeout)?;
        if info
            .get("protocolVersion")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
            || !info["capabilities"]["tools"].is_object()
        {
            return Err("Invalid MCP handshake or missing tool capability".into());
        }
        if let Some((_, Some(bundle))) = command
            .get_envs()
            .find(|(name, _)| *name == "INKSCAPE_MCP_INSTRUCTION_BUNDLE")
        {
            let instructions = crate::update::instructions::Instructions::load(Path::new(bundle))?;
            if info["instructions"].as_str()
                != Some(instructions.expand(&instructions.initialization).as_str())
            {
                return Err("Candidate did not load the selected instruction bundle".into());
            }
        }
        wire.send("notifications/initialized", json!({}), true, timeout)?;

        let discovery = wire.send("tools/list", json!({}), false, timeout)?;
        let tools = discovery["tools"]
            .as_array()
            .ok_or("Invalid tool discovery")?;
        for name in ["get_workspace_info", "create_document", "render_preview"] {
            if !tools.iter().any(|t| t["name"] == name) {
                return Err("Required Inkscape tools unavailable".into());
            }
        }
        let workspace = wire.send(
            "tools/call",
            json!({"name":"get_workspace_info","arguments":{}}),
            false,
            timeout,
        )?;
        if !workspace.is_object()
            || workspace
                .get("isError")
                .is_some_and(|v| !v.is_null() && !v.is_boolean())
        {
            return Err("Invalid workspace response".into());
        }
        if workspace["isError"] == true {
            return Err("First workspace request failed".into());
        }
        Ok(
            json!({"handshake":true,"tools":tools.len(),"first_request":"get_workspace_info","server":info.get("serverInfo")}),
        )
    })();
    // Drop receiver first so a full notification queue cannot trap the reader.
    let Wire {
        process, messages, ..
    } = wire;
    drop(messages);
    drop(process);
    // A descendant could retain stdout after the owned server exits. Do not wait
    // indefinitely or terminate unrelated processes merely to close that pipe.
    let deadline = Instant::now() + Duration::from_secs(1);
    while !reader.is_finished() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    if reader.is_finished() {
        reader.join().map_err(|_| "MCP reader panicked")?;
    }

    result
}
fn archive_with(
    repo: &Path,
    skills: &Path,
    mut rename: impl FnMut(&Path, &Path) -> std::io::Result<()>,
) -> Result<PathBuf> {
    let local = repo.join(".inkscape-mcp-local");
    if linked(&local) || !local.is_dir() {
        return Err("Invalid installation directory".into());
    }
    let owner = skills.join(".inkscape-mcp-owner");
    let owned = !linked(skills)
        && !linked(&owner)
        && owner.is_file()
        && fs::read_to_string(&owner)?.trim() == repo.to_str().ok_or("Invalid repository path")?;
    let staged = local.join("removed-skill");
    let backup = repo.join(format!(
        ".inkscape-mcp-backup-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    if backup.try_exists()? || linked(&backup) {
        return Err("Archive destination already exists".into());
    }
    if owned {
        if staged.try_exists()? || linked(&staged) {
            return Err("Skill archive destination already exists; installation retained".into());
        }
        rename(skills, &staged)?;
    }
    if let Err(error) = rename(&local, &backup) {
        if owned {
            rename(&staged, skills).map_err(|rollback| {
                format!(
                    "Archive failed: {error}; skill restore failed: {rollback}; recover {} to {}",
                    staged.display(),
                    skills.display()
                )
            })?;
        }
        return Err(error.into());
    }
    Ok(backup)
}
fn manage(repo: &Path, profiles: &Profiles, client: Client, action: &str) -> Result<()> {
    let launcher = repo.join("run-mcp.sh");
    if action == "upgrade-existing" {
        if entry(profiles, client, &launcher, false)?.is_none() {
            return Ok(());
        }
        return manage(repo, profiles, client, "upgrade");
    }
    if action == "config" {
        match client {
            Client::Codex => println!(
                "[mcp_servers.inkscape]\ncommand = {}\nargs = []",
                toml::Value::String(launcher.to_str().ok_or("Invalid launcher path")?.into())
            ),
            Client::Claude => println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"mcpServers":{"inkscape":{"type":"stdio","command":launcher,"args":[]}}})
                )?
            ),
        }
        return Ok(());
    }
    if action == "check" {
        println!(
            "{}",
            serde_json::to_string_pretty(&probe(&mut Command::new(&launcher), REQUEST_TIMEOUT)?)?
        );
        return Ok(());
    }
    let current = entry(profiles, client, &launcher, action != "upgrade")?;
    let (record, mut clients) = records(repo)?;
    if action == "uninstall" {
        for other in &clients {
            let other = Client::parse(other)?;
            if other != client
                && entry(profiles, other, &launcher, false)?
                    .is_some_and(|e| e["command"].as_str() == launcher.to_str())
            {
                return Err("Another client uses this installation; disconnect it first".into());
            }
        }
    }
    // Preserve the requirement for an available client CLI even for an absent entry.
    let cli_available = env::split_paths(&env::var_os("PATH").unwrap_or_default()).any(|p| {
        let path = p.join(client.name());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        }
        #[cfg(not(unix))]
        {
            path.is_file()
        }
    });
    if !cli_available && action != "upgrade" {
        return Err("Client CLI missing; install it or use the config action".into());
    }
    if action == "connect" || action == "upgrade" {
        let mut report = probe(&mut Command::new(&launcher), REQUEST_TIMEOUT)?;
        if action == "upgrade" {
            upgrade_registration(profiles, client, &launcher)?;
        } else if current.is_none() {
            let launcher = launcher.to_str().ok_or("Invalid launcher path")?;
            let args = match client {
                Client::Codex => vec!["mcp", "add", NAME, "--", launcher],
                Client::Claude => vec![
                    "mcp",
                    "add",
                    "--transport",
                    "stdio",
                    "--scope",
                    "user",
                    NAME,
                    "--",
                    launcher,
                ],
            };
            run_cli(client, &args)?;
        }
        if entry(profiles, client, &launcher, true)?.is_none() {
            return Err("Client registration was not persisted".into());
        }
        if !clients.iter().any(|c| c == client.name()) {
            clients.push(client.name().into());
        }
        save_clients(&record, &clients)?;
        report["client"] = json!(client.name());
        println!(
            "{}\nRestart/reconnect your client to load the server.",
            serde_json::to_string_pretty(&report)?
        );
    } else {
        if current.is_some() {
            let mut args = vec!["mcp", "remove", NAME];
            if client == Client::Claude {
                args.extend(["--scope", "user"]);
            }
            run_cli(client, &args)?;
            if entry(profiles, client, &launcher, true)?.is_some() {
                return Err("Client removal failed; installation retained".into());
            }
        }
        if action == "disconnect" {
            clients.retain(|name| name != client.name());
            save_clients(&record, &clients)?;
        }
        if action == "uninstall" {
            let backup = archive_with(repo, &profiles.skill(client), |a, b| fs::rename(a, b))?;
            println!("Installation moved to {}", backup.display());
        }
        println!("Client disconnected. Existing client processes must be restarted by the user.");
    }
    Ok(())
}
fn main_result() -> Result<()> {
    let mut args = env::args().skip(1);
    let mut repo = None;
    let mut client = None;
    let mut action = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo" if repo.is_none() => {
                repo = Some(PathBuf::from(args.next().ok_or("Missing --repo value")?))
            }
            "--client" if client.is_none() => {
                client = Some(Client::parse(
                    &args.next().ok_or("Missing --client value")?,
                )?)
            }
            "config" | "check" | "connect" | "upgrade" | "upgrade-existing" | "disconnect"
            | "uninstall"
                if action.is_none() =>
            {
                action = Some(arg)
            }
            "--help" | "-h" => {
                println!(
                    "Usage: inkscape-mcp-client --repo DIRECTORY --client codex|claude config|check|connect|upgrade|disconnect|uninstall"
                );
                return Ok(());
            }
            _ => return Err(format!("Invalid argument: {arg}").into()),
        }
    }
    let repo = fs::canonicalize(repo.ok_or("--repo is required")?)?;
    if !repo.is_dir() {
        return Err("Repository must be a directory".into());
    }
    manage(
        &repo,
        &Profiles::environment()?,
        client.ok_or("--client is required")?,
        &action.ok_or("Action is required")?,
    )
}
pub fn cli() {
    if let Err(e) = main_result() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

/// Validate a permanent launcher before any one-time client binding change.
pub fn migrate_bindings(repo: &Path, launcher: &Path) -> Result<()> {
    probe(&mut Command::new(launcher), REQUEST_TIMEOUT)?;
    migrate_recorded_bindings(repo, &Profiles::environment()?, launcher)
}
fn migrate_recorded_bindings(repo: &Path, profiles: &Profiles, launcher: &Path) -> Result<()> {
    let (_, clients) = records(repo)?;
    for name in &clients {
        let client = Client::parse(name)?;
        if entry(profiles, client, launcher, false)?.is_some_and(|e| {
            e["command"].as_str() != repo.join("run-mcp.sh").to_str()
                && e["command"].as_str() != launcher.to_str()
        }) {
            return Err("recorded client now uses another installation; preserved".into());
        }
    }
    for client in clients {
        let client = Client::parse(&client)?;

        if entry(profiles, client, launcher, false)?.is_some() {
            upgrade_registration(profiles, client, launcher)?;
        }
    }
    Ok(())
}

/// Bounded candidate identity/doctor command. Uses the same owned-process deadline.
pub fn bounded_output(command: &mut Command, limit: u64) -> Result<Vec<u8>> {
    let output = tempfile::tempfile()?;
    let mut reader = output.try_clone()?;
    let mut process = OwnedChild(
        command
            .stdin(Stdio::null())
            .stdout(output)
            .stderr(Stdio::null())
            .spawn()?,
    );
    match wait_until(&mut process.0, REQUEST_TIMEOUT)? {
        Some(status) if status.success() => {}
        Some(_) => return Err("Candidate command failed".into()),
        None => return Err("Candidate command timed out".into()),
    }
    let mut bytes = Vec::new();
    reader.rewind()?;
    Read::by_ref(&mut reader)
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("Candidate output exceeds limit".into());
    }
    Ok(bytes)
}
/// Recovery works without the replaceable runtime or helper binaries.
pub fn disconnect_binding(name: &str, launcher: &Path) -> Result<()> {
    let client = Client::parse(name)?;
    let profiles = Profiles::environment()?;
    if entry(&profiles, client, launcher, true)?.is_none() {
        return Ok(());
    }
    let mut args = vec!["mcp", "remove", NAME];
    if client == Client::Claude {
        args.extend(["--scope", "user"]);
    }
    run_cli(client, &args)?;
    if entry(&profiles, client, launcher, true)?.is_some() {
        return Err("Client removal failed".into());
    }
    Ok(())
}
pub fn disconnect_owned_bindings(names: &[String], launcher: &Path) -> Result<()> {
    let profiles = Profiles::environment()?;
    for name in names {
        let client = Client::parse(name)?;
        if entry(&profiles, client, launcher, false)?
            .is_some_and(|e| e["command"].as_str() == launcher.to_str())
        {
            disconnect_binding(name, launcher)?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};
    #[test]
    fn upgrade_replaces_transport_preserving_configuration_and_comments() {
        let (_root, repo, profiles) = fixture();
        let launcher = repo.join("run-mcp.sh");
        let codex = "# user preferences\nmodel = 'custom'\n[mcp_servers.other]\nurl = 'https://other.invalid'\n[mcp_servers.inkscape]\nurl = 'https://old.invalid'\nstartup_timeout_sec = 90\n[mcp_servers.inkscape.env]\nCUSTOM = 'kept'\n";
        let claude = json!({"unrelated":{"keep":true},"mcpServers":{"other":{"url":"https://other.invalid"},"inkscape":{"url":"https://old.invalid","headers":{"old":"token"},"timeout":90,"env":{"CUSTOM":"kept"}}}}).to_string();
        for (client, text) in [(Client::Codex, codex.to_owned()), (Client::Claude, claude)] {
            fs::write(profiles.config(client), &text).unwrap();
            upgrade_registration(&profiles, client, &launcher).unwrap();
            let item = entry(&profiles, client, &launcher, true).unwrap().unwrap();
            assert_eq!(item["command"], launcher.to_str().unwrap());
            assert_eq!(item["args"], json!([]));
            assert_eq!(item["env"]["CUSTOM"], "kept");
            assert!(item.get("url").is_none());
            let updated = fs::read_to_string(profiles.config(client)).unwrap();
            assert!(updated.contains("other.invalid"));
            assert!(updated.contains("90"));
            if client == Client::Codex {
                assert!(updated.contains("# user preferences"));
                assert!(updated.contains("model = 'custom'"));
            }
            upgrade_registration(&profiles, client, &launcher).unwrap();
            assert_eq!(
                fs::read_to_string(profiles.config(client)).unwrap(),
                updated
            );
        }
        assert!(upgraded_config("mcp_servers = 7", Client::Codex, &launcher).is_err());
        assert!(upgraded_config("[]", Client::Claude, &launcher).is_err());
    }
    #[test]
    fn migration_skips_absent_recorded_clients_and_preserves_unrelated_settings() {
        for client in [Client::Codex, Client::Claude] {
            let (_root, repo, profiles) = fixture();
            let (record, _) = records(&repo).unwrap();
            save_clients(&record, &[client.name().into()]).unwrap();
            let launcher = repo.join("permanent-launcher");
            migrate_recorded_bindings(&repo, &profiles, &launcher).unwrap();
            assert!(!profiles.config(client).exists());
            let unrelated = if client == Client::Codex {
                "# preferences\nmodel = 'kept'\n[mcp_servers.other]\ncommand = 'keep'\n"
            } else {
                r#"{"preference":true,"mcpServers":{"other":{"command":"keep"}}}"#
            };
            fs::write(profiles.config(client), unrelated).unwrap();
            migrate_recorded_bindings(&repo, &profiles, &launcher).unwrap();
            assert_eq!(
                fs::read_to_string(profiles.config(client)).unwrap(),
                unrelated
            );
            let old = repo.join("run-mcp.sh");
            upgrade_registration(&profiles, client, &old).unwrap();
            migrate_recorded_bindings(&repo, &profiles, &launcher).unwrap();
            assert_eq!(
                entry(&profiles, client, &launcher, true).unwrap().unwrap()["command"],
                launcher.to_str().unwrap()
            );
        }
    }
    #[test]
    fn failed_upgrade_handshake_preserves_old_registration() {
        let (_root, repo, profiles) = fixture();
        let config = profiles.config(Client::Codex);
        let original = "[mcp_servers.inkscape]\ncommand = '/old/run-mcp.sh'\n";
        fs::write(&config, original).unwrap();
        fs::write(repo.join("run-mcp.sh"), "#!/bin/sh\nexit 1\n").unwrap();
        fs::set_permissions(repo.join("run-mcp.sh"), fs::Permissions::from_mode(0o700)).unwrap();
        assert!(manage(&repo, &profiles, Client::Codex, "upgrade").is_err());
        assert_eq!(fs::read_to_string(config).unwrap(), original);
        assert!(records(&repo).unwrap().1.is_empty());
    }
    fn fixture() -> (tempfile::TempDir, PathBuf, Profiles) {
        let root = tempfile::tempdir().unwrap();
        let repo = root.path().join("source space");
        fs::create_dir_all(repo.join(".inkscape-mcp-local")).unwrap();
        let profiles = Profiles {
            home: root.path().join("home"),
            codex: root.path().join("custom codex"),
        };
        fs::create_dir_all(&profiles.home).unwrap();
        fs::create_dir_all(&profiles.codex).unwrap();
        (root, repo, profiles)
    }
    fn server(root: &Path, body: &str) -> Command {
        let script = root.join("server.sh");
        fs::write(&script, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
        Command::new(script)
    }
    fn timely_body() -> &'static str {
        r#"while IFS= read -r line; do
printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/message"}'
case "$line" in
*'"method":"initialize"'*) printf '%s\n' '{"id":1,"result":{"protocolVersion":"2025-03-26","capabilities":{"tools":{}}}}';;
*'"method":"tools/list"'*) printf '%s\n' '{"id":3,"result":{"tools":[{"name":"get_workspace_info"},{"name":"create_document"},{"name":"render_preview"}]}}';;
*'"method":"tools/call"'*) printf '%s\n' '{"id":4,"result":{"isError":false}}';;
esac
done"#
    }
    #[test]
    fn timely_notifications_and_workspace_probe() {
        let root = tempfile::tempdir().unwrap();
        let report = probe(
            &mut server(root.path(), timely_body()),
            Duration::from_secs(2),
        )
        .unwrap();
        assert_eq!(report["handshake"], true);
        assert_eq!(report["tools"], 3);
        assert_eq!(report["first_request"], "get_workspace_info");
    }
    #[test]
    fn fixed_deadline_with_silence_and_continuous_notifications() {
        for flood in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let body = if flood {
                "read -r line\ni=0\nwhile [ $i -lt 200 ]; do printf '%s\\n' '{\"method\":\"notifications/message\"}'; i=$((i+1)); /bin/sleep 0.002; done"
            } else {
                "read -r line\n/bin/sleep 0.5"
            };
            let start = Instant::now();
            let error = probe(&mut server(root.path(), body), Duration::from_millis(100))
                .unwrap_err()
                .to_string();
            assert!(error.contains("MCP request timed out"), "{error}");
            assert!(start.elapsed() < Duration::from_secs(4));
            // A full queue must unblock on drop, and the owned child must be reaped.
        }
    }
    #[test]
    fn malformed_missing_tools_and_failed_workspace_reject() {
        for (body, expected) in [
            ("read -r line; printf '%s\\n' 'not json'", "expected"),
            (
                "read -r line; printf '%s\\n' '{\"id\":1,\"error\":{}}'",
                "MCP request failed",
            ),
            ("exit 0", "closed STDIO"),
        ] {
            let root = tempfile::tempdir().unwrap();
            assert!(
                probe(&mut server(root.path(), body), Duration::from_secs(1))
                    .unwrap_err()
                    .to_string()
                    .contains(expected)
            );
        }
        for (old, new, expected) in [
            (
                "render_preview",
                "different_tool",
                "Required Inkscape tools unavailable",
            ),
            (
                "\"isError\":false",
                "\"isError\":true",
                "First workspace request failed",
            ),
            (
                "\"isError\":false",
                "\"isError\":\"invalid\"",
                "Invalid workspace response",
            ),
        ] {
            let root = tempfile::tempdir().unwrap();
            assert!(
                probe(
                    &mut server(root.path(), &timely_body().replace(old, new)),
                    Duration::from_secs(1)
                )
                .unwrap_err()
                .to_string()
                .contains(expected)
            );
        }
    }
    #[test]
    fn oversized_reply_is_bounded() {
        let root = tempfile::tempdir().unwrap();
        let error = probe(
            &mut server(
                root.path(),
                "read -r line; /usr/bin/head -c 8388609 /dev/zero",
            ),
            Duration::from_secs(2),
        )
        .unwrap_err();
        assert!(error.to_string().contains("Oversized MCP response"));
    }
    #[test]
    fn each_request_keeps_its_own_fixed_deadline() {
        for method in ["initialize", "tools/list", "tools/call"] {
            let root = tempfile::tempdir().unwrap();
            let pattern = format!("*'\"method\":\"{method}\"'*) ");
            let replacement = format!("{pattern}/bin/sleep 1.2; ");
            let body = timely_body().replace(&pattern, &replacement);
            assert_ne!(body, timely_body());
            let error = probe(&mut server(root.path(), &body), Duration::from_millis(500))
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("timed out") && error.contains(method),
                "{method}: {error}; {body}"
            );
        }
    }
    #[test]
    fn codex_toml_and_claude_json_preserve_foreign_entries() {
        let (_root, repo, profiles) = fixture();
        let launcher = repo.join("run-mcp.sh");
        for client in [Client::Codex, Client::Claude] {
            let path = profiles.config(client);
            let text = if client == Client::Codex {
                format!(
                    "model = 'custom'\n[mcp_servers.other]\nurl = 'https://example.invalid'\n[mcp_servers.inkscape]\ncommand = {}\nargs = []\n",
                    toml::Value::String(launcher.to_str().unwrap().into())
                )
            } else {
                json!({"unrelated":{"keep":true},"mcpServers":{"inkscape":{"command":launcher,"args":[]}}}).to_string()
            };
            fs::write(&path, &text).unwrap();
            assert!(entry(&profiles, client, &launcher, true).unwrap().is_some());
            assert_eq!(fs::read_to_string(&path).unwrap(), text);
            for foreign in [
                text.replace(launcher.to_str().unwrap(), "/other/run-mcp.sh"),
                text.replace("args = []", "args = ['foreign']")
                    .replace("\"args\":[]", "\"args\":[\"foreign\"]"),
            ] {
                fs::write(&path, &foreign).unwrap();
                assert!(
                    entry(&profiles, client, &launcher, true)
                        .unwrap_err()
                        .to_string()
                        .contains("differs")
                );
                assert!(
                    entry(&profiles, client, &launcher, false)
                        .unwrap()
                        .is_some()
                );
                assert_eq!(fs::read_to_string(&path).unwrap(), foreign);
            }
        }
    }
    #[test]
    fn client_config_and_record_symlinks_refused() {
        let (root, repo, profiles) = fixture();
        let outside = root.path().join("outside");
        fs::write(&outside, "[]").unwrap();
        symlink(&outside, repo.join(".inkscape-mcp-local/clients.json")).unwrap();
        assert!(records(&repo).is_err());
        symlink(&outside, profiles.config(Client::Codex)).unwrap();
        assert!(entry(&profiles, Client::Codex, &repo.join("run-mcp.sh"), true).is_err());
        assert_eq!(fs::read_to_string(&outside).unwrap(), "[]");
    }
    #[test]
    fn recorded_clients_validate_and_atomic_save() {
        let (_root, repo, _profiles) = fixture();
        let (path, clients) = records(&repo).unwrap();
        assert!(clients.is_empty());
        save_clients(&path, &["codex".into(), "claude".into()]).unwrap();
        assert_eq!(records(&repo).unwrap().1, ["codex", "claude"]);
        for invalid in ["{}", "[\"foreign\"]", "[1]"] {
            fs::write(&path, invalid).unwrap();
            assert!(records(&repo).is_err());
        }
    }
    #[test]
    fn uninstall_blocks_only_another_client_on_this_launcher() {
        let (_root, repo, profiles) = fixture();
        let launcher = repo.join("run-mcp.sh");
        save_clients(
            &repo.join(".inkscape-mcp-local/clients.json"),
            &["codex".into(), "claude".into()],
        )
        .unwrap();
        fs::write(
            profiles.config(Client::Claude),
            json!({"mcpServers":{"inkscape":{"command":launcher,"args":["extra"]}}}).to_string(),
        )
        .unwrap();
        assert!(
            manage(&repo, &profiles, Client::Codex, "uninstall")
                .unwrap_err()
                .to_string()
                .contains("Another client")
        );
        // A switched client is valid for blocking inspection but remains foreign for its own actions.
        fs::write(
            profiles.config(Client::Claude),
            json!({"mcpServers":{"inkscape":{"command":"/other/run-mcp.sh"}}}).to_string(),
        )
        .unwrap();
        assert!(entry(&profiles, Client::Claude, &launcher, true).is_err());
        assert_ne!(
            entry(&profiles, Client::Claude, &launcher, false)
                .unwrap()
                .unwrap()["command"],
            json!(launcher)
        );
        assert!(repo.join(".inkscape-mcp-local").is_dir());
    }
    #[test]
    fn archive_failures_restore_customized_owned_skill_and_settings() {
        for failure in ["skill", "settings"] {
            let (_root, repo, profiles) = fixture();
            let local = repo.join(".inkscape-mcp-local");
            fs::write(local.join("setup.conf"), "saved settings").unwrap();
            let skill = profiles.skill(Client::Codex);
            fs::create_dir_all(&skill).unwrap();
            fs::write(skill.join("SKILL.md"), "customized").unwrap();
            fs::write(
                skill.join(".inkscape-mcp-owner"),
                format!("{}\n", repo.display()),
            )
            .unwrap();
            let error = archive_with(&repo, &skill, |source, dest| {
                if source == if failure == "skill" { &skill } else { &local } {
                    return Err(std::io::Error::from_raw_os_error(if failure == "skill" {
                        libc::EXDEV
                    } else {
                        libc::EACCES
                    }));
                }
                fs::rename(source, dest)
            })
            .unwrap_err();
            assert_eq!(
                error
                    .downcast_ref::<std::io::Error>()
                    .unwrap()
                    .raw_os_error(),
                Some(if failure == "skill" {
                    libc::EXDEV
                } else {
                    libc::EACCES
                })
            );
            assert_eq!(
                fs::read_to_string(local.join("setup.conf")).unwrap(),
                "saved settings"
            );
            assert_eq!(
                fs::read_to_string(skill.join("SKILL.md")).unwrap(),
                "customized"
            );
            assert!(!local.join("removed-skill").exists());
        }
    }
    #[test]
    fn archive_preserves_drawings_and_skill_ownership_semantics() {
        for ownership in ["owned", "foreign", "unmarked", "symlink"] {
            let (root, repo, profiles) = fixture();
            let drawings = root.path().join("drawings");
            fs::create_dir(&drawings).unwrap();
            fs::write(drawings.join("original.svg"), "original").unwrap();
            let skill = profiles.skill(Client::Codex);
            fs::create_dir_all(&skill).unwrap();
            fs::write(skill.join("SKILL.md"), "customized").unwrap();
            if ownership != "unmarked" {
                fs::write(
                    skill.join(".inkscape-mcp-owner"),
                    if ownership == "foreign" {
                        "/other".to_string()
                    } else {
                        repo.display().to_string()
                    },
                )
                .unwrap();
            }
            if ownership == "symlink" {
                let actual = root.path().join("actual-skill");
                fs::rename(&skill, &actual).unwrap();
                symlink(actual, &skill).unwrap();
            }
            let backup = archive_with(&repo, &skill, |a, b| fs::rename(a, b)).unwrap();
            assert!(!repo.join(".inkscape-mcp-local").exists());
            assert_eq!(
                fs::read_to_string(drawings.join("original.svg")).unwrap(),
                "original"
            );
            assert_eq!(skill.exists(), ownership != "owned");
            assert_eq!(
                backup.join("removed-skill/SKILL.md").exists(),
                ownership == "owned"
            );
        }
    }
    #[test]
    fn occupied_skill_archive_refuses_before_moving() {
        let (_root, repo, profiles) = fixture();
        let skill = profiles.skill(Client::Codex);
        fs::create_dir_all(&skill).unwrap();
        fs::write(
            skill.join(".inkscape-mcp-owner"),
            repo.display().to_string(),
        )
        .unwrap();
        fs::write(repo.join(".inkscape-mcp-local/removed-skill"), "keep").unwrap();
        assert!(archive_with(&repo, &skill, |a, b| fs::rename(a, b)).is_err());
        assert!(skill.is_dir());
    }
}
