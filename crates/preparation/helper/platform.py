"""macOS resource metadata/signing helper; mutation restricted to an owned artifact."""
import hashlib,json,os,plistlib,subprocess,sys,tempfile
from pathlib import Path

def load(path):
    with path.open('rb') as f:return plistlib.load(f)

def electron_framework(app):
    candidates=[]
    for bundle in (app/'Contents/Frameworks').glob('*.framework'):
        meta=bundle/'Resources/Info.plist'
        if not meta.is_file():continue
        info=load(meta)
        if info.get('CFBundleIdentifier')!='com.github.Electron.framework':continue
        executable=bundle/info.get('CFBundleExecutable','')
        if not executable.is_file() or not executable.resolve().is_relative_to(app):raise RuntimeError('Unsupported: invalid Electron framework executable')
        candidates.append({'bundle':str(bundle.relative_to(app)),'executable':str(executable.relative_to(app))})
    if len(candidates)!=1:raise RuntimeError('Unsupported: expected one Electron framework identity')
    return candidates[0]

def described(app):
    info=load(app/'Contents/Info.plist')
    executable=app/'Contents/MacOS'/info['CFBundleExecutable']
    if not executable.resolve().is_relative_to(app):raise RuntimeError('Executable escapes app')
    arches=subprocess.check_output(['/usr/bin/lipo','-archs',str(executable)],text=True).split()
    if 'arm64' not in arches:raise RuntimeError('Unsupported: target has no arm64 slice')
    ent=subprocess.run(['/usr/bin/codesign','-d','--entitlements',':-',str(app)],capture_output=True)
    entitlements=plistlib.loads(ent.stdout) if ent.stdout.strip() else {}
    allowed=set(); nested=[]; visited=set()
    def add(p):
        if not p.exists() and not p.is_symlink():raise RuntimeError('Missing sealed resource: '+str(p.relative_to(app)))
        if p.is_symlink() and not p.resolve().is_relative_to(app):raise RuntimeError('Unsupported: sealed symlink leaves source bundle')
        allowed.add(str(p.relative_to(app/'Contents')))
    def collect(bundle):
        resolved=bundle.resolve()
        if resolved in visited:return
        visited.add(resolved)
        if (bundle/'Contents').is_dir():base=bundle/'Contents';info_path=base/'Info.plist'
        elif bundle.suffix=='.framework':
            base=(bundle/'Versions/Current').resolve() if (bundle/'Versions/Current').exists() else bundle
            info_path=base/'Resources/Info.plist'
            for directory in [bundle,bundle/'Versions']:
                if directory.is_dir():
                    for child in directory.iterdir():
                        if child.is_symlink():add(child)
        else:base=bundle;info_path=base/'Info.plist'
        seal_path=base/'_CodeSignature/CodeResources'
        if not seal_path.is_file():raise RuntimeError('Unsupported: code bundle has no resource seal: '+str(bundle.relative_to(app)))
        local_info=load(info_path);add(info_path);add(seal_path)
        if local_info.get('CFBundleExecutable'):
            executable=base/'MacOS'/local_info['CFBundleExecutable'] if (base/'MacOS').is_dir() else base/local_info['CFBundleExecutable']
            add(executable)
        elif (base/'MacOS').is_dir():
            candidates=[p for p in (base/'MacOS').iterdir() if p.is_file() and os.access(p,os.X_OK)]
            if len(candidates)!=1:raise RuntimeError('Unsupported: ambiguous nested executable')
            add(candidates[0])
        if (base/'PkgInfo').is_file():add(base/'PkgInfo')
        for sidecar in (base/'_CodeSignature').iterdir():
            if sidecar.is_file():add(sidecar)
        seal=load(seal_path)
        for name,value in seal.get('files2',seal.get('files',{})).items():
            relative=Path(name)
            if relative.is_absolute() or '..' in relative.parts:raise RuntimeError('Unsupported: invalid sealed resource path')
            p=base/relative
            if not p.exists() and not p.is_symlink() and isinstance(value,dict) and value.get('optional'):continue
            if isinstance(value,dict) and 'cdhash' in value and p.is_dir():nested.append(str(p.relative_to(app/'Contents')));collect(p)
            else:add(p)
    collect(app)
    sealed=sorted(allowed)
    return {'info':{'CFBundleExecutable':info['CFBundleExecutable'],'CFBundleIdentifier':info.get('CFBundleIdentifier',''),'CFBundleShortVersionString':info.get('CFBundleShortVersionString',info.get('CFBundleVersion','unknown')),'ElectronAsarIntegrity':info.get('ElectronAsarIntegrity',{})},'electron_framework':electron_framework(app),'sandboxed':bool(entitlements.get('com.apple.security.app-sandbox')),'sealed':sealed,'nested':nested}

def owned(app):
    marker=app.parent/'ownership.json'
    if not marker.is_file():raise RuntimeError('Missing artifact ownership')
    value=json.loads(marker.read_text())
    if value.get('app')!=str(app) or value.get('owner')!='injectra.preparation.v1' or Path(value['source_app']).resolve()==app:raise RuntimeError('Invalid artifact ownership')
    for p in app.rglob('*'):
        if p.is_symlink() and not p.resolve().is_relative_to(app):raise RuntimeError('Symlink escapes artifact')
    return value

def sign(path,app,data_mode):
    cmd=['/usr/bin/codesign','--force','--sign','-','--timestamp=none']
    tmp=None
    if path.suffix in ('.app','.xpc') or path.parent.name=='MacOS':
        r=subprocess.run(['/usr/bin/codesign','-d','--entitlements',':-',str(path)],capture_output=True)
        ent=plistlib.loads(r.stdout) if r.stdout.strip() else {}
        if data_mode=='isolated':
            for name in ['com.apple.security.application-groups','keychain-access-groups','com.apple.application-identifier','com.apple.developer.team-identifier','com.apple.developer.associated-domains']:ent.pop(name,None)
        ent.pop('get-task-allow',None);ent.pop('com.apple.security.get-task-allow',None)
        ent['com.apple.security.cs.disable-library-validation']=True
        tmp=tempfile.NamedTemporaryFile(dir=app.parent,suffix='.plist');tmp.write(plistlib.dumps(ent));tmp.flush()
        cmd+=['--preserve-metadata=identifier,flags','--entitlements',tmp.name]
    else:cmd+=['--preserve-metadata=identifier,entitlements,flags']
    r=subprocess.run(cmd+[str(path)],capture_output=True,text=True)
    if tmp:tmp.close()
    if r.returncode:raise RuntimeError('Signing failed for '+str(path.relative_to(app))+': '+r.stderr)

def integrity_digest(app,info):
    framework=app/electron_framework(app)['executable']
    if not framework.is_file():return None
    data=bytearray(framework.read_bytes());sentinel=b'AGbevlPCksUGKNL8TSn7wGmJEuJsXb2A';positions=[];cursor=0
    while True:
        start=data.find(sentinel,cursor)
        if start<0:break
        at=start+len(sentinel)
        if at+34>len(data):raise RuntimeError('Unsupported integrity digest slot format')
        slot=bytes(data[at:at+34])
        original=load(Path(owned(app)['source_app'])/'Contents/Info.plist')
        previous=hashlib.sha256()
        for name,entry in sorted(original.get('ElectronAsarIntegrity',{}).items()):
            for value in [name,entry['algorithm'],entry['hash']]:previous.update(value.encode())
        if slot!=bytes(34) and slot!=b'\x01\x01'+previous.digest():raise RuntimeError('Unsupported integrity digest slot format or baseline')
        positions.append(at);cursor=at+34
    if not positions:return None
    if not info.get('ElectronAsarIntegrity'):return None
    h=hashlib.sha256()
    for name,entry in sorted(info.get('ElectronAsarIntegrity',{}).items()):
        for value in [name,entry['algorithm'],entry['hash']]:h.update(value.encode())
    digest=h.digest()
    for at in positions:data[at:at+34]=b'\x01\x01'+digest
    framework.write_bytes(data)
    return {'slots':positions,'hash':digest.hex(),'enforcement':True}

try:
    op=sys.argv[1];app=Path(sys.argv[2]).resolve()
    if op=='describe':result=described(app)
    else:
        meta=owned(app);mode=meta['data_mode'];info=load(app/'Contents/Info.plist')
        if op=='sign-nested':
            signed=[]
            # Only Electron helper processes need the new process signing identity.
            # Unchanged libraries and nested application resources retain their seals.
            framework_dir=app/'Contents/Frameworks'
            for p in sorted(framework_dir.glob('*.app')):
                sign(p,app,mode);signed.append(str(p.relative_to(app)))
            result={'signed':signed}
        elif op=='finish':
            update=json.load(sys.stdin)
            if mode=='isolated':info['CFBundleIdentifier']='org.injectra.development.'+meta['id'].replace('-','')
            info['CFBundleDisplayName']=info.get('CFBundleDisplayName',info.get('CFBundleName','App'))+' - Injectra'
            if update.get('asar_hash'):info.setdefault('ElectronAsarIntegrity',{})['Resources/app.asar']={'algorithm':'SHA256','hash':update['asar_hash']}
            with (app/'Contents/Info.plist').open('wb') as f:plistlib.dump(info,f)
            digest=integrity_digest(app,info)
            if digest:sign(app/electron_framework(app)['bundle'],app,mode)
            sign(app,app,mode)
            r=subprocess.run(['/usr/bin/codesign','--verify','--deep','--strict',str(app)],capture_output=True,text=True)
            if r.returncode:raise RuntimeError('Derived signature validation failed: '+r.stderr)
            result={'signature_valid':True,'digest':digest,'library_validation_exception':True}
        else:raise RuntimeError('Unknown platform operation')
    print(json.dumps({'ok':True,'value':result}))
except Exception as e:
    print(json.dumps({'ok':False,'kind':'Unsupported' if str(e).startswith('Unsupported:') else 'Prepare','message':str(e)}));sys.exit(1)
