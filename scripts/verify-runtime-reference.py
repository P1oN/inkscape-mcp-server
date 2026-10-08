#!/usr/bin/env python3
"""Validate immutable-reference evidence before protected native signature qualification."""
import json
import os
import pathlib
import runpy
import sys

verifier = runpy.run_path(str(pathlib.Path(__file__).with_name('verify-signed-distribution.py')))
require, digest = (verifier[key] for key in ('require', 'digest'))
def validate(root):
    original=json.loads((root/'inkscape-mcp-update.json').read_text())
    requested=json.loads((root/'requested-update.json').read_text())
    candidate=json.loads((root/'CANDIDATE.json').read_text())
    evidence=json.loads((root/'distribution-evidence.json').read_text())
    receipt=json.loads((root/'FINAL-ACCEPTANCE.json').read_text())
    runtime=original['runtime']
    require(runtime==requested['runtime'] and runtime['distribution_tag']==original['tag'], 'Original immutable runtime reference differs')
    require(runtime['launcher_minimum']==2 and evidence['developer_id_verified'] is True and evidence['private_glib_hardened'] is True and evidence['team_id']==os.environ.get('EXPECTED_SIGNING_TEAM'), 'Unsigned or differently signed historical runtime cannot be reused')
    require(evidence.get('runtime_reused') is False and evidence['build_info']==candidate['package_build'], 'Original runtime build evidence differs')
    require(runtime['build_id']==evidence['build_info']['build_id'] and runtime['source_revision']==candidate['source_revision'], 'Reference source/build differs')
    ticket=evidence['notarization']['runtime']
    require(ticket['status']['status']=='Accepted' and ticket['ticket_validated'] is True, 'Referenced runtime ticket missing')
    require(receipt['passed'] is True and receipt['runtime_exercised'] is True and receipt['team_id']==evidence['team_id'], 'Referenced native acceptance missing')
    for name in ('CANDIDATE.json','distribution-evidence.json','inkscape-mcp-update.json','inkscape-mcp-macos-arm64.tar.gz'):
        require(receipt['assets'][name]==digest(root/name), 'Reference receipt/final bytes differ')
    require(runtime['asset']['sha256']==digest(root/'inkscape-mcp-macos-arm64.tar.gz') and runtime['asset']['bytes']==(root/'inkscape-mcp-macos-arm64.tar.gz').stat().st_size, 'Reference runtime bytes differ')
    return runtime
if __name__=='__main__':
    try: print(json.dumps(validate(pathlib.Path(sys.argv[1]))))
    except (ValueError, KeyError, OSError, IndexError, TypeError) as error:sys.exit('Runtime reuse refused: '+str(error))
