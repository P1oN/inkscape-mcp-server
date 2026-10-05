#!/usr/bin/env python3
"""Download hash-pinned native build inputs into a fresh owned directory; no Homebrew install."""

import argparse
import hashlib
import json
import tarfile
import urllib.parse
import urllib.request
from pathlib import Path, PurePosixPath


class HTTPSRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, newurl):
        if urllib.parse.urlsplit(newurl).scheme != "https":
            raise RuntimeError("build input redirected outside HTTPS")
        redirected = super().redirect_request(request, fp, code, message, headers, newurl)
        if redirected and urllib.parse.urlsplit(newurl).netloc != "ghcr.io":
            redirected.remove_header("Authorization")
        return redirected


def download(url, destination, checksum, token=None):
    if urllib.parse.urlsplit(url).scheme != "https":
        raise RuntimeError("only HTTPS build inputs are allowed")
    headers = {"Authorization": "Bearer " + token} if token else {}
    request = urllib.request.Request(url, headers=headers)  # noqa: S310 -- HTTPS checked above
    opener = urllib.request.build_opener(HTTPSRedirect())
    with opener.open(request, timeout=60) as response:
        if urllib.parse.urlsplit(response.url).scheme != "https":
            raise RuntimeError("download redirected outside HTTPS")
        digest = hashlib.sha256()
        size = 0
        with destination.open("xb") as output:
            while chunk := response.read(1024 * 1024):
                size += len(chunk)
                if size > 128 * 1024 * 1024:
                    raise RuntimeError("native archive exceeds input cap")
                digest.update(chunk)
                output.write(chunk)
    if digest.hexdigest() != checksum:
        raise RuntimeError("native archive checksum differs: " + destination.name)


def extract(archive, destination, prefix):
    with tarfile.open(archive) as stream:
        members = stream.getmembers()
        if len(members) > 50000 or sum(row.size for row in members) > 512 * 1024 * 1024:
            raise RuntimeError("native archive exceeds extraction cap")
        names = set()
        for row in members:
            path = PurePosixPath(row.name)
            if path.is_absolute() or ".." in path.parts or row.name in names:
                raise RuntimeError("unsafe or duplicate native archive path")
            names.add(row.name)
            if row.name != prefix.rstrip("/") and not row.name.startswith(prefix):
                # Bottles may include an enclosing formula directory.
                if not (row.isdir() and row.name == prefix.split("/")[0]):
                    raise RuntimeError("unexpected native archive root: " + row.name)
            if not (row.isfile() or row.isdir() or row.issym() or row.islnk()):
                raise RuntimeError("special native archive member")
        stream.extractall(destination, members=members, filter="data")


def prepare(lock, output):
    output.mkdir(parents=True, exist_ok=False)
    metadata = json.loads(lock.read_text())
    if metadata["target"] != "macos-arm64" or len(metadata["bottles"]) != 6:
        raise RuntimeError("unexpected native input lock")
    for row in metadata["bottles"]:
        name = row["name"]
        if name not in {"glib", "dbus", "gettext", "pcre2", "json-c", "libunistring"}:
            raise RuntimeError("unexpected native formula")
        checksum = row["sha256"]
        expected = f"https://ghcr.io/v2/homebrew/core/{name}/blobs/sha256:{checksum}"
        if row["url"] != expected or len(checksum) != 64:
            raise RuntimeError("unexpected native input URL")
        query = urllib.parse.urlencode(
            {"service": "ghcr.io", "scope": f"repository:homebrew/core/{name}:pull"}
        )
        with urllib.request.urlopen("https://ghcr.io/token?" + query, timeout=30) as response:
            token = json.load(response)["token"]
        archive = output / (name + ".tar.gz")
        print("Preparing pinned native input: " + name, flush=True)
        download(row["url"], archive, checksum, token)
        extract(archive, output, name + "/" + row["version"] + "/")
        archive.unlink()
    (output / "inputs.json").write_text(json.dumps(metadata, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lock", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    prepare(args.lock, args.output)
