"""Setup diagnosis is bounded and does not create/start/repair a session."""

import json
import sys
import tempfile
from collections.abc import Iterator
from pathlib import Path

import pytest

from inkscape_mcp.live import macos_diagnostics as diag
from inkscape_mcp.live import macos_launcher


@pytest.fixture
def short_directory() -> Iterator[Path]:
    # macOS pytest paths often already exceed the Unix socket limit.
    with tempfile.TemporaryDirectory(
        prefix="imcp-", dir="/tmp" if Path("/tmp").is_dir() else None
    ) as directory:
        yield Path(directory)


@pytest.fixture
def environment(short_directory: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    tmp_path = short_directory
    monkeypatch.setattr(diag.sys, "platform", "darwin")
    binary = tmp_path / "Inkscape.app/Contents/MacOS/inkscape"
    vendor = binary.parent.parent / "Resources/share/inkscape/extensions/inkex"
    vendor.mkdir(parents=True)
    data = tmp_path / "profile"
    (data / "extensions").mkdir(parents=True)
    monkeypatch.setattr(
        diag.shutil, "which", lambda name: str(binary) if name == "inkscape" else name
    )
    monkeypatch.setattr(diag.importlib.util, "find_spec", lambda name: object())
    monkeypatch.setattr(
        diag,
        "_stdout",
        lambda argv: "Inkscape 1.4.3" if argv[-1] == "--version" else str(data),
    )
    monkeypatch.setattr(
        diag,
        "build_dependencies",
        lambda: {"clang": True, "codesign": True, "glib_headers": True},
    )
    monkeypatch.setattr(macos_launcher, "_read_session", lambda root: None)
    return tmp_path / "session"


def test_fresh_setup_is_ready_and_creates_nothing(environment: Path) -> None:
    before = sorted(environment.parent.rglob("*"))
    report = diag.diagnose_macos(environment)
    assert report.ready and report.state == "ready_to_launch"
    assert report.inkscape_version == "Inkscape 1.4.3"
    assert report.helper_installed is False
    assert sorted(environment.parent.rglob("*")) == before


def test_missing_dependencies_do_not_probe_a_session(
    environment: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(diag.shutil, "which", lambda name: None)
    monkeypatch.setattr(diag, "_stdout", lambda argv: pytest.fail("must not run CLI"))
    report = diag.diagnose_macos(environment)
    assert not report.ready and report.state == "missing_dependencies"
    assert any("brew install" in step for step in report.next_steps)


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX ownership/permissions")
@pytest.mark.parametrize("unsafe", ["public", "symlink"])
def test_unsafe_directory_is_not_read_or_repaired(
    environment: Path, monkeypatch: pytest.MonkeyPatch, unsafe: str
) -> None:
    if unsafe == "public":
        environment.mkdir(mode=0o755)
    else:
        environment.symlink_to(environment.parent, target_is_directory=True)
    monkeypatch.setattr(
        macos_launcher, "_read_session", lambda root: pytest.fail("unsafe root must not be read")
    )
    report = diag.diagnose_macos(environment)
    assert not report.ready and report.state == "unsafe_session"
    assert (
        environment.is_symlink()
        if unsafe == "symlink"
        else environment.stat().st_mode & 0o777 == 0o755
    )


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX ownership/permissions")
def test_unreachable_session_is_preserved(environment: Path) -> None:
    environment.mkdir(mode=0o700)
    manifest = environment / "session.json"
    manifest.write_text("stale metadata")
    report = diag.diagnose_macos(environment)
    assert report.state == "session_unavailable" and not report.ready
    assert manifest.read_text() == "stale metadata"
    assert any("Save and close" in step for step in report.next_steps)


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX ownership/permissions")
@pytest.mark.parametrize("enabled", [True, False])
def test_running_session_checks_the_native_helper(
    environment: Path, monkeypatch: pytest.MonkeyPatch, enabled: bool
) -> None:
    environment.mkdir(mode=0o700)
    (environment / "inkscape.stdout.log").write_text("")
    monkeypatch.setattr(
        macos_launcher, "_read_session", lambda root: {"address": "private-test-bus"}
    )
    cli = diag._stdout
    monkeypatch.setattr(
        diag,
        "_stdout",
        lambda argv: (
            f"(({str(enabled).lower()}, signature '', @av []),)"
            if "--address" in argv
            else cli(argv)
        ),
    )
    report = diag.diagnose_macos(environment)
    assert report.ready is enabled and report.insertion_available is enabled
    assert report.state == ("running" if enabled else "running_helper_unavailable")


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX ownership/permissions")
def test_missing_selection_output_is_not_reported_ready(
    environment: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    environment.mkdir(mode=0o700)
    monkeypatch.setattr(
        macos_launcher, "_read_session", lambda root: {"address": "private-test-bus"}
    )
    report = diag.diagnose_macos(environment)
    assert report.state == "session_unavailable" and not report.ready
    assert any("selection output is missing" in step for step in report.next_steps)


def test_file_access_fault_is_structured(
    environment: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    def denied(argv: list[str]) -> str:
        raise PermissionError("private host path")

    monkeypatch.setattr(diag, "_stdout", denied)
    report = diag.diagnose_macos(environment)
    assert report.state == "diagnosis_failed"
    assert "private host path" not in report.model_dump_json()


def test_unsupported_platform_does_not_inspect_setup(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(diag.sys, "platform", "win32")
    monkeypatch.setattr(diag.shutil, "which", lambda name: pytest.fail("must not inspect"))
    assert diag.diagnose_macos(tmp_path).state == "unsupported_platform"


def test_doctor_cli_never_launches_and_returns_readiness_exit_code(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setattr(sys, "argv", ["inkscape-mcp-macos", "--doctor"])
    monkeypatch.setenv("PATH", "")
    monkeypatch.setattr(
        diag, "diagnose_macos", lambda root: diag.MacOSDiagnosis(state="missing_dependencies")
    )
    monkeypatch.setattr(
        macos_launcher, "ensure_session", lambda *args: pytest.fail("must not launch")
    )
    with pytest.raises(SystemExit) as result:
        macos_launcher.main()
    assert result.value.code == 1
    assert json.loads(capsys.readouterr().out)["state"] == "missing_dependencies"


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX file locking")
@pytest.mark.parametrize("held", [False, True])
def test_existing_supervisor_lock_is_probed_without_changing_files(
    environment: Path, held: bool
) -> None:
    import fcntl

    environment.mkdir(mode=0o700)
    lock_path = environment / "supervisor.lock"
    lock_path.write_text("preserved")
    before = sorted(environment.rglob("*"))
    with lock_path.open("rb") as lock:
        if held:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        report = diag.diagnose_macos(environment)
        assert report.ready is (not held)
        assert report.state == ("session_unavailable" if held else "ready_to_launch")
    assert lock_path.read_text() == "preserved"
    assert sorted(environment.rglob("*")) == before


@pytest.mark.parametrize("exists", [False, True])
@pytest.mark.parametrize("socket_bytes", [103, 104])
def test_socket_path_limit_prevents_false_readiness(
    environment: Path, exists: bool, socket_bytes: int
) -> None:
    import os

    parent = environment.parent.resolve()
    padding = socket_bytes - len(os.fsencode(parent / "bus.sock")) - 1
    root = parent / ("s" * padding)
    assert len(os.fsencode(root / "bus.sock")) == socket_bytes
    if exists:
        root.mkdir(mode=0o700)
    report = diag.diagnose_macos(root)
    assert report.ready is (socket_bytes < 104)
    assert report.state == ("ready_to_launch" if socket_bytes < 104 else "unsafe_session")
    if socket_bytes >= 104:
        assert any("shorter" in step for step in report.next_steps)
    assert root.exists() is exists


def test_new_launch_requires_context_build_tools(
    environment: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(
        diag,
        "build_dependencies",
        lambda: {"clang": False, "codesign": True, "glib_headers": False},
    )
    report = diag.diagnose_macos(environment)
    assert not report.ready and report.state == "missing_dependencies"
    assert not environment.exists()
    assert any("command line tools" in step for step in report.next_steps)


@pytest.mark.parametrize("available", [True, False])
def test_doctor_checks_bridge_without_activating_or_repairing(
    environment: Path, monkeypatch: pytest.MonkeyPatch, available: bool
) -> None:
    environment.mkdir(mode=0o700)
    (environment / "inkscape.stdout.log").touch()
    monkeypatch.setattr(
        macos_launcher,
        "_read_session",
        lambda root: {"address": "unix:path=/private/test/bus.sock", "context_bridge": True},
    )
    original = diag._stdout

    def output(argv: list[str]) -> str | None:
        if argv[-1] == diag.INSERT_ACTION:
            return "((true, signature '', @av []),)"
        if argv[-1] == f"{diag.INTERFACE}.ListDocuments":
            return "(@a(sss) [],)" if available else None
        return original(argv)

    monkeypatch.setattr(diag, "_stdout", output)
    before = sorted(environment.rglob("*"))
    report = diag.diagnose_macos(environment)
    assert report.context_bridge_available is available
    assert report.ready is available
    assert report.state == ("running" if available else "running_context_unavailable")
    assert sorted(environment.rglob("*")) == before
