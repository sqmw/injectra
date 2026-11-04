from pathlib import Path
import hashlib,json,os,secrets,subprocess,time
from reference_paths import ROOT, CACHE, SHARED, GENERATOR, GADGET_LIBRARY
from bridge import NodeBridge,PageBridge,ELECTRON
run=Path(json.loads((CACHE/'active.json').read_text())['run']);target=json.loads((run/'selected-page.json').read_text())
output=CACHE/('page-verification-'+str(time.time_ns()));output.mkdir()
helper=GENERATOR
report={'checks':[],'source_run':str(run),'target':target,'revision':'reference-archive-v1','status':'running'}
b=None

def check(name,condition,evidence):
 report['checks'].append({'name':name,'passed':bool(condition),'evidence':evidence});print(('PASS ' if condition else 'FAIL ')+name,flush=True)
 if not condition:raise AssertionError(json.dumps(evidence,ensure_ascii=False))

try:
 b=NodeBridge(ROOT,run);p=PageBridge(b,target)
 report['runtime']=b.value('(()=>{const e='+ELECTRON+';return {pid:process.pid,appVersion:e.app.getVersion(),versions:process.versions,paths:{home:e.app.getPath("home"),profile:e.app.getPath("userData")}}})()')
 report['source_sha256']={name:hashlib.sha256(path.read_bytes()).hexdigest() for name,path in {'shared/bridge.py':SHARED/'bridge.py','shared/agent/v8.js':SHARED/'agent/v8.js','verify_page.py':ROOT/'verify_page.py'}.items()}
 proof=p.value('(()=>{const p=document.getElementById("injectra-douyin-proof");return {count:document.querySelectorAll("#injectra-douyin-proof").length,click:p?.querySelector("button").dataset.count,background:p?getComputedStyle(p).backgroundColor:null}})()')
 check('real click readback',proof['count']==1 and proof['click']=='1',proof)
 check('page CSS applied',proof['background']=='rgb(37, 23, 65)',proof)
 debug=b.value('(()=>{const e='+ELECTRON+';return {port:e.app.commandLine.hasSwitch("remote-debugging-port"),pipe:e.app.commandLine.hasSwitch("remote-debugging-pipe"),attached:e.webContents.fromId('+str(target['id'])+').debugger.isAttached()}})()')
 check('no CDP entry or debugger used',not any(debug.values()),debug)
 others=b.value('(()=>{const e='+ELECTRON+';return e.webContents.getAllWebContents().filter(w=>w.id!=='+str(target['id'])+').map(w=>({id:w.id,url:w.getURL()}))})()')
 second=next(x for x in others if x['url'].startswith('https://live.douyin.com/'))
 second_proof=PageBridge(b,second).value('document.querySelectorAll("#injectra-douyin-proof").length')
 check('other page untouched',second_proof==0,second_proof)
 promised=p.evaluate('new Promise(r=>setTimeout(()=>r(42),30))')
 check('page Promise awaited',promised.get('ok') and promised['value']==42,promised)
 thrown=p.evaluate('(()=>{throw new Error("controlled-page-failure")})()')
 check('page exception preserves original diagnostic',not thrown['ok'] and thrown['error']['certainty']=='Executed' and 'controlled-page-failure' in thrown['error']['message'],thrown)
 rejected=p.evaluate('Promise.reject(new Error("controlled-rejection"))')
 check('rejected Promise preserves original diagnostic',not rejected['ok'] and rejected['error']['certainty']=='Executed' and 'controlled-rejection' in rejected['error']['message'],rejected)
 refused=PageBridge(b,{'id':target['id'],'url':'https://invalid.example/'}).evaluate('globalThis.invalidTargetSideEffect=1')
 check('target URL mismatch refused before page',not refused['ok'] and refused['error']['certainty']=='NotExecuted',refused)
 key='__injectra_attempt_'+secrets.token_hex(8)
 timed=p.evaluate('(async()=>{const k='+json.dumps(key)+';globalThis[k]=(globalThis[k]??0)+1;await new Promise(r=>setTimeout(r,250));return true})()',timeout=.03)
 check('timeout classified Unknown',not timed['ok'] and timed['error']['certainty']=='Unknown',timed)
 time.sleep(.4)
 attempts=p.value('globalThis['+json.dumps(key)+']')
 check('no automatic replay',attempts==1,attempts)
 apply=subprocess.check_output([str(helper),'apply',str(ROOT/'payload/manifest.json')],text=True)
 remove=subprocess.check_output([str(helper),'remove','injectra.probe.douyin-gadget'],text=True)
 (output/'shared-apply.js').write_text(apply)
 updated=p.evaluate(apply);count=p.value('document.querySelectorAll("#injectra-douyin-proof").length')
 check('same bundle updates without duplicate',updated.get('ok') and updated['value']['status']=='updated' and count==1,updated)
 removed=p.evaluate(remove);count=p.value('document.querySelectorAll("#injectra-douyin-proof").length')
 check('remove cleans managed content',removed.get('ok') and removed['value']['status']=='removed' and count==0,removed)
 final=p.evaluate(apply)
 check('final reapply succeeds',final.get('ok') and final['value']['status']=='applied',final)
 report['status']='passed'
except Exception as error:
 report['status']='failed';report['error']={'type':type(error).__name__,'message':str(error)}
 raise
finally:
 if b:b.close()
 (output/'result.json').write_text(json.dumps(report,ensure_ascii=False,indent=2))
 (CACHE/'verification-latest.json').write_text(json.dumps({'run':str(output),'source_run':str(run),'status':report['status']},indent=2))
 print(json.dumps({'status':report['status'],'checks':len(report['checks']),'output':str(output)}))
