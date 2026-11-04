from pathlib import Path
import json,os,subprocess,time
from reference_paths import ROOT, CACHE, SHARED, GENERATOR, GADGET_LIBRARY, bundle_manifest
manifest=bundle_manifest()
from bridge import NodeBridge,ELECTRON
run=Path(json.loads((CACHE/'active.json').read_text())['run']);b=None
helper=GENERATOR
report={'phase':'page-apply','status':'running','started':time.time()}
try:
 b=NodeBridge(ROOT,run)
 pages=b.value('(()=>{const e='+ELECTRON+';return e.webContents.getAllWebContents().map(w=>({id:w.id,title:w.getTitle(),url:w.getURL()}))})()')
 selected=[p for p in pages if p['url']=='https://www.douyin.com/jingxuan']
 assert len(selected)==1,selected
 target=selected[0];report['target']=target
 program=subprocess.check_output([str(helper),'apply',str(manifest)],text=True)
 (run/'shared-apply.js').write_text(program)
 wrap=lambda js:'(()=>{const e='+ELECTRON+';const w=e.webContents.fromId('+str(target['id'])+');if(!w||w.isDestroyed()||w.getURL()!=='+json.dumps(target['url'])+')throw Error("Target changed");return w.executeJavaScript('+json.dumps(js)+');})()'
 result=b.evaluate(wrap(program));report['apply']=result
 assert result.get('ok') and result['value']['status'] in ['applied','updated'],result
 proof='''(()=>{const p=document.getElementById('injectra-douyin-proof'),r=p?.getBoundingClientRect();return {title:document.title,url:location.href,count:document.querySelectorAll('#injectra-douyin-proof').length,executed:p?.dataset.executed,text:p?.innerText,background:p?getComputedStyle(p).backgroundColor:null,rect:r?.toJSON(),viewport:[innerWidth,innerHeight]}})()'''
 report['proof']=b.value(wrap(proof))
 assert report['proof']['count']==1 and report['proof']['executed']=='1' and report['proof']['background']=='rgb(37, 23, 65)',report['proof']
 (run/'selected-page.json').write_text(json.dumps(target,ensure_ascii=False,indent=2))
 report['status']='page-apply-passed'
 print(json.dumps(report,ensure_ascii=False,indent=2),flush=True)
finally:
 if b:b.close()
 (run/('page-apply-'+str(time.time_ns())+'.json')).write_text(json.dumps(report,ensure_ascii=False,indent=2))
