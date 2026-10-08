#!/usr/bin/env python3
"""Read-only publication gate for CI-bound final bytes and native acceptance receipts."""
import hashlib
import json
import os
import pathlib
import re
import sys

ASSETS = (
    'Inkscape-MCP-Manager.dmg', 'inkscape-mcp-macos-arm64.tar.gz',
    'inkscape-mcp-instructions.tar.gz', 'inkscape-mcp-launcher.tar.gz',
    'inkscape-mcp-source-bootstrap.tar.gz', 'inkscape-mcp-update.json',
    'CANDIDATE.json', 'distribution-evidence.json', 'FINAL-ACCEPTANCE.json',
    'RELEASE-METADATA.json', 'Inkscape-MCP-Manager.dmg.sha256',
    'inkscape-mcp-macos-arm64.tar.gz.sha256', 'inkscape-mcp-instructions.tar.gz.sha256',
    'inkscape-mcp-launcher.tar.gz.sha256', 'inkscape-mcp-source-bootstrap.tar.gz.sha256',
)

def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    require(path.is_file() and not path.is_symlink(), 'Missing/linked final asset')
    with path.open('rb') as handle:
        result = hashlib.sha256()
        for chunk in iter(lambda: handle.read(1024 * 1024), b''):
            result.update(chunk)
        return result.hexdigest()


def validate(root):
    candidate = json.loads((root / 'CANDIDATE.json').read_text())
    evidence = json.loads((root / 'distribution-evidence.json').read_text())
    acceptance = json.loads((root / 'FINAL-ACCEPTANCE.json').read_text())
    manifest = json.loads((root / 'inkscape-mcp-update.json').read_text())
    require(candidate['format'] == 1 and re.fullmatch('[0-9a-f]{40}', candidate['source_revision']), 'Invalid candidate provenance')
    require(evidence['format'] == 2 and evidence['developer_id_verified'] is True and evidence['notarized'] is True and evidence['private_glib_hardened'] is True, 'Unsigned/unhardened distribution')
    require(re.fullmatch('[A-Z0-9]{10}', evidence['team_id']) and evidence['team_id'] == os.environ.get('EXPECTED_SIGNING_TEAM'), 'Wrong signing team')
    require(evidence['tag'] == candidate['tag'] == manifest['tag'], 'Tag differs from candidate')
    require(evidence['candidate_package_build'] == candidate['package_build'], 'Final Manager candidate differs from source build')
    require(manifest['prerelease'] == candidate['prerelease'], 'Channel mismatch')
    require(manifest['runtime']['source_revision'] == evidence['build_info']['revision'] and manifest['runtime']['build_id'] == evidence['build_info']['build_id'], 'Runtime identity mismatch')
    if evidence['runtime_reused'] is True:
        require(manifest['runtime'] == candidate['runtime_reference'] == evidence['runtime_reference'], 'Immutable runtime reference changed')
    else:
        require(manifest['runtime']['source_revision'] == candidate['source_revision'], 'Runtime source differs from candidate')
    require(manifest['runtime']['launcher_minimum'] == 2 and manifest['runtime']['format'] == 1, 'Unsupported bundle update contract')
    code_paths = {item['path'] for item in evidence['code']}
    code_root = 'Inkscape MCP Runtime.app/Contents/'
    required_code = {code_root + 'MacOS/' + name for name in (
        'inkscape-mcp', 'inkscape-mcp-launcher', 'inkscape-mcp-client',
        'inkscape-mcp-supervisor', 'inkscape-mcp-inx', 'inkscape-mcp-live',
        'dbus-daemon', 'gdbus',
    )} | {code_root + 'Frameworks/context.so'}
    require(required_code <= code_paths and len(code_paths) == len(evidence['code'])
            and any(path.startswith(code_root + 'Frameworks/') and path.endswith('.dylib') for path in code_paths),
            'Incomplete/duplicate code inventory')
    for item in evidence['code']:
        require(item['path'].startswith('Inkscape MCP Runtime.app/Contents/') and re.fullmatch('[0-9a-f]{64}', item['final_sha256']), 'Invalid code identity')
    for label in ('runtime', 'manager', 'dmg'):
        record = evidence['notarization'][label]
        require(record['status']['status'] == 'Accepted' and record['ticket_validated'] is True and record['id'] == record['status']['id'], 'Missing accepted ticket: ' + label)
        log = record['log']
        require(log['status'] == 'Accepted' and log['jobId'] == record['id']
                and log['sha256'] == record['input_sha256']
                and re.fullmatch('[0-9a-f]{64}', record['input_sha256'])
                and re.fullmatch('[0-9a-f]{64}', record['log_sha256']),
                'Accepted notarization log differs from submission: ' + label)

    require(digest(root / 'Inkscape-MCP-Manager.dmg') == evidence['dmg_sha256'], 'Final DMG changed')
    require((root / 'Inkscape-MCP-Manager.dmg').stat().st_size == evidence['dmg_bytes'], 'Final DMG length changed')
    require(digest(root / 'inkscape-mcp-macos-arm64.tar.gz') == evidence['runtime_archive_sha256'], 'Final runtime changed')
    for key in ('runtime', 'instruction_asset', 'launcher_asset'):
        asset = manifest[key]['asset'] if key == 'runtime' else manifest[key]
        require(asset['name'] in ASSETS, 'Unexpected component asset')
        path = root / asset['name']
        require(digest(path) == asset['sha256'] and path.stat().st_size == asset['bytes'], 'Component final-byte mismatch')
    require(acceptance['format'] == 1 and acceptance['passed'] is True and acceptance['runtime_exercised'] is True, 'Final distinct-runtime acceptance missing/failed')
    require(acceptance['source_revision'] == candidate['source_revision'] and acceptance['team_id'] == evidence['team_id'], 'Acceptance provenance mismatch')
    for asset in ASSETS:
        if asset == 'FINAL-ACCEPTANCE.json':
            continue
        require(acceptance['assets'][asset] == digest(root / asset), 'Acceptance bound to different final asset: ' + asset)
    require(all(acceptance['checks'].get(key) is True for key in ('signatures', 'package', 'installer', 'updates')), 'Missing final native gate')
    for name in ASSETS:
        if name.endswith('.sha256'):
            fields = (root / name).read_text().split()
            original = name.removesuffix('.sha256')
            require(fields == [digest(root / original), original], 'Incorrect checksum sidecar: ' + name)
    return {name: {'sha256': digest(root / name), 'bytes': (root / name).stat().st_size} for name in ASSETS}

if __name__ == '__main__':
    try:
        print(json.dumps(validate(pathlib.Path(sys.argv[1]).resolve()), sort_keys=True))
    except (ValueError, KeyError, OSError, IndexError, TypeError) as error:
        sys.exit('Signed distribution refused: ' + str(error))
