from pathlib import Path
import json,os,time
from reference_paths import ROOT, CACHE, SHARED, GENERATOR, GADGET_LIBRARY
from bridge import NodeBridge,ELECTRON
run=Path(json.loads((CACHE/'active.json').read_text())['run']);image_path=run/('panel-'+str(time.time_ns())+'.png');target=json.loads((run/'selected-page.json').read_text());b=None
try:
 b=NodeBridge(ROOT,run)
 probe='(()=>{const p=document.getElementById("injectra-douyin-proof");if(!p)throw Error("Panel absent");return {text:p.innerText,rect:p.getBoundingClientRect().toJSON(),button:p.querySelector("button").textContent,count:p.querySelector("button").dataset.count??"0",viewport:[innerWidth,innerHeight]}})()'
 expr='(async()=>{const e='+ELECTRON+';const w=e.webContents.fromId('+str(target['id'])+');if(!w||w.getURL()!=='+json.dumps(target['url'])+')throw Error("Target changed");const proof=await w.executeJavaScript('+json.dumps(probe)+');const r=proof.rect;const rect={x:Math.max(0,Math.floor(r.x)-4),y:Math.max(0,Math.floor(r.y)-4),width:Math.ceil(r.width)+8,height:Math.ceil(r.height)+8};const image=await w.capturePage(rect);process.getBuiltinModule("fs").writeFileSync('+json.dumps(str(image_path))+',image.toPNG());return proof;})()'
 result=b.evaluate(expr)
 result['screenshot']=str(image_path)
 (run/('capture-'+str(time.time_ns())+'.json')).write_text(json.dumps(result,ensure_ascii=False,indent=2));print(json.dumps(result,ensure_ascii=False,indent=2))
finally:
 if b:b.close()
