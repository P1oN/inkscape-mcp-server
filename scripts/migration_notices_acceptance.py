#!/usr/bin/env python3
"""Exercise crate notice integrity/path refusal and verify a real package inventory."""

import argparse
import hashlib
import io
import json
import shutil
import tarfile
from pathlib import Path
from tempfile import TemporaryDirectory

from migration_package_notices import archive_notices, native_source_notices
from migration_probe import require, write


def main(package, report):
    checks = {}
    with TemporaryDirectory(prefix="imcp-notices-") as temporary:
        root = Path(temporary)
        for scenario in ("valid", "checksum", "traversal", "symlink", "duplicate", "oversized"):
            archive = root / (scenario + ".crate")
            with tarfile.open(archive, "w:gz") as stream:
                metadata = b'[package]\nname="fixture"\nversion="1.0.0"\nlicense="MIT"\n'
                entry = tarfile.TarInfo("fixture-1.0.0/Cargo.toml")
                entry.size = len(metadata)
                stream.addfile(entry, io.BytesIO(metadata))
                legal = tarfile.TarInfo("fixture-1.0.0/LICENSE")
                if scenario == "traversal":
                    legal.name = "fixture-1.0.0/../../LICENSE"
                if scenario == "symlink":
                    legal.type = tarfile.SYMTYPE
                    legal.linkname = "/outside"
                    stream.addfile(legal)
                else:
                    content = b"synthetic notice\n"
                    if scenario == "oversized":
                        content = b"x" * (4 * 1024 * 1024 + 1)
                    legal.size = len(content)
                    stream.addfile(legal, io.BytesIO(content))
                    if scenario == "duplicate":
                        stream.addfile(legal, io.BytesIO(content))
            checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
            if scenario == "checksum":
                checksum = "0" * 64
            failure = None
            try:
                metadata, _, notices = archive_notices(archive, checksum, "fixture", "1.0.0")
            except RuntimeError as exception:
                failure = str(exception)
            checks[scenario] = (
                failure is None and metadata["license"] == "MIT" and len(notices) == 1
                if scenario == "valid"
                else failure is not None
            )
        checks["no_archive_extraction"] = not (root / "fixture-1.0.0").exists()

    inventory = json.loads((package / "LICENSE-INVENTORY.json").read_text())
    files = json.loads((package / "FILES.json").read_text())
    hashes = {}
    for row in inventory["rust_crates"]:
        require(row["notices"], "missing real crate notices")
        for notice in row["notices"]:
            path = Path(notice["path"])
            require(not path.is_absolute() and ".." not in path.parts, "notice escaped package")
            target = package / path
            require(target.is_file() and not target.is_symlink(), "missing/linked notice")
            content = target.read_bytes()
            digest = hashlib.sha256(content).hexdigest()
            require(
                len(content) == notice["bytes"] and digest == notice["sha256"],
                "notice hash differs",
            )
            require(files[str(path)]["sha256"] == digest, "FILES notice binding differs")
            hashes[str(path)] = digest
    checks["real_crate_notices_bound_to_FILES"] = bool(hashes)
    supplements = list(inventory.get("native_source_supplements", []))
    supplements.extend(
        inventory[key]
        for key in ("cpython_source_supplement", "glib_build_source_supplement")
        if inventory.get(key)
    )
    for supplement in supplements:
        for notice in supplement["notices"]:
            content = (package / notice["path"]).read_bytes()
            digest = hashlib.sha256(content).hexdigest()
            require(
                len(content) == notice["bytes"] and digest == notice["sha256"],
                "native notice hash differs",
            )
            require(files[notice["path"]]["sha256"] == digest, "native notice FILES differs")
    if inventory.get("cpython_source_supplement"):
        checks["cpython_distribution_notices_bound"] = (
            len(inventory["cpython_source_supplement"]["notices"]) == 21
        )
    if inventory.get("helper_wheel_provenance"):
        wheel_lock = inventory["helper_wheel_provenance"]
        for wheel in wheel_lock["wheels"]:
            for member in wheel["members"]:
                relative = (
                    "libexec/inkscape-mcp/python/lib/python3.12/site-packages/" + member["path"]
                )
                require(
                    files[relative]["sha256"] == member["sha256"], "wheel FILES binding differs"
                )
                require(
                    hashlib.sha256((package / relative).read_bytes()).hexdigest()
                    == member["sha256"],
                    "wheel member changed after collection",
                )
        checks["six_wheels_bound_to_FILES"] = len(wheel_lock["wheels"]) == 6

    if inventory.get("native_source_supplements"):
        native_paths = {
            notice["path"]
            for supplement in inventory["native_source_supplements"]
            for notice in supplement["notices"]
        }
        prefix = "libexec/inkscape-mcp/licenses/native-source/"
        checks["runtime_license_texts_packaged"] = all(
            prefix + relative in native_paths
            for relative in [
                "glib/2.90.0/LICENSES/LGPL-2.1-or-later.txt",
                "dbus/1.16.2/LICENSES/AFL-2.1.txt",
                "dbus/1.16.2/LICENSES/GPL-2.0-or-later.txt",
                "gettext/1.0/gettext-runtime/intl/COPYING.LIB",
                "pcre2/10.48/COPYING",
            ]
        )

    if inventory["target"].startswith("macos"):
        with TemporaryDirectory(prefix="imcp-native-notices-") as temporary:
            root = Path(temporary)
            fixture = root / "package"
            shutil.copytree(
                package / "libexec/inkscape-mcp/licenses/dbus",
                fixture / "libexec/inkscape-mcp/licenses/dbus",
            )
            vendor = root / "vendor"
            shutil.copytree(Path("migration/vendor-notices/native"), vendor)

            def record(content, relative):
                return {
                    "path": str(relative),
                    "sha256": hashlib.sha256(content).hexdigest(),
                    "bytes": len(content),
                }

            rows, gaps = native_source_notices(fixture, record, vendor)
            checks["exact_native_source_supplements"] = len(rows) == 4 and not gaps
            selected = vendor / "glib/2.90.0/provenance.json"
            saved = selected.read_bytes()
            for scenario in ("archive_checksum", "upstream_url", "notice_checksum", "traversal"):
                value = json.loads(saved)
                if scenario == "archive_checksum":
                    value["sha256"] = "0" * 64
                elif scenario == "upstream_url":
                    value["url"] = "https://invalid.example/source"
                elif scenario == "notice_checksum":
                    value["notices"][0]["sha256"] = "0" * 64
                else:
                    value["notices"][0]["path"] = "../../outside"
                selected.write_text(json.dumps(value))
                refused = False
                try:
                    native_source_notices(fixture, record, vendor)
                except RuntimeError:
                    refused = True
                checks["native_refuses_" + scenario] = refused
                selected.write_bytes(saved)
    checks["audit_gaps_explicit"] = inventory["redistribution_audit_complete"] is False and bool(
        inventory["remaining_gaps"]
    )
    result = {
        "passed": all(checks.values()),
        "checks": checks,
        "rust_crates": len(inventory["rust_crates"]),
        "notice_hashes": hashes,
        "scope": (
            "Crate source-archive integrity/path guards and copied notices only; "
            "not legal clearance."
        ),
    }
    write(report, result)
    print(
        json.dumps(
            {"passed": result["passed"], "checks": len(checks), "crates": result["rust_crates"]}
        )
    )
    require(result["passed"], "notice checks failed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    main(args.package, args.report)
