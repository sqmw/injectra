from pathlib import Path
import json,os,time
from reference_paths import ROOT, CACHE, SHARED, GENERATOR, GADGET_LIBRARY
from bridge import NodeBridge,ELECTRON
run=Path(json.loads((CACHE/('active.json' if (CACHE/'active.json').exists() else 'latest.json')).read_text())['run']);b=None
try:
 b=NodeBridge(ROOT,run)
 result=b.evaluate('(()=>{const e='+ELECTRON+';return {paths:{home:e.app.getPath("home"),appData:e.app.getPath("appData"),userData:e.app.getPath("userData")},pages:e.webContents.getAllWebContents().map(w=>({id:w.id,type:w.getType(),title:w.getTitle(),url:w.getURL(),loading:w.isLoading(),destroyed:w.isDestroyed()}))}})()')
 (run/('pages-'+str(time.time_ns())+'.json')).write_text(json.dumps(result,ensure_ascii=False,indent=2))
 print(json.dumps(result,ensure_ascii=False,indent=2),flush=True)
finally:
 if b:b.close()
