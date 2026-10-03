#!/usr/bin/env python3
"""Attribute packaged helper files to exact SHA-256-verified PyPI wheels; no extraction."""

import argparse
import base64
import csv
import hashlib
import io
import itertools
import json
import urllib.request
import zipfile
from pathlib import Path, PurePosixPath

from migration_probe import require


def fetch(url, cap):
    require(
        url.startswith("https://pypi.org/") or url.startswith("https://files.pythonhosted.org/"),
        "unexpected package origin",
    )
    with urllib.request.urlopen(url, timeout=45) as response:  # noqa: S310 HTTPS origins checked above
        content = response.read(cap + 1)
    require(len(content) <= cap, "package download exceeds cap")
    return content


def main(package, output):
    output.mkdir(parents=True, exist_ok=False)
    site = package / "libexec/inkscape-mcp/python/lib/python3.12/site-packages"
    rows = []
    for line in Path("rust/package/helper-requirements.txt").read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        name, version = line.split("==")
        metadata_url = f"https://pypi.org/pypi/{name}/{version}/json"
        metadata_bytes = fetch(metadata_url, 4 * 1024 * 1024)
        metadata = json.loads(metadata_bytes)
        (output / (name + "-pypi.json")).write_bytes(metadata_bytes)
        info = site / (name + "-" + version + ".dist-info")
        wheel_metadata = (info / "WHEEL").read_bytes()
        tags = {
            line[5:].strip()
            for line in wheel_metadata.decode().splitlines()
            if line.startswith("Tag: ")
        }
        candidates = []
        for row in metadata["urls"]:
            if row["packagetype"] != "bdist_wheel":
                continue
            python, abi, platform = row["filename"][:-4].split("-")[-3:]
            expanded = {
                "-".join(parts)
                for parts in itertools.product(
                    python.split("."), abi.split("."), platform.split(".")
                )
            }
            if expanded == tags:
                candidates.append(row)
        require(len(candidates) == 1, "ambiguous wheel origin: " + name)
        artifact = candidates[0]
        wheel_bytes = fetch(artifact["url"], 64 * 1024 * 1024)
        digest = hashlib.sha256(wheel_bytes).hexdigest()
        require(digest == artifact["digests"]["sha256"], "wheel checksum differs")
        wheel_path = output / artifact["filename"]
        wheel_path.write_bytes(wheel_bytes)
        exact, records = [], []
        with zipfile.ZipFile(wheel_path) as archive:
            require(len(archive.infolist()) <= 100000, "wheel member cap")
            for member in archive.infolist():
                if member.is_dir():
                    continue
                relative = PurePosixPath(member.filename)
                require(
                    not relative.is_absolute() and ".." not in relative.parts, "unsafe wheel member"
                )
                require(member.file_size <= 64 * 1024 * 1024, "wheel member size cap")
                installed = site / str(relative)
                require(
                    installed.is_file() and not installed.is_symlink(),
                    "packaged wheel member missing: " + str(relative),
                )
                upstream = archive.read(member)
                actual = installed.read_bytes()
                if relative.name == "RECORD" and relative.parent.name.endswith(".dist-info"):
                    # uv adds its own installation metadata to RECORD. Preserve both exact
                    # inputs for audit rather than ignoring every dist-info file.
                    (output / (name + "-RECORD-upstream.txt")).write_bytes(upstream)
                    (output / (name + "-RECORD-installed.txt")).write_bytes(actual)
                    before_rows = list(csv.reader(io.StringIO(upstream.decode())))
                    after_rows = list(csv.reader(io.StringIO(actual.decode())))
                    before = {row[0]: row[1:] for row in before_rows}
                    after = {row[0]: row[1:] for row in after_rows}
                    require(
                        len(before) == len(before_rows) and len(after) == len(after_rows),
                        "duplicate wheel RECORD member",
                    )
                    require(
                        all(after.get(key) == value for key, value in before.items()),
                        "upstream RECORD row changed or removed",
                    )
                    extras = after.keys() - before.keys()
                    dist_info = str(relative.parent)
                    permitted = {dist_info + "/INSTALLER", dist_info + "/REQUESTED"}
                    if name == "numpy":
                        # Builder deliberately omits installer-created development CLIs.
                        permitted |= {"../../../bin/f2py", "../../../bin/numpy-config"}
                    require(extras == permitted, "unexplained installer RECORD rows")
                    for filename in extras:
                        if filename.startswith("../../../bin/"):
                            require(
                                not (package / "bin" / Path(filename).name).exists(),
                                "development wheel CLI unexpectedly packaged",
                            )
                            continue
                        added = (site / filename).read_bytes()
                        expected = b"uv" if filename.endswith("/INSTALLER") else b""
                        require(added == expected, "unexpected installer metadata content")
                        installer_digest = base64.urlsafe_b64encode(hashlib.sha256(added).digest())
                        require(
                            after[filename]
                            == ["sha256=" + installer_digest.decode().rstrip("="), str(len(added))],
                            "installer RECORD hash differs",
                        )
                    records.append(str(relative))
                else:
                    require(upstream == actual, "packaged wheel member differs: " + str(relative))
                    exact.append(str(relative))
        require(str(info.relative_to(site) / "WHEEL") in exact, "wheel metadata not matched")
        rows.append(
            {
                "name": name,
                "version": version,
                "metadata_url": metadata_url,
                "wheel_url": artifact["url"],
                "wheel": artifact["filename"],
                "sha256": digest,
                "bytes": len(wheel_bytes),
                "exact_members": exact,
                "RECORD_inputs_preserved": records,
                "declared_license": metadata["info"].get("license_expression")
                or metadata["info"].get("license"),
            }
        )
        print(name, version, len(exact), "exact members", flush=True)
    result = {
        "passed": True,
        "package": str(package.resolve()),
        "wheels": rows,
        "scope": "Every wheel archive member except installer-updated RECORD matches "
        "package bytes; RECORD inputs preserved separately. PyPI archives verified. "
        "Not legal clearance or complete native-vendor source rebuild.",
    }
    (output / "comparison.json").write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    main(args.package, args.output)
