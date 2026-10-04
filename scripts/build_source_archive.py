#!/usr/bin/env python3
"""Export committed sources plus revision metadata, excluding local settings/build products."""

import argparse
import gzip
import io
import re
import shutil
import subprocess
import tarfile
from pathlib import Path
from tempfile import TemporaryDirectory


def main(output, working_tree=False):
    revision = subprocess.check_output(
        ["git", "rev-parse", "--verify", "HEAD"], text=True, timeout=30
    ).strip()
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise RuntimeError("invalid source commit")
    output.parent.mkdir(parents=True, exist_ok=True)
    prefix = "inkscape-mcp-source-bootstrap/"
    with TemporaryDirectory(prefix="imcp-source-export-") as temporary:
        archive = Path(temporary) / "source.tar"
        if working_tree:
            tracked = (
                subprocess.check_output(["git", "ls-files", "-z"], timeout=30).decode().split("\0")
            )
            additions = (
                subprocess.check_output(
                    [
                        "git",
                        "ls-files",
                        "--others",
                        "--exclude-standard",
                        "-z",
                        "--",
                        "scripts",
                        "skills",
                        "docs",
                        "rust/src",
                        "rust/build.rs",
                        "uninstall.sh",
                    ],
                    timeout=30,
                )
                .decode()
                .split("\0")
            )
            with tarfile.open(archive, "w") as stream:
                for name in sorted(set(tracked + additions) - {""}):
                    path = Path(name)
                    if path.is_symlink():
                        raise RuntimeError("working-tree source export refuses symlinks")
                    if path.is_file():
                        stream.add(path, arcname=prefix + name, recursive=False)
                content = b"uncommitted-working-tree\n"
                member = tarfile.TarInfo(prefix + "SOURCE_STATE")
                member.size = len(content)
                stream.addfile(member, io.BytesIO(content))
        else:
            with archive.open("wb") as stream:
                subprocess.run(
                    ["git", "archive", "--format=tar", "--prefix=" + prefix, revision],
                    stdout=stream,
                    check=True,
                    timeout=120,
                )
        if not working_tree:
            with tarfile.open(archive, "a") as stream:
                name = prefix + "SOURCE_REVISION"
                if name in stream.getnames():
                    raise RuntimeError("source revision metadata already tracked")
                content = ("inkscape-mcp-source-v1\n" + revision + "\n").encode()
                member = tarfile.TarInfo(name)
                member.size = len(content)
                member.mode = 0o644
                stream.addfile(member, io.BytesIO(content))
        with output.open("xb") as destination:
            with gzip.GzipFile(fileobj=destination, mode="wb", mtime=0, filename="") as compressed:
                with archive.open("rb") as stream:
                    shutil.copyfileobj(stream, compressed)
    print(str(output))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--working-tree",
        action="store_true",
        help="Explicit local acceptance snapshot including unpublished project changes",
    )
    args = parser.parse_args()
    main(args.output, args.working_tree)
