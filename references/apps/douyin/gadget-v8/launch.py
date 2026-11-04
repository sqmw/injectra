"""Create only a new isolated Douyin instance; connect via authenticated Gadget."""
from reference_paths import ROOT, CACHE, SHARED, GENERATOR, GADGET_LIBRARY
from pathlib import Path
import argparse,json,os,plistlib,secrets,shutil,socket,subprocess,time,traceback
import frida
args=argparse.ArgumentParser(description=__doc__)
args.add_argument('--app',required=True,type=Path)
args.add_argument('--proxy',default='')
args.add_argument('--code-signing',choices=['optional','required'],default='required')
args.add_argument('--launcher',choices=['direct','launchservices'],default='launchservices')
args.add_argument('--startup-timeout',type=float,default=45)
a=args.parse_args();app=a.app.resolve(strict=True)
info=plistlib.loads((app/'Contents/Info.plist').read_bytes())
assert info['CFBundleIdentifier']=='com.bytedance.douyin.desktop'
exe=app/'Contents/MacOS'/info['CFBundleExecutable']
existing=[x for x in subprocess.check_output(['ps','-axo','pid=,command='],text=True).splitlines() if len(x.strip().split(None,1))==2 and x.strip().split(None,1)[1].startswith(str(exe))]
if existing:raise SystemExit('Existing Douyin instances must be normally exited with user authorization before this launcher; no process was changed')
run=CACHE/('run-'+str(time.time_ns()));run.mkdir(parents=True)
home=run/'home';(home/'Library/Application Support').mkdir(parents=True)
shutil.copy2(GADGET_LIBRARY,run/'FridaGadget.dylib')
with socket.socket() as sock:
 sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
token=secrets.token_hex(24)
config={'interaction':{'type':'listen','address':'127.0.0.1','port':port,'token':token,'on_load':'wait','on_port_conflict':'fail'},'runtime':'qjs','code_signing':a.code_signing}
(run/'FridaGadget.config').write_text(json.dumps(config,indent=2))
os.chmod(run/'FridaGadget.config',0o600)
state={'run':str(run),'home':str(home),'port':port,'token':token,'executable':str(exe),'version':info['CFBundleShortVersionString'],'phase':'prepared','code_signing':a.code_signing,'launcher':a.launcher}
log=(run/'startup.log').open('w')
command=[str(exe),'--user-data-dir='+str(home/'chromium-profile'),'--no-first-run']
if a.proxy: command+=['--proxy-server='+a.proxy]
if a.launcher=='direct':
 child=subprocess.Popen(command,env={**os.environ,'HOME':str(home),'CFFIXED_USER_HOME':str(home),'DYLD_INSERT_LIBRARIES':str(run/'FridaGadget.dylib')},stdout=log,stderr=log,start_new_session=True)
else:
 launch_command=['/usr/bin/open','-n','-a',str(app),'--env','HOME='+str(home),'--env','CFFIXED_USER_HOME='+str(home),'--env','DYLD_INSERT_LIBRARIES='+str(run/'FridaGadget.dylib'),'--stdout',str(run/'startup.log'),'--stderr',str(run/'startup.log'),'--args',*command[1:]]
 child=subprocess.Popen(launch_command,stdout=(run/'launcher.log').open('w'),stderr=subprocess.STDOUT,start_new_session=True)
state['launcher_pid']=child.pid;state['pid']=child.pid if a.launcher=='direct' else None;state['command']=command
(run/'state.json').write_text(json.dumps(state,ensure_ascii=False,indent=2));os.chmod(run/'state.json',0o600)
(CACHE/'latest.json').write_text(json.dumps({'run':str(run),'pid':state['pid'],'port':port},indent=2))
report={'pid':state['pid'],'launcher_pid':child.pid,'port':port,'phase':'loader','status':'running','startup_timeout':a.startup_timeout}
session=None;script=None
try:
 for _ in range(max(1,int(a.startup_timeout*10))):
  if child.poll() is not None and (a.launcher=='direct' or child.returncode!=0): raise RuntimeError('Test launcher/instance exited: '+str(child.returncode))
  try:
   with socket.create_connection(('127.0.0.1',port),timeout=.15): break
  except OSError:time.sleep(.1)
 else: raise TimeoutError('Gadget did not open its loopback listener')
 device=frida.get_device_manager().add_remote_device('127.0.0.1:'+str(port),token=token)
 # Gadget's spawn implementation keeps resume_on_attach=false; no OS process spawn here.
 pid=device.spawn(['re.frida.Gadget'])
 if a.launcher=='direct':assert pid==child.pid,(pid,child.pid)
 else:
  actual=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip()
  assert actual.startswith(str(exe)) and str(home/'chromium-profile') in actual,actual
  state['pid']=pid;report['pid']=pid
 session=device.attach(pid)
 script=session.create_script((SHARED/'agent/bootstrap.js').read_text());script.load()
 receipt=script.exports_sync.bootstrap();report['bootstrap']=receipt
 assert receipt['pid']==pid and receipt['executable']==str(exe)
 assert receipt['home']==str(home) and receipt['foundationHome']==str(home)
 assert receipt['afterDyld'] is None
 report['status']='bootstrap-passed';report['phase']='ready-to-resume'
 (CACHE/'latest.json').write_text(json.dumps({'run':str(run),'pid':pid,'port':port},indent=2))
 (CACHE/'active.json').write_text(json.dumps({'run':str(run),'pid':pid,'port':port},indent=2))
 state['phase']='ready-to-resume';(run/'state.json').write_text(json.dumps(state,ensure_ascii=False,indent=2));os.chmod(run/'state.json',0o600)
 print(json.dumps(report,ensure_ascii=False,indent=2),flush=True)
except Exception as error:
 report['status']='failed';report['error']={'type':type(error).__name__,'message':str(error)}
 print(json.dumps(report,ensure_ascii=False,indent=2),flush=True);traceback.print_exc()
finally:
 if script:script.unload()
 if session:session.detach()
 (run/'bootstrap-result.json').write_text(json.dumps(report,ensure_ascii=False,indent=2))
 print('RUN',run,flush=True)
if report['status']!='bootstrap-passed':raise SystemExit(1)
