"""Install failure recovery and source/build identity regressions; isolated files only."""

import errno
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]


def load_script(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()


@pytest.fixture
def source_repo(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    git(repo, "init", "--quiet")
    git(repo, "config", "user.name", "Install regression")
    git(repo, "config", "user.email", "regression@example.invalid")
    (repo / "rust").mkdir()
    (repo / "rust/source.txt").write_text("committed\n")
    git(repo, "add", ".")
    git(repo, "commit", "--quiet", "-m", "fixture")
    return repo


def test_working_tree_archive_does_not_claim_committed_revision(source_repo, monkeypatch, tmp_path):
    monkeypatch.chdir(source_repo)
    exporter = load_script("build_source_archive")
    revision = git(source_repo, "rev-parse", "HEAD")
    (source_repo / "rust/source.txt").write_text("uncommitted\n")
    for working_tree in (False, True):
        output = tmp_path / (str(working_tree) + ".tar.gz")
        exporter.main(output, working_tree=working_tree)
        with tarfile.open(output) as stream:
            prefix = "inkscape-mcp-source-bootstrap/"
            assert (prefix + "SOURCE_REVISION" in stream.getnames()) == (not working_tree)
            content = stream.extractfile(prefix + "rust/source.txt").read()
            assert content == (b"uncommitted\n" if working_tree else b"committed\n")
            if working_tree:
                assert (
                    stream.extractfile(prefix + "SOURCE_STATE").read()
                    == b"uncommitted-working-tree\n"
                )
            else:
                assert (
                    stream.extractfile(prefix + "SOURCE_REVISION").read().decode().splitlines()[1]
                    == revision
                )


@pytest.mark.parametrize("linked", [False, True])
def test_build_script_tracks_actual_git_paths(source_repo, tmp_path, linked):
    rustc = shutil.which("rustc") or str(Path.home() / ".cargo/bin/rustc")
    git(source_repo, "pack-refs", "--all")
    checkout = source_repo
    if linked:
        checkout = tmp_path / "linked"
        git(source_repo, "worktree", "add", "--detach", str(checkout))
        assert (checkout / ".git").is_file()
    executable = tmp_path / "build-script"
    subprocess.run(
        [rustc, str(ROOT / "rust/build.rs"), "--edition=2024", "-o", str(executable)],
        env={**os.environ, "CARGO_MANIFEST_DIR": str(checkout / "rust")},
        check=True,
        capture_output=True,
        text=True,
    )
    output = subprocess.check_output([str(executable)], text=True)
    for item in ("HEAD", "refs", "packed-refs"):
        path = git(checkout, "rev-parse", "--path-format=absolute", "--git-path", item)
        assert Path(path).exists()
        assert "cargo:rerun-if-changed=" + path + "\n" in output
    assert "cargo:rustc-env=INKSCAPE_MCP_REVISION=" + git(checkout, "rev-parse", "HEAD") in output
    (checkout / "rust/source.txt").write_text("next commit\n")
    git(checkout, "add", ".")
    git(checkout, "commit", "--quiet", "-m", "next fixture")
    next_output = subprocess.check_output([str(executable)], text=True)
    assert (
        "cargo:rustc-env=INKSCAPE_MCP_REVISION=" + git(checkout, "rev-parse", "HEAD") in next_output
    )
    assert next_output != output


@pytest.mark.parametrize("failure", ["skill", "settings"])
def test_uninstall_rename_failure_preserves_settings_and_owned_skill(
    tmp_path, monkeypatch, failure
):
    client = load_script("mcp_client")
    repo = tmp_path / "repo"
    local = repo / ".inkscape-mcp-local"
    local.mkdir(parents=True)
    (local / "setup.conf").write_text("saved settings\n")
    codex_home = tmp_path / "codex"
    skills = codex_home / "skills/inkscape-mcp"
    skills.mkdir(parents=True)
    (skills / "SKILL.md").write_text("user customizations\n")
    (skills / ".inkscape-mcp-owner").write_text(str(repo) + "\n")
    monkeypatch.setenv("CODEX_HOME", str(codex_home))
    monkeypatch.setattr(
        sys, "argv", ["mcp_client", "--repo", str(repo), "--client", "codex", "uninstall"]
    )
    monkeypatch.setattr(client.shutil, "which", lambda _: "codex")
    monkeypatch.setattr(client, "client_config", lambda *_: None)
    original_rename = Path.rename
    failing_path = skills if failure == "skill" else local

    def rename(path, destination):
        if path == failing_path:
            raise OSError(
                errno.EXDEV if failure == "skill" else errno.EACCES, "injected rename failure"
            )
        return original_rename(path, destination)

    monkeypatch.setattr(Path, "rename", rename)
    with pytest.raises(OSError, match="injected rename failure"):
        client.main()
    assert (local / "setup.conf").read_text() == "saved settings\n"
    assert (skills / "SKILL.md").read_text() == "user customizations\n"
    assert not (local / "removed-skill").exists()
    assert not list(repo.glob(".inkscape-mcp-backup-*"))


@pytest.mark.parametrize("notifications", [False, True])
def test_client_probe_deadline_survives_unrelated_notifications(monkeypatch, notifications):
    client = load_script("mcp_client")
    child = (
        "import json,sys,time\n"
        "request=json.loads(sys.stdin.readline())\n"
        "deadline=time.monotonic()+0.8\n"
        "while time.monotonic()<deadline:\n"
        f" if {notifications!r}:\n"
        "  print(json.dumps({'jsonrpc':'2.0','method':'notifications/message'}),flush=True)\n"
        " time.sleep(0.002)\n"
        "print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':{}}),flush=True)\n"
    )
    original_popen = client.subprocess.Popen
    processes = []

    def popen(*args, **kwargs):
        process = original_popen(*args, **kwargs)
        processes.append(process)
        return process

    monkeypatch.setattr(client.subprocess, "Popen", popen)
    with pytest.raises(TimeoutError, match="MCP request timed out: initialize"):
        client.probe([sys.executable, "-u", "-c", child], timeout=0.2)
    assert len(processes) == 1
    assert processes[0].poll() is not None
    assert processes[0].stdin.closed
    assert processes[0].stdout.closed


def test_client_probe_accepts_notifications_before_timely_replies():
    client = load_script("mcp_client")
    child = (
        "import json,sys\n"
        "for line in sys.stdin:\n"
        " request=json.loads(line)\n"
        " if 'id' not in request: continue\n"
        " print(json.dumps({'jsonrpc':'2.0','method':'notifications/message'}),flush=True)\n"
        " if request['method']=='initialize':\n"
        "  result={'protocolVersion':'2025-03-26','capabilities':{'tools':{'listChanged':False}}}\n"
        " elif request['method']=='tools/list':\n"
        "  result={'tools':[{'name':n} for n in "
        "('get_workspace_info','create_document','render_preview')]}\n"
        " else: result={'isError':False}\n"
        " print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result}),flush=True)\n"
    )
    assert client.probe([sys.executable, "-u", "-c", child], timeout=2)["handshake"]


@pytest.mark.parametrize("other_installation", [True, False])
def test_uninstall_only_blocks_another_client_using_this_installation(
    tmp_path, monkeypatch, other_installation
):
    client = load_script("mcp_client")
    repo = tmp_path / "repo"
    local = repo / ".inkscape-mcp-local"
    local.mkdir(parents=True)
    (local / "setup.conf").write_text("saved settings\n")
    (local / "clients.json").write_text('["codex", "claude"]')
    codex_home = tmp_path / "codex"
    codex_home.mkdir()
    home = tmp_path / "home"
    home.mkdir()
    monkeypatch.setenv("CODEX_HOME", str(codex_home))
    monkeypatch.setattr(client.Path, "home", classmethod(lambda _: home))
    launcher = (tmp_path / "other" if other_installation else repo) / "run-mcp.sh"
    config = home / ".claude.json"
    content = json.dumps({"mcpServers": {"inkscape": {"command": str(launcher), "args": []}}})
    config.write_text(content)
    monkeypatch.setattr(client.shutil, "which", lambda _: "codex")
    monkeypatch.setattr(
        client, "run", lambda _: pytest.fail("Uninstall must not modify the other client")
    )
    monkeypatch.setattr(
        sys, "argv", ["mcp_client", "--repo", str(repo), "--client", "codex", "uninstall"]
    )
    if other_installation:
        # Target-client actions must still refuse a differing existing entry.
        with pytest.raises(RuntimeError, match="Existing inkscape entry differs"):
            client.client_config("claude", repo)
        client.main()
        assert not local.exists()
        backups = list(repo.glob(".inkscape-mcp-backup-*"))
        assert len(backups) == 1
        assert (backups[0] / "setup.conf").read_text() == "saved settings\n"
    else:
        with pytest.raises(RuntimeError, match="Another client uses this installation"):
            client.main()
        assert (local / "setup.conf").read_text() == "saved settings\n"
        assert not list(repo.glob(".inkscape-mcp-backup-*"))
    assert config.read_text() == content
