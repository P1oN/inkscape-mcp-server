"""Install failure recovery and source/build identity regressions; isolated files only."""

import errno
import importlib.util
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
