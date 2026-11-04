from reference_paths import ROOT, CACHE, SHARED, GENERATOR, GADGET_LIBRARY
from pathlib import Path
import json,os,time,secrets,traceback
import frida
run=Path(json.loads((CACHE/('active.json' if (CACHE/'active.json').exists() else 'latest.json')).read_text())['run']);state=json.loads((run/'state.json').read_text())
report={'phase':'node','status':'running','pid':state['pid']}
session=None;script=None;messages=[]
try:
 if state.get('phase') not in {'ready-to-resume','running'}:raise RuntimeError('Unverified bootstrap; no request sent')
 device=frida.get_device_manager().add_remote_device('127.0.0.1:'+str(state['port']),token=state['token'])
 session=device.attach(state['pid'])
 script=session.create_script((SHARED/'agent/v8.js').read_text())
 def message(m,d):
  messages.append(m)
  print('AGENT',json.dumps(m),flush=True)
 script.on('message',message);script.load()
 device.resume(state['pid']);time.sleep(1)
 setup=script.exports_sync.setup();report['setup']=setup
 if 'error' in setup:raise RuntimeError(setup['error'])
 for _ in range(60):
  health=script.exports_sync.status()
  if health['initialized']:break
  if health['setupError']:raise RuntimeError(health['setupError'])
  time.sleep(.1)
 else:raise TimeoutError('Main queue initialization not acknowledged')
 nonce=secrets.token_hex(8)
 source='(()=>{process.stdout.write("INJECTRA_NODE_PROOF:"+JSON.stringify({nonce:'+json.dumps(nonce)+',pid:process.pid,versions:process.versions,proof:2+2})+"\\n")})()'
 report['dispatch']=script.exports_sync.execute(nonce,source)
 for _ in range(100):
  text=(run/'startup.log').read_text(errors='replace')
  matches=[line.split('INJECTRA_NODE_PROOF:',1)[1] for line in text.splitlines() if 'INJECTRA_NODE_PROOF:' in line]
  replies=[]
  for line in matches:
   try:
    j=json.loads(line)
    if j.get('nonce')==nonce:replies.append(j)
   except ValueError:pass
  if replies:
   report['node']=replies[-1];assert report['node']['pid']==state['pid'] and report['node']['proof']==4
   report['status']='node-passed';break
  if any(m.get('payload',{}).get('event')=='v8-error' for m in messages):raise RuntimeError('V8 native call failed; see message')
  time.sleep(.1)
 else:raise TimeoutError('No Node-side acknowledgement; result Unknown')
except Exception as error:
 report['status']='failed';report['error']={'type':type(error).__name__,'message':str(error)}
 traceback.print_exc()
finally:
 report['messages']=messages
 if script:
  try:
   report['shutdown']=script.exports_sync.shutdown()
   for _ in range(50):
    if script.exports_sync.status()['closed'] or not script.exports_sync.status()['initialized']:break
    time.sleep(.1)
   else:raise RuntimeError('uv close not acknowledged; refusing unsafe unload')
   script.unload()
   if session:session.detach()
  except Exception as error:report['release_error']=str(error)
 (run/('node-result-'+str(time.time_ns())+'.json')).write_text(json.dumps(report,ensure_ascii=False,indent=2))
 print(json.dumps(report,ensure_ascii=False,indent=2),flush=True)
if report['status']!='node-passed':raise SystemExit(1)
