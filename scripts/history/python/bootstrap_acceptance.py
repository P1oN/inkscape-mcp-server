#!/usr/bin/env python3
"""Check bootstrap archive/path guards and cleanup on failed tool downloads."""

import argparse
import hashlib
import io
import json
import os
import subprocess
import tarfile
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import bootstrap_native as native
import migration_build_macos_package as builder
from migration_probe import require


def main(output):
    output.mkdir(parents=True, exist_ok=False)
    checks = []
    with TemporaryDirectory(prefix="imcp-bootstrap-check-") as temporary:
        root = Path(temporary)
        for scenario, name, link in (
            ("regular", "fixture/1.0/LICENSE", None),
            ("traversal", "fixture/1.0/../../outside", None),
            ("outside-link", "fixture/1.0/link", str(root / "outside")),
        ):
            archive = root / (scenario + ".tar.gz")
            destination = root / scenario
            destination.mkdir()
            with tarfile.open(archive, "w:gz") as stream:
                member = tarfile.TarInfo(name)
                if link:
                    member.type = tarfile.SYMTYPE
                    member.linkname = link
                    stream.addfile(member)
                else:
                    member.size = 7
                    stream.addfile(member, io.BytesIO(b"license"))
            refused = False
            try:
                native.extract(archive, destination, "fixture/1.0/")
            except (RuntimeError, tarfile.FilterError):
                refused = True
            require(refused == (scenario != "regular"), scenario)
            checks.append(scenario)
        for checksum, expected in ((hashlib.sha256(b"input").hexdigest(), True), ("0" * 64, False)):
            response = io.BytesIO(b"input")
            response.url = "https://ghcr.io/input"
            with patch.object(native.urllib.request.OpenerDirector, "open", return_value=response):
                refused = False
                try:
                    native.download("https://ghcr.io/input", root / checksum, checksum)
                except RuntimeError:
                    refused = True
                require(refused != expected, "checksum acceptance differs")
            checks.append("checksum-valid" if expected else "checksum-refusal")
        with patch.object(native.urllib.request.OpenerDirector, "open") as network:
            try:
                native.download("file:///secret", root / "never", "0" * 64)
            except RuntimeError:
                pass
            else:
                raise RuntimeError("non-HTTPS input accepted")
            require(not network.called, "network called for invalid scheme")
        checks.append("non-HTTPS refuses before I/O")
        inputs = root / "native"
        library = inputs / "glib/1.0/lib/libglib.dylib"
        library.parent.mkdir(parents=True)
        library.write_bytes(b"library")
        with patch.dict(os.environ, {"INKSCAPE_MCP_BUILD_NATIVE_ROOT": str(inputs)}):
            for name in (
                "/opt/homebrew/Cellar/glib/1.0/lib/libglib.dylib",
                "@@HOMEBREW_CELLAR@@/glib/1.0/lib/libglib.dylib",
                "@@HOMEBREW_PREFIX@@/opt/glib/lib/libglib.dylib",
            ):
                require(
                    builder.native_input_path(name) == library.resolve(), "native mapping differs"
                )
            try:
                builder.native_input_path("/opt/homebrew/Cellar/../../escape")
            except RuntimeError:
                pass
            else:
                raise RuntimeError("native path escaped input tree")
        checks.append("bottle mapping and escape refusal")

        source = root / "extracted-source"
        source.mkdir()
        original_cwd = Path.cwd()
        try:
            os.chdir(source)
            with patch.object(builder, "command", side_effect=RuntimeError("Git must not run")):
                require(builder.source_revision() is None, "unmarked source download needs Git")
                revision = source / "SOURCE_REVISION"
                revision.write_text("inkscape-mcp-source-v1\n" + "a" * 40 + "\n")
                require(builder.source_revision() == "a" * 40, "archive revision lost")
                revision.write_text("inkscape-mcp-source-v1\n$(touch SHOULD_NOT_EXIST)\n")
                try:
                    builder.source_revision()
                except RuntimeError:
                    pass
                else:
                    raise RuntimeError("invalid source revision accepted")
        finally:
            os.chdir(original_cwd)
        checks.append("git-free source revision and invalid metadata refusal")

        # Substitute only the downloader in an isolated copy: force a genuine
        # shell failure before installation and observe its EXIT cleanup.
        checkout = root / "checkout"
        scripts = checkout / "scripts"
        scripts.mkdir(parents=True)
        failed_download = root / "failed-download"
        failed_download.write_text("#!/bin/bash\nexit 22\n")
        failed_download.chmod(0o700)
        source = Path("scripts/bootstrap-local-package.sh").read_text()
        (scripts / "bootstrap-local-package.sh").write_text(
            source.replace("/usr/bin/curl", str(failed_download))
        )
        private = checkout / ".inkscape-mcp-local"
        private.mkdir()
        sentinel = private / "existing-user-file"
        sentinel.write_bytes(b"preserve")
        result = subprocess.run(
            ["/bin/bash", str(scripts / "bootstrap-local-package.sh"), "--fresh-tools"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        require(result.returncode != 0 and not result.stdout, "failed download succeeded")
        require(
            sentinel.read_bytes() == b"preserve" and not list(private.glob("bootstrap.*")),
            "failure cleanup changed user files or left temporary tools",
        )
        checks.append("failed download cleanup preserves existing files")
    (output / "comparison.json").write_text(
        json.dumps(
            {
                "passed": True,
                "checks": checks,
                "scope": (
                    "Synthetic guards and failed-download cleanup; "
                    "real bootstrap acceptance is separate."
                ),
            },
            indent=2,
        )
        + "\n"
    )
    print("Bootstrap guards: " + str(len(checks)) + " checks passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    main(parser.parse_args().output)
