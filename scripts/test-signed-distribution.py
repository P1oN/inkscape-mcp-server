#!/usr/bin/env python3
"""Synthetic guard fixtures; never invoke a real GitHub publication or signing identity."""
import copy
import json
import os
import pathlib
import runpy
import subprocess
import tempfile
import io
import tarfile

verifier = runpy.run_path(str(pathlib.Path(__file__).with_name('verify-signed-distribution.py')))
assets, digest, validate = (verifier[key] for key in ('ASSETS', 'digest', 'validate'))
os.environ['EXPECTED_SIGNING_TEAM'] = 'ABCDE12345'
# These parsing guards supplement codesign cryptographic/Team verification in native CI.
signature_check = runpy.run_path(str(pathlib.Path(__file__).with_name('verify-signature-info.py')))['validate']
valid_signature = 'Authority=Developer ID Application: Fixture (ABCDE12345)\nTimestamp=Oct 8, 2026 at 12:00:00 PM\nCodeDirectory flags=0x10000(runtime)\n'
signature_check(valid_signature, True)
signature_check(valid_signature.replace('(runtime)', ''), False)
for invalid in (valid_signature.replace('Timestamp=', 'Signed Time='),
                valid_signature.replace('Timestamp=Oct 8, 2026 at 12:00:00 PM', 'Timestamp=none'),
                valid_signature.replace('Developer ID Application:', 'Apple Development:'),
                valid_signature.replace('(runtime)', '')):
    try:
        signature_check(invalid, True)
        raise AssertionError('invalid native signature display accepted')
    except ValueError:
        pass
print('signature authority, secure timestamp and hardening refusals passed')

with tempfile.TemporaryDirectory() as directory:
    root = pathlib.Path(directory)
    for name in assets:
        (root / name).write_bytes(b'synthetic fixture asset')
    candidate = dict(format=1, source_revision='a'*40, tag='v9.0.0', prerelease=True,
                     package_build=dict(build_id='fixture', revision='a'*40))
    evidence = dict(format=2, developer_id_verified=True, notarized=True,
                    private_glib_hardened=True, team_id='ABCDE12345', tag='v9.0.0',
                    build_info=candidate['package_build'], candidate_package_build=candidate['package_build'], runtime_reused=False,
                    code=[dict(path='Inkscape MCP Runtime.app/Contents/'+path, final_sha256='b'*64) for path in (
                        *('MacOS/'+name for name in ('inkscape-mcp','inkscape-mcp-launcher','inkscape-mcp-client','inkscape-mcp-supervisor','inkscape-mcp-inx','inkscape-mcp-live','dbus-daemon','gdbus')),
                        'Frameworks/context.so','Frameworks/libgio.dylib')],
                    notarization={key:dict(id='fixture-'+key, input_sha256='c'*64, log_sha256='d'*64, log=dict(jobId='fixture-'+key,status='Accepted',sha256='c'*64), status=dict(id='fixture-'+key, status='Accepted'),ticket_validated=True) for key in ('runtime','manager','dmg')},
                    dmg_sha256=digest(root/'Inkscape-MCP-Manager.dmg'), dmg_bytes=(root/'Inkscape-MCP-Manager.dmg').stat().st_size,
                    runtime_archive_sha256=digest(root/'inkscape-mcp-macos-arm64.tar.gz'))
    asset = lambda name: dict(name=name,sha256=digest(root/name),bytes=(root/name).stat().st_size)
    manifest = dict(tag='v9.0.0',prerelease=True,runtime=dict(format=1,distribution_tag='v9.0.0',source_revision='a'*40,build_id='fixture',launcher_minimum=2,asset=asset('inkscape-mcp-macos-arm64.tar.gz')),
                    instruction_asset=asset('inkscape-mcp-instructions.tar.gz'),launcher_asset=asset('inkscape-mcp-launcher.tar.gz'))
    base = {'CANDIDATE.json':candidate,'distribution-evidence.json':evidence,'inkscape-mcp-update.json':manifest}
    def reset():
        for name, value in base.items():
            (root/name).write_text(json.dumps(value))
        for name in assets:
            if name.endswith('.sha256'):
                original = name.removesuffix('.sha256')
                (root/name).write_text(digest(root/original)+'  '+original+'\n')
        receipt=dict(format=1,passed=True,runtime_exercised=True,source_revision='a'*40,team_id='ABCDE12345',checks=dict(signatures=True,package=True,installer=True,updates=True),assets={name:digest(root/name) for name in assets if name!='FINAL-ACCEPTANCE.json'})
        (root/'FINAL-ACCEPTANCE.json').write_text(json.dumps(receipt))
    reset()
    validate(root)
    for relative, value in (
        ('package/acceptance.json', dict(passed=True)),
        ('updates/report.json', dict(passed=True, runtime_exercised=True)),
        ('installer/report.json', dict(passed=True, missing_client_exercised=True,
                                      foreign_binding_refused=True, skill_conflict_preserved=True,
                                      legacy_custom_root_migration=True, component_recovery=True,
                                      custom_engine_reused=True, missing_engine_picker_repair=True)),
    ):
        path = root/'acceptance'/relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value))
    recorder = pathlib.Path(__file__).with_name('record-signed-acceptance.py')
    recorded = subprocess.run(['python3', str(recorder), str(root)], capture_output=True, text=True)
    assert recorded.returncode == 0, recorded.stderr
    validate(root)
    installer_path = root/'acceptance/installer/report.json'
    full_installer = json.loads(installer_path.read_text())
    for gate in ('missing_client_exercised', 'foreign_binding_refused',
                 'skill_conflict_preserved', 'legacy_custom_root_migration', 'component_recovery',
                 'custom_engine_reused', 'missing_engine_picker_repair'):
        incomplete = dict(full_installer, **{gate: False})
        installer_path.write_text(json.dumps(incomplete))
        receipt_before = (root/'FINAL-ACCEPTANCE.json').read_bytes()
        refused = subprocess.run(['python3', str(recorder), str(root)], capture_output=True, text=True)
        assert refused.returncode != 0 and gate in refused.stderr
        assert (root/'FINAL-ACCEPTANCE.json').read_bytes() == receipt_before
        print(gate + ': incomplete native receipt refused')
    installer_path.write_text(json.dumps(full_installer))
    reset()
    cases = [
        ('candidate mismatch','CANDIDATE.json',lambda v:v.update(package_build=dict(build_id='different'))),
        ('wrong signer','distribution-evidence.json',lambda v:v.update(team_id='ZZZZZ12345')),
        ('unsigned runtime','distribution-evidence.json',lambda v:v.update(developer_id_verified=False)),
        ('notary rejection','distribution-evidence.json',lambda v:v['notarization']['runtime']['status'].update(status='Invalid')),
        ('notary timeout','distribution-evidence.json',lambda v:v['notarization']['manager']['status'].update(status='In Progress')),
        ('notary log wrong submission','distribution-evidence.json',lambda v:v['notarization']['runtime']['log'].update(jobId='other-submission')),
        ('notary log wrong bytes','distribution-evidence.json',lambda v:v['notarization']['manager']['log'].update(sha256='e'*64)),
        ('notary log not accepted','distribution-evidence.json',lambda v:v['notarization']['dmg']['log'].update(status='Invalid')),

        ('missing ticket','distribution-evidence.json',lambda v:v['notarization']['dmg'].update(ticket_validated=False)),
        ('missing code','distribution-evidence.json',lambda v:v.update(code=[])),
        ('missing context bridge','distribution-evidence.json',lambda v:v.update(code=[row for row in v['code'] if not row['path'].endswith('context.so')])),
        ('absent distinct-runtime gate','FINAL-ACCEPTANCE.json',lambda v:v.update(runtime_exercised=False)),
        ('failed acceptance','FINAL-ACCEPTANCE.json',lambda v:v['checks'].update(installer=False)),
    ]
    for label, name, mutation in cases:
        reset(); value=json.loads((root/name).read_text());mutation(value);(root/name).write_text(json.dumps(value))
        try: validate(root)
        except ValueError: print(label+': refused')
        else: raise AssertionError(label+' accepted')
    reset(); (root/'Inkscape-MCP-Manager.dmg').write_bytes(b'changed')
    try: validate(root)
    except ValueError: print('final byte mutation: refused')
    else: raise AssertionError('Changed DMG accepted')
    (root/'Inkscape-MCP-Manager.dmg').write_bytes(b'synthetic fixture asset');reset()
    sidecar = 'Inkscape-MCP-Manager.dmg.sha256'
    (root/sidecar).write_text('0'*64+'  Inkscape-MCP-Manager.dmg\n')
    receipt = json.loads((root/'FINAL-ACCEPTANCE.json').read_text())
    receipt['assets'][sidecar] = digest(root/sidecar)
    (root/'FINAL-ACCEPTANCE.json').write_text(json.dumps(receipt))
    try: validate(root)
    except ValueError: print('receipt-bound incorrect checksum sidecar: refused')
    else: raise AssertionError('Incorrect checksum sidecar accepted')
    reset()
    (root/'requested-update.json').write_text((root/'inkscape-mcp-update.json').read_text())
    reference_verifier = runpy.run_path(str(pathlib.Path(__file__).with_name('verify-runtime-reference.py')))['validate']
    reference_verifier(root)
    altered=copy.deepcopy(evidence);altered['developer_id_verified']=False
    (root/'distribution-evidence.json').write_text(json.dumps(altered))
    try: reference_verifier(root)
    except ValueError: print('unsigned immutable runtime reuse: refused')
    else: raise AssertionError('Unsigned runtime reference accepted')
    reset()
    # Separately bind current Manager/source to an older immutable runtime.
    reused=copy.deepcopy(evidence);reused.update(runtime_reused=True,runtime_reference=manifest['runtime'],candidate_package_build=dict(build_id='new-manager',revision='b'*40))
    new_candidate=copy.deepcopy(candidate);new_candidate.update(source_revision='b'*40,package_build=reused['candidate_package_build'],runtime_reference=manifest['runtime'])
    (root/'CANDIDATE.json').write_text(json.dumps(new_candidate));(root/'distribution-evidence.json').write_text(json.dumps(reused))
    receipt=json.loads((root/'FINAL-ACCEPTANCE.json').read_text());receipt['source_revision']='b'*40;receipt['assets']={name:digest(root/name) for name in assets if name!='FINAL-ACCEPTANCE.json'};(root/'FINAL-ACCEPTANCE.json').write_text(json.dumps(receipt))
    validate(root);print('separate Manager/runtime identities and immutable reference: passed')
    reset()
    # Mock every gh invocation and verify draft/upload/publish failure boundaries.
    mock=root/'mock';mock.mkdir();log=root/'gh.log'
    gh=mock/'gh'
    gh.write_text('''#!/usr/bin/env python3
import json,os,pathlib,sys
root=pathlib.Path(os.environ['SIGNED_FIXTURE_ROOT']);args=sys.argv[1:]
with (root/'gh.log').open('a') as file:file.write(json.dumps(args)+'\\n')
mode=os.environ['SIGNED_FIXTURE_MODE']
if args[0]=='api':
 if '/releases/tags/' in args[1]:
  inventory=json.loads(os.environ['SIGNED_FIXTURE_INVENTORY'])
  entries=[dict(name=name,size=v['bytes'],digest='sha256:'+v['sha256']) for name,v in inventory.items()]
  if mode=='remote-duplicate':entries[-1]=entries[0]
  print(json.dumps(dict(draft=True,tag_name='v9.0.0',target_commitish='a'*40,assets=entries)))
 elif args[1].endswith('/releases/42'):
  inventory=json.loads(os.environ['SIGNED_FIXTURE_INVENTORY'])
  existing=[dict(name=name,size=v['bytes'],digest='sha256:'+v['sha256']) for name,v in (list(inventory.items()) if mode=='recover-complete' else list(inventory.items())[:2])]
  if mode=='recover-corrupt':existing[0]['digest']='sha256:'+'0'*64
  if mode=='recover-foreign':existing.append(dict(name='foreign',size=1,digest='sha256:'+'0'*64))
  if mode=='recover-duplicate':existing.append(existing[0])
  print(json.dumps(dict(id=42,draft=mode!='recover-published',tag_name='v9.0.0',target_commitish=('b' if mode=='recover-source' else 'a')*40,prerelease=True,assets=existing)))
 elif '/releases?' in args[-1]:
  print('[{"id":42,"draft":true,"tag_name":"v9.0.0"}]' if mode.startswith('recover-') else '[{"tag_name":"v9.0.0"}]' if mode=='draft' else '[]')
 else:print('[]')
elif args[:2]==['release','upload'] and mode=='upload-failure':sys.exit(1)
''');gh.chmod(0o755)
    for mode in ('draft','upload-failure','remote-duplicate','success'):
        log.write_text('');env=os.environ.copy();env.update(PATH=str(mock)+':'+env['PATH'],GITHUB_REPOSITORY='fixture/repository',RUNNER_TEMP=str(root),SIGNED_FIXTURE_ROOT=str(root),SIGNED_FIXTURE_MODE=mode,SIGNED_FIXTURE_INVENTORY=json.dumps(validate(root)))
        result=subprocess.run(['bash',str(pathlib.Path(__file__).with_name('publish-signed-distribution.sh')),str(root)],env=env,capture_output=True,text=True)
        calls=[json.loads(line) for line in log.read_text().splitlines()];published=any(v[:2]==['release','edit'] for v in calls)
        assert published==(mode=='success') and (result.returncode==0)==(mode=='success'),(mode,result.stderr)
        if mode=='draft':assert not any(v[:2]==['release','create'] for v in calls)
        print(mode+': publication boundary passed')
    for mode in ('recover-success', 'recover-complete', 'recover-corrupt', 'recover-foreign', 'recover-duplicate', 'recover-published', 'recover-source'):
        log.write_text('')
        env['SIGNED_FIXTURE_MODE'] = mode
        result = subprocess.run([
            'bash', str(pathlib.Path(__file__).with_name('publish-signed-distribution.sh')),
            str(root), '--recover-draft', '42',
        ], env=env, capture_output=True, text=True)
        calls = [json.loads(line) for line in log.read_text().splitlines()]
        mutations = [call for call in calls if call[0] == 'release']
        assert (result.returncode == 0) == (mode in ('recover-success', 'recover-complete')), result.stderr
        if mode in ('recover-success', 'recover-complete'):
            assert all(call[:2] != ['release', 'create'] for call in mutations)
            if mode == 'recover-success':
                upload = next(call for call in mutations if call[:2] == ['release', 'upload'])
                uploaded = {pathlib.Path(value).name for value in upload[5:]}
                assert uploaded == set(assets) - set(list(validate(root))[:2])
                assert '--clobber' not in upload
            else:
                assert all(call[:2] != ['release', 'upload'] for call in mutations)
            assert mutations[-1][:2] == ['release', 'edit']
        else:
            assert not mutations, mutations
        print(mode + ': explicit draft recovery boundary passed')

# Archive refusals must happen before any output directory is created.
with tempfile.TemporaryDirectory() as directory:
    root = pathlib.Path(directory)
    for label, names, link in (
        ('valid', ['a/file'], False),
        ('traversal', ['../file'], False),
        ('normalized duplicate', ['a/file', 'a/./file'], False),
        ('parent conflict', ['a', 'a/file'], False),
        ('symlink', ['a'], True),
    ):
        archive = root / 'fixture.tar.gz'
        target = root / label
        with tarfile.open(archive, 'w:gz') as bundle:
            for name in names:
                member = tarfile.TarInfo(name)
                if link:
                    member.type = tarfile.SYMTYPE
                    member.linkname = '/outside'
                    bundle.addfile(member)
                else:
                    member.size = 1
                    bundle.addfile(member, io.BytesIO(b'x'))
        result = subprocess.run([
            'python3', str(pathlib.Path(__file__).with_name('extract-ci-artifact.py')),
            str(archive), str(target),
        ], capture_output=True, text=True)
        assert (result.returncode == 0) == (label == 'valid'), result.stderr
        assert target.exists() == (label == 'valid')
        print(label + ': extraction boundary passed')
