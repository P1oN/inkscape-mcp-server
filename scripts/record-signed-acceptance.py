#!/usr/bin/env python3
"""Bind successful native CI receipts to exact publication assets."""
import json
import os
import pathlib
import sys
import runpy
verifier = runpy.run_path(str(pathlib.Path(__file__).with_name("verify-signed-distribution.py")))
ASSETS, digest, require = (verifier[key] for key in ("ASSETS", "digest", "require"))

root = pathlib.Path(sys.argv[1]).resolve()
candidate = json.loads((root / 'CANDIDATE.json').read_text())
for filename in ('package/acceptance.json', 'installer/report.json', 'updates/report.json', 'launcher-startup/comparison.json'):
    require(json.loads((root / 'acceptance' / filename).read_text())['passed'] is True, 'Native acceptance failed')
updates = json.loads((root / 'acceptance/updates/report.json').read_text())
require(updates['runtime_exercised'] is True, 'Distinct-runtime activation required')
installer = json.loads((root / 'acceptance/installer/report.json').read_text())
for gate in ('missing_client_exercised', 'foreign_binding_refused',
             'skill_conflict_preserved', 'legacy_custom_root_migration', 'component_recovery',
             'custom_engine_reused', 'missing_engine_picker_repair'):
    require(installer.get(gate) is True, 'Native installer gate missing: ' + gate)
startup = json.loads((root / 'acceptance/launcher-startup/comparison.json').read_text())
require(startup.get('sessions') == startup.get('completed') == 128 and startup.get('parallel') == 4,
        '128-session/four-client launcher startup required')
require(startup.get('binary_sha256') == digest(root / 'Inkscape MCP Manager.app/Contents/Helpers/inkscape-mcp-launcher'),
        'Startup tested a different launcher')
receipt = dict(format=1, passed=True, runtime_exercised=True,
               source_revision=candidate['source_revision'], team_id=os.environ['EXPECTED_SIGNING_TEAM'],
               packaging_revision=candidate['source_revision'],
               checks=dict(signatures=True, package=True, installer=True, updates=True, launcher_startup=True),
               launcher_startup={key: startup[key] for key in ('sessions', 'completed', 'parallel', 'binary_sha256')},
               assets={name: digest(root / name) for name in ASSETS if name != 'FINAL-ACCEPTANCE.json'})
(root / 'FINAL-ACCEPTANCE.json').write_text(json.dumps(receipt, indent=2) + '\n')
