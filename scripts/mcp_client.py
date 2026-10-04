#!/usr/bin/env python3
"""Generate/register/remove client configuration and verify bounded MCP STDIO onboarding."""

import argparse
import json
import os
import queue
import shutil
import subprocess
import tempfile
import threading
import time
from pathlib import Path

NAME = "inkscape"


def run(args):
    return subprocess.run(args, capture_output=True, text=True, timeout=30, check=False)


def client_config(client, repo):
    home = Path.home()
    path = (
        Path(os.environ.get("CODEX_HOME", home / ".codex")) / "config.toml"
        if client == "codex"
        else home / ".claude.json"
    )
    if path.is_symlink() or path.parent.is_symlink():
        raise RuntimeError("Client configuration must not be symlinked")
    if not path.exists():
        return None
    if not path.is_file():
        raise RuntimeError("Client configuration must be a regular file")
    if client == "codex":
        import tomllib

        entries = tomllib.loads(path.read_text()).get("mcp_servers", {})
    else:
        entries = json.loads(path.read_text()).get("mcpServers", {})
    entry = entries.get(NAME)
    if entry is not None and (
        entry.get("command") != str(repo / "run-mcp.sh")
        or entry.get("args", [])
        or entry.get("env", {})
        or entry.get("url")
    ):
        raise RuntimeError("Existing inkscape entry differs; preserved. Remove or rename it first.")
    return entry


def probe(command, timeout=30):
    """Validate initialization, discovery, and a read-only workspace request. No GUI."""
    messages = queue.Queue(maxsize=16)
    # stderr goes to an anonymous file: diagnostics remain local and cannot block stdout.

    with tempfile.TemporaryFile() as errors:
        process = subprocess.Popen(
            command,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=errors,
            text=True,
            bufsize=1,
        )

        def read():
            try:
                while line := process.stdout.readline(8 * 1024 * 1024 + 1):
                    if len(line) > 8 * 1024 * 1024:
                        messages.put(RuntimeError("Oversized MCP response"), timeout=0.1)
                        return
                    messages.put(json.loads(line), timeout=0.1)
            except Exception as error:
                try:
                    messages.put(error, timeout=0.1)
                except queue.Full:
                    pass
            finally:
                try:
                    messages.put(RuntimeError("MCP server closed STDIO"), timeout=0.1)
                except queue.Full:
                    pass

        reader = threading.Thread(target=read, daemon=True)
        reader.start()
        sequence = 0

        def send(method, params, notification=False):
            nonlocal sequence
            sequence += 1
            message = {"jsonrpc": "2.0", "method": method, "params": params}
            if not notification:
                message["id"] = sequence
            process.stdin.write(json.dumps(message) + "\n")
            process.stdin.flush()
            if notification:
                return None
            deadline = time.monotonic() + timeout
            while True:
                reply = messages.get(timeout=max(0.01, deadline - time.monotonic()))
                if isinstance(reply, Exception):
                    raise reply
                if reply.get("id") == sequence:
                    if "error" in reply:
                        raise RuntimeError("MCP request failed: " + method)
                    return reply["result"]

        try:
            info = send(
                "initialize",
                {
                    "protocolVersion": "2025-03-26",
                    "capabilities": {},
                    "clientInfo": {"name": "inkscape-install-check", "version": "1"},
                },
            )
            if not info.get("protocolVersion") or not info.get("capabilities", {}).get("tools"):
                raise RuntimeError("Invalid MCP handshake or missing tool capability")
            send("notifications/initialized", {}, True)
            tools = send("tools/list", {})["tools"]
            names = {tool["name"] for tool in tools}
            if not {"get_workspace_info", "create_document", "render_preview"}.issubset(names):
                raise RuntimeError("Required Inkscape tools unavailable")
            result = send("tools/call", {"name": "get_workspace_info", "arguments": {}})
            if result.get("isError"):
                raise RuntimeError("First workspace request failed")
            return {
                "handshake": True,
                "tools": len(tools),
                "first_request": "get_workspace_info",
                "server": info.get("serverInfo"),
            }
        finally:
            process.stdin.close()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.terminate()  # only this explicitly created server; never Inkscape
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
            reader.join(timeout=1)
            process.stdout.close()


def client_record(repo):
    local = repo / ".inkscape-mcp-local"
    record = local / "clients.json"
    if local.is_symlink() or not local.is_dir() or record.is_symlink():
        raise RuntimeError("Invalid installation/client record")
    if record.exists() and not record.is_file():
        raise RuntimeError("Client record must be a regular file")
    clients = json.loads(record.read_text()) if record.exists() else []
    if not isinstance(clients, list) or any(c not in ("codex", "claude") for c in clients):
        raise RuntimeError("Invalid recorded clients")
    return record, clients


def save_clients(record, clients):
    with tempfile.NamedTemporaryFile(mode="w", dir=record.parent, delete=False) as stream:
        temporary = Path(stream.name)
        json.dump(clients, stream)
    try:
        temporary.replace(record)
    finally:
        temporary.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--client", choices=["codex", "claude"], required=True)
    parser.add_argument("action", choices=["config", "connect", "check", "disconnect", "uninstall"])
    args = parser.parse_args()
    repo = args.repo.resolve(strict=True)
    launcher = str(repo / "run-mcp.sh")
    if args.action == "config":
        if args.client == "codex":
            print("[mcp_servers.inkscape]\ncommand = " + json.dumps(launcher) + "\nargs = []")
        else:
            print(
                json.dumps(
                    {"mcpServers": {NAME: {"type": "stdio", "command": launcher, "args": []}}},
                    indent=2,
                )
            )
        return
    if args.action == "check":
        print(json.dumps(probe([launcher]), indent=2))
        return
    cli = shutil.which("codex" if args.client == "codex" else "claude")
    if not cli:
        raise RuntimeError("Client CLI missing; install it or use the config action")
    entry = client_config(args.client, repo)
    record, clients = client_record(repo)
    if args.action == "uninstall" and any(
        client != args.client and client_config(client, repo) is not None for client in clients
    ):
        raise RuntimeError("Another client uses this installation; disconnect it first")
    if args.action == "connect":
        report = probe([launcher])  # complete handshake before changing client settings
        if entry is None:
            command = (
                [cli, "mcp", "add", NAME, "--", launcher]
                if args.client == "codex"
                else [
                    cli,
                    "mcp",
                    "add",
                    "--transport",
                    "stdio",
                    "--scope",
                    "user",
                    NAME,
                    "--",
                    launcher,
                ]
            )
            if run(command).returncode:
                raise RuntimeError("Client registration failed; inspect client CLI diagnostics")
        if client_config(args.client, repo) is None:
            raise RuntimeError("Client registration was not persisted")
        if args.client not in clients:
            clients.append(args.client)
        save_clients(record, clients)
        report["client"] = args.client
        print(json.dumps(report, indent=2))
        print("Restart/reconnect your client to load the server.")
    else:
        if entry is not None:
            command = [cli, "mcp", "remove", NAME]
            if args.client == "claude":
                command += ["--scope", "user"]
            if run(command).returncode or client_config(args.client, repo) is not None:
                raise RuntimeError("Client removal failed; installation retained")
        if args.action == "uninstall":
            # Archive owned setup/build state and skill; workspace drawings survive.
            local = repo / ".inkscape-mcp-local"
            if local.is_symlink() or not local.is_dir():
                raise RuntimeError("Invalid installation directory")
            backup = repo / (".inkscape-mcp-backup-" + str(time.time_ns()))
            skills = (
                Path(os.environ.get("CODEX_HOME", Path.home() / ".codex"))
                if args.client == "codex"
                else Path.home() / ".claude"
            ) / "skills/inkscape-mcp"
            owner = skills / ".inkscape-mcp-owner"
            owned = (
                not skills.is_symlink()
                and not owner.is_symlink()
                and owner.is_file()
                and owner.read_text().strip() == str(repo)
            )
            staged_skill = local / "removed-skill"
            if owned:
                if staged_skill.exists() or staged_skill.is_symlink():
                    raise RuntimeError(
                        "Skill archive destination already exists; installation retained"
                    )
                skills.rename(staged_skill)
            try:
                local.rename(backup)
            except OSError:
                if owned:
                    staged_skill.rename(skills)
                raise
            print("Installation moved to " + str(backup))
        print("Client disconnected. Existing client processes must be restarted by the user.")


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, OSError, ValueError, queue.Empty, subprocess.TimeoutExpired) as error:
        raise SystemExit(str(error) or "MCP handshake timed out") from error
