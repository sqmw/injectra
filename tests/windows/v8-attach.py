"""Explicit-target Windows V8 dispatch, following successful fixture validation."""
import argparse,json,ntpath,pathlib,sys,time
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[2]/'scripts'))
from windows_frida_hook_identity import identity
p=argparse.ArgumentParser();p.add_argument('--target',type=pathlib.Path,required=True);p.add_argument('--module',required=True);p.add_argument('--source',type=pathlib.Path,required=True);a=p.parse_args()
target=json.loads(a.target.read_text(encoding='utf-8-sig'));expected={'started':target['started'],'exe':ntpath.normcase(ntpath.normpath(target['exe']))};session=script=None;events=[];result={'dispatched':False,'certainty':'NotExecuted','product_backend':False}
try:
 if sys.platform!='win32' or identity(target['pid'])!=expected:raise RuntimeError('Target identity mismatch')
 import frida
 session=frida.attach(target['pid'])
 if identity(target['pid'])!=expected:raise RuntimeError('Target generation changed during attach')
 script=session.create_script(pathlib.Path(__file__).with_name('v8-windows.js').read_text(),runtime='qjs');script.on('message',lambda m,d:events.append(m));script.load();result['agent']=script.exports_sync.start(a.module);result['certainty']='Unknown';script.exports_sync.execute('bootstrap',a.source.read_text())
 for _ in range(150):
  if any(m.get('payload',{}).get('event') in ('v8-run','v8-error') or m.get('type')=='error' for m in events):break
  time.sleep(.1)
 result['dispatched']=any(m.get('payload',{}).get('event')=='v8-run' and m['payload'].get('returned') for m in events)
 if result['dispatched']:result['certainty']='Executed'
except Exception as e:result['error']=str(e)
finally:
 if script:
  try:
   result['detach']=script.exports_sync.stop()
   if result['certainty']=='Unknown' and result['detach'].get('attempted')==0:result['certainty']='NotExecuted'
  except Exception as e:result['detach_error']=str(e)
 if session:
  try:session.detach()
  except Exception as e:result['detach_error']=str(e)
 result['events']=events;print(json.dumps(result))
