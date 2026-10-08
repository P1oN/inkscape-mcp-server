#!/usr/bin/env python3
"""Bounded CI artifact extraction; refuse links, special files and traversal before writing."""
import pathlib
import shutil
import sys
import tarfile

archive, destination = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
if destination.exists() or destination.is_symlink():
    sys.exit('Extraction output must be new')
with tarfile.open(archive, 'r:gz') as bundle:
    members = bundle.getmembers()
    if len(members) > 50000 or sum(m.size for m in members) > 4 * 1024**3:
        sys.exit('CI artifact exceeds extraction bounds')
    seen = set()
    for member in members:
        name = pathlib.PurePosixPath(member.name)
        if (name.is_absolute() or '..' in name.parts or not name.parts
                or str(name) in seen or not (member.isfile() or member.isdir())):
            sys.exit('Unsafe CI archive member')
        seen.add(str(name))
    # Refuse a file acting as another member's parent.
    files = {str(pathlib.PurePosixPath(m.name)) for m in members if m.isfile()}
    for member in members:
        if any(str(parent) in files for parent in pathlib.PurePosixPath(member.name).parents):
            sys.exit('CI archive has conflicting parents')
    destination.mkdir(mode=0o700)
    for member in members:
        path = destination.joinpath(*pathlib.PurePosixPath(member.name).parts)
        if member.isdir():
            path.mkdir(parents=True, exist_ok=True, mode=0o700)
        else:
            path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
            with bundle.extractfile(member) as source, path.open('xb') as target:
                shutil.copyfileobj(source, target, 1024 * 1024)
            path.chmod(member.mode & 0o755)
