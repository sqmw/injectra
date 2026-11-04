"""Recheck an already-owned delayed startup without rewriting its timeout result."""
from reference_paths import ROOT, CACHE, SHARED, GENERATOR, GADGET_LIBRARY
from pathlib import Path
import json,os,subprocess,time
import frida
run=Path(json.loads((CACHE/'latest.json').read_text())['run']);state=json.loads((run/'state.json').read_text())
result={'stage':'late-listener-bootstrap','original_outcome':json.loads((run/'bootstrap-result.json').read_text())['status']}
session=None;script=None
try:
 d=frida.get_device_manager().add_remote_device('127.0.0.1:'+str(state['port']),token=state['token'])
 pid=d.spawn(['re.frida.Gadget'])
 command=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip()
 assert command.startswith(state['executable']) and state['home'] in command
 session=d.attach(pid);script=session.create_script((SHARED/'agent/bootstrap.js').read_text());script.load()
 receipt=script.exports_sync.bootstrap()
 assert receipt['pid']==pid and receipt['executable']==state['executable']
 assert receipt['foundationHome']==state['home'] and receipt['afterDyld'] is None
 result.update(status='bootstrap-passed',pid=pid,receipt=receipt)
 state['launcher_pid']=state['pid'];state['pid']=pid;state['phase']='ready-to-resume'
 (run/'state.json').write_text(json.dumps(state,ensure_ascii=False,indent=2));os.chmod(run/'state.json',0o600)
 (CACHE/'active.json').write_text(json.dumps({'run':str(run),'pid':pid,'port':state['port']},indent=2))
finally:
 if script:script.unload()
 if session:session.detach()
 (run/('recovery-'+str(time.time_ns())+'.json')).write_text(json.dumps(result,ensure_ascii=False,indent=2))
 print(json.dumps(result,ensure_ascii=False,indent=2))
