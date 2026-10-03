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


def main(output):
    revision = subprocess.check_output(
        ["git", "rev-parse", "--verify", "HEAD"], text=True, timeout=30
    ).strip()
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise RuntimeError("invalid source commit")
    output.parent.mkdir(parents=True, exist_ok=True)
    prefix = "inkscape-mcp-source-bootstrap/"
    with TemporaryDirectory(prefix="imcp-source-export-") as temporary:
        archive = Path(temporary) / "source.tar"
        with archive.open("wb") as stream:
            subprocess.run(
                ["git", "archive", "--format=tar", "--prefix=" + prefix, revision],
                stdout=stream,
                check=True,
                timeout=120,
            )
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
    main(parser.parse_args().output)
