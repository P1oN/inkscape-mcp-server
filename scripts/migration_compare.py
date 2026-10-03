#!/usr/bin/env python3
"""Compare captured discovery contracts exactly, with no ignored metadata fields."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


def differences(expected: object, actual: object, path: str = "") -> list[dict]:
    """Return precise leaf differences; lists retain their original ordering."""
    if type(expected) is not type(actual):
        return [{"path": path, "expected": expected, "actual": actual}]
    if isinstance(expected, dict) and isinstance(actual, dict):
        output = []
        for key in sorted(expected.keys() | actual.keys()):
            if key not in expected or key not in actual:
                output.append(
                    {
                        "path": f"{path}/{key}",
                        "expected": expected.get(key),
                        "actual": actual.get(key),
                        "missing_key": True,
                    }
                )
            else:
                output.extend(differences(expected[key], actual[key], f"{path}/{key}"))
        return output
    if isinstance(expected, list) and isinstance(actual, list):
        if len(expected) != len(actual):
            return [{"path": path, "expected_length": len(expected), "actual_length": len(actual)}]
        return [
            difference
            for index, (left, right) in enumerate(zip(expected, actual, strict=True))
            for difference in differences(left, right, f"{path}/{index}")
        ]
    return [] if expected == actual else [{"path": path, "expected": expected, "actual": actual}]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    results = []
    # The frozen contract directory also holds private live-protocol fixtures.
    # Select only the exact gate/profile/description configuration filenames;
    # every field inside each discovery response is still compared unchanged.
    configuration = re.compile(
        r"live-(?:true|false)_raw-(?:true|false)_(?:core|full)_(?:full|short)\.json"
    )
    files = sorted(
        path for path in args.reference.glob("live-*.json") if configuration.fullmatch(path.name)
    )
    if len(files) != 16:
        parser.error("reference must contain all 16 gate/profile/description configurations")
    for reference in files:
        candidate = args.candidate / reference.name
        delta = differences(json.loads(reference.read_text()), json.loads(candidate.read_text()))
        results.append({"configuration": reference.stem, "passed": not delta, "differences": delta})
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(
        json.dumps({"normalization": "none", "results": results}, indent=2) + "\n"
    )
    failed = sum(not result["passed"] for result in results)
    print(f"Discovery contracts: {len(results) - failed}/{len(results)} exact matches")
    if failed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
