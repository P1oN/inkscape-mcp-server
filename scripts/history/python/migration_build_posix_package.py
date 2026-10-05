#!/usr/bin/env python3
"""Build a native macOS/Linux candidate package; no cross-compilation or publishing."""

import argparse
from pathlib import Path

from migration_build_macos_package import build_and_archive

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--binary", type=Path)
    args = parser.parse_args()
    build_and_archive(args.output, args.archive, args.binary)
