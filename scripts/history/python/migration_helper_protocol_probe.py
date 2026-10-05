#!/usr/bin/env python3
"""Investigate the packaged one-shot effect contract offline, without launching GUI.

This executes only the fixed shipped helper in owned synthetic directories. It is
not a Rust helper or proof that a future native extension preserves Inkscape Undo.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import sys
import time
from pathlib import Path

from lxml import etree

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "runtime"))
from insert_payload import document_fingerprint

SVG = b"""<?xml version="1.0" encoding="UTF-8"?>
<!-- protocol prolog --><?protocol preserved?>
<!DOCTYPE svg>
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80">
 <rect id="selected" width="10" height="10" style="fill:blue"/>
 <text id="text"><tspan id="run">Hello</tspan></text>
</svg>"""
EPILOG = b"<?protocol after?><!-- protocol epilog -->"
NONCE = "mcp_" + "a" * 32


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument(
        "--helper-root", type=Path, help="Owned snapshot of the fixed source helpers"
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    package = args.package.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    output.chmod(0o700)
    payload = package / "libexec/inkscape-mcp"
    python = payload / "python/bin/python3"
    helpers = args.helper_root.resolve() if args.helper_root else payload / "helpers"
    helper = helpers / "inkscape_mcp_insert.py"
    vendor = Path("/Applications/Inkscape.app/Contents/Resources/share/inkscape/extensions")
    input_root = etree.fromstring(SVG)
    expected_ids = [e.get("id") for e in input_root.iter() if e.get("id")]
    base = {
        "nonce": NONCE,
        "operation": "style",
        "selection": ["selected"],
        "expected_ids": expected_ids,
        "expected_fingerprint": document_fingerprint(input_root),
        "style": {"fill": "blue"},
    }
    cases = [
        ("style-noop", {}, ["selected"], True, False),
        ("style-change", {"style": {"fill": "red"}}, ["selected"], True, True),
        ("style-with-stylesheet", {"style": {"fill": "red"}}, ["selected"], True, True),
        (
            "style-with-stylesheet-transform",
            {"style": {"fill": "red"}, "transform": "translate(5, 3)"},
            ["selected"],
            True,
            True,
        ),
        ("selection-mismatch", {}, ["text"], False, False),
        ("id-mismatch", {"expected_ids": ["selected"]}, ["selected"], False, False),
        ("fingerprint-mismatch", {"expected_fingerprint": "0" * 64}, ["selected"], False, False),
        (
            "text-change",
            {"operation": "text", "selection": ["text"], "text": "New"},
            ["text"],
            True,
            True,
        ),
        (
            "text-noop",
            {"operation": "text", "selection": ["text"], "text": "Hello"},
            ["text"],
            True,
            False,
        ),
        ("duplicate", {"operation": "duplicate"}, ["selected"], True, True),
        ("insert", {"operation": None, "fragment": '<circle id="new" r="5"/>'}, [], True, True),
        ("insert-refused", {"operation": None, "fragment": "<script/>"}, [], False, False),
    ]
    observations = []
    for mode in ("file", "stdin"):
        for name, overrides, selection, success, changed in cases:
            directory = output / f"{mode}-{name}"
            directory.mkdir(mode=0o700)
            (directory / "home").mkdir()
            source = directory / "input.svg"
            # inkex's baseline deepcopy reverses multiple epilog siblings even on no-op.
            # Preserve that failed probe; test epilog fidelity separately on actual edits.
            case_input = SVG + EPILOG if changed else SVG
            if name.startswith("style-with-stylesheet"):
                case_input = case_input.replace(
                    b' <rect id="selected"',
                    b' <style id="stylesheet">rect {stroke:black}</style>\n <rect id="selected"',
                )
            case_root = etree.fromstring(case_input)
            source.write_bytes(case_input)
            request = {**base, **overrides}
            case_ids = [e.get("id") for e in case_root.iter() if e.get("id")]
            request["expected_ids"] = case_ids
            request["expected_fingerprint"] = document_fingerprint(case_root)
            if name == "id-mismatch":
                request["expected_ids"] = ["selected"]
            elif name == "fingerprint-mismatch":
                request["expected_fingerprint"] = "0" * 64
            (directory / "insert-request.json").write_text(json.dumps(request) + "\n")
            command = [str(python), "-B", str(helper), *[f"--id={oid}" for oid in selection]]
            if mode == "file":
                command.append(str(source))
            env = {
                "HOME": str(directory / "home"),
                "TMPDIR": str(directory),
                "PATH": "",
                "LANG": "en_US.UTF-8",
                "PYTHONPATH": str(vendor),
                "PYTHONDONTWRITEBYTECODE": "1",
                "INKSCAPE_MCP_MANAGED_DIR": str(directory),
            }
            started = time.perf_counter_ns()
            child = subprocess.run(
                command,
                input=case_input if mode == "stdin" else None,
                capture_output=True,
                env=env,
                timeout=20,
                check=False,
            )
            elapsed = time.perf_counter_ns() - started
            (directory / "stdout.svg").write_bytes(child.stdout)
            (directory / "stderr.txt").write_bytes(child.stderr)
            result = json.loads((directory / "insert-result.json").read_text())
            # Refused edits return no SVG output; refused insertion uses AbortExtension.
            checks = {
                "result_identity": result["nonce"] == NONCE,
                "result_success": result["ok"] is success,
                "stdout_presence": bool(child.stdout) is changed,
                "input_unchanged": source.read_bytes() == case_input,
                "request_unchanged": json.loads((directory / "insert-request.json").read_text())
                == request,
                "atomic_result_complete": not (directory / "insert-result.tmp").exists(),
                "result_private": (directory / "insert-result.json").stat().st_mode & 0o777
                == 0o600,
                "returncode": (child.returncode == 0) is (name != "insert-refused"),
            }
            if changed and result["ok"]:
                after = etree.fromstring(child.stdout)

                def outside_root(root):
                    before, following = [], []
                    node = root.getprevious()
                    while node is not None:
                        before.append(etree.tostring(node))
                        node = node.getprevious()
                    node = root.getnext()
                    while node is not None:
                        following.append(etree.tostring(node))
                        node = node.getnext()
                    return before[::-1], following

                checks["document_prolog_epilog_preserved"] = outside_root(after) == outside_root(
                    case_root
                )
                checks["document_doctype_preserved"] = (
                    after.getroottree().docinfo.doctype == case_root.getroottree().docinfo.doctype
                )
                ids = [e.get("id") for e in after.iter() if e.get("id")]
                checks["svg_unique_ids"] = len(ids) == len(set(ids))
                checks["existing_ids_retained"] = set(case_ids) <= set(ids)
                if request.get("operation"):
                    checks["result_fingerprint_matches_output"] = result[
                        "fingerprint"
                    ] == document_fingerprint(after)
                if name == "style-change" or name.startswith("style-with-stylesheet"):
                    checks["changed_style"] = (
                        after.xpath('//*[@id="selected"]')[0].get("style") == "fill:red"
                    )
                    if name.endswith("-transform"):
                        checks["changed_transform"] = bool(
                            after.xpath('//*[@id="selected"]')[0].get("transform")
                        )
                elif name == "text-change":
                    checks["changed_text"] = after.xpath('//*[@id="run"]')[0].text == "New"
                elif name == "insert":
                    checks["inserted_wrapper"] = after[-1].get("id") == NONCE
                    checks["inserted_child"] = after[-1][0].get("id") == NONCE + "_0"
            row = {
                "name": name,
                "mode": mode,
                "command": command,
                "env": env,
                "elapsed_ns": elapsed,
                "input_sha256": digest(case_input),
                "input_has_epilog": changed,
                "exit_code": child.returncode,
                "stdout_bytes": len(child.stdout),
                "stdout_sha256": digest(child.stdout),
                "stderr_sha256": digest(child.stderr),
                "result": result,
                "checks": checks,
                "passed": all(checks.values()),
            }
            (directory / "observation.json").write_text(json.dumps(row, indent=2) + "\n")
            observations.append(row)
    paths = [python, helper, vendor / "inkex/base.py", vendor / "inkex/extensions.py"]
    paths.extend(sorted(helpers.glob("*")))
    report = {
        "passed": all(row["passed"] for row in observations),
        "platform": platform.platform(),
        "package": str(package),
        "helper_root": str(helpers),
        "helper_override": args.helper_root is not None,
        "fixture_limits": (
            "Two epilog siblings are included only in changed cases. Vendor inkex's "
            "baseline deepcopy reverses their order and emits SVG even on no-op/refusal. "
            "The failed all-epilog observations are retained separately; this does not "
            "establish no-op compatibility for documents with multiple epilog siblings."
        ),
        "fixed_input_sha256": digest(SVG),
        "source_hashes": {str(p): digest(p.read_bytes()) for p in paths if p.is_file()},
        "observations": observations,
        "scope": (
            "Real packaged Python/vendor inkex executed offline with the recorded helper root. "
            "No GUI, effect dispatch or native Undo tested; no Rust helper implemented."
        ),
        "sources": [
            "https://inkscape.gitlab.io/extensions/documentation/authors/inx-overview.html",
            "https://wiki.inkscape.org/wiki/Script_extensions",
        ],
    }
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(f"Helper protocol: {sum(r['passed'] for r in observations)}/{len(observations)} passed")
    if not report["passed"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
