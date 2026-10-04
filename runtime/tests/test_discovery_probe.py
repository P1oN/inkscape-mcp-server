"""Exercise discovery harness diagnostics against an owned synthetic STDIO peer."""

import importlib.util
import json
import os
import sys
from pathlib import Path

import pytest

spec = importlib.util.spec_from_file_location(
    "migration_probe", Path(__file__).resolve().parents[2] / "scripts/migration_probe.py"
)
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


@pytest.mark.parametrize("mode", ["reply", "silent", "invalid_utf8", "eof"])
def test_probe_reports_pending_request_and_cleans_up_reader(tmp_path, mode):
    child = (
        "import json,sys\n"
        "for line in sys.stdin:\n"
        f" mode={mode!r}\n"
        " if mode=='eof': break\n"
        " if mode=='silent': continue\n"
        " if mode=='invalid_utf8':\n"
        "  sys.stdout.buffer.write(bytes([255,10]));sys.stdout.buffer.flush();continue\n"
        " request=json.loads(line)\n"
        " print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':{}}),flush=True)\n"
    )
    wire = probe.Wire(
        [sys.executable, "-u", "-c", child],
        os.environ.copy(),
        tmp_path / "server.stderr.log",
        request_timeout=0.5,
    )
    try:
        if mode == "reply":
            assert wire.request("tools/list")["result"] == {}
            assert wire.pending is None
            assert wire.trace[0]["request"]["method"] == "tools/list"
        else:
            error = TimeoutError if mode == "silent" else RuntimeError
            match = {
                "silent": r"method=tools/list, id=1, pid=\d+, exit_code=None",
                "invalid_utf8": "reader_error",
                "eof": "eof",
            }[mode]
            with pytest.raises(error, match=match):
                wire.request("tools/list")
            assert wire.pending == {"jsonrpc": "2.0", "id": 1, "method": "tools/list"}
    finally:
        wire.close()
    assert wire.process.poll() is not None
    assert not wire.reader.is_alive()
    assert wire.process.stdout.closed
    assert wire.log.closed


def test_failed_discovery_retains_completed_and_pending_requests(tmp_path, monkeypatch):
    child = (
        "import json,sys\n"
        "for line in sys.stdin:\n"
        " request=json.loads(line)\n"
        " if request['method']=='initialize':\n"
        "  print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':{}}),flush=True)\n"
    )
    original_wire = probe.Wire
    monkeypatch.setattr(
        probe,
        "Wire",
        lambda command, env, log: original_wire(command, env, log, request_timeout=0.5),
    )
    output = tmp_path / "discovery"
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "migration_probe",
            "--output",
            str(output),
            "--discovery-only",
            "--",
            sys.executable,
            "-u",
            "-c",
            child,
        ],
    )
    with pytest.raises(TimeoutError, match="method=tools/list"):
        probe.main()
    trace = json.loads((output / "live-true_raw-true_full_full.trace.json").read_text())
    assert trace["completed"][0]["request"]["method"] == "initialize"
    assert trace["pending"] == {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}
