"""Offline checks only: parse sources, compile agents, smoke shared payload."""
from pathlib import Path
import ast,json,os,subprocess
import frida
ROOT=Path(__file__).resolve().parents[1]
base=Path(os.environ.get('INJECTRA_CACHE_DIR',Path(os.environ.get('XDG_CACHE_HOME',Path.home()/'.cache'))/'injectra'))
cache=Path(os.environ.get('INJECTRA_REFERENCE_CACHE',base/'references'))
validation=cache/'validation';validation.mkdir(parents=True,exist_ok=True)
stats=json.loads(subprocess.check_output(['node',str(ROOT/'scripts/stats.mjs'),'--json'],text=True))
for item in stats['files']:
    p=ROOT/item['path']
    if p.suffix=='.py':ast.parse(p.read_text(),filename=item['path'])
    elif p.suffix in {'.js','.mjs'}:subprocess.run(['node','--check',str(p)],check=True)
    elif p.suffix=='.sh':subprocess.run(['sh','-n',str(p)],check=True)
compiler=frida.Compiler()
for name in ['shared/electron-v8/agent/bootstrap.js','shared/electron-v8/agent/v8.js','apps/douyin/gadget-v8/agent/cocoa.js']:
    code=compiler.build(name,project_root=str(ROOT))
    (validation/(name.replace('/','-')+'.bundle.js')).write_text(code)
helper=cache/'cargo-target/debug/injectra-reference-program'
# Generate minimal offline test input outside the repository; no shipped plugin.
fixture=validation/'fixture-input';fixture.mkdir(exist_ok=True)
manifest=fixture/'manifest.json'
manifest.write_text(json.dumps({'version':1,'id':'injectra.reference.fixture','html':'content.html','js':'script.js'}))
(fixture/'content.html').write_text('<section id="injectra-reference-proof"><button data-count="0">Fixture</button></section>')
(fixture/'script.js').write_text('const p=context.root.querySelector("section");p.dataset.executed="1";const b=p.querySelector("button");b.onclick=()=>b.dataset.count="1";')
apply=subprocess.check_output([str(helper),'apply',str(manifest)],text=True)
remove=subprocess.check_output([str(helper),'remove','injectra.reference.fixture'],text=True)
programs=validation/'programs.json';programs.write_text(json.dumps({'apply':apply,'remove':remove}))
subprocess.run(['node',str(ROOT/'scripts/payload-smoke.mjs'),str(programs)],check=True)
print(f'Reference offline checks OK: {len(stats["files"])} source files, 3 compiled agents; no target application started')
