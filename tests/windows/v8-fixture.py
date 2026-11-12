"""Controlled Node validation of the Windows V8 experiment; no installed apps."""
import argparse,json,pathlib,subprocess,time
import frida
p=argparse.ArgumentParser();p.add_argument('--node',required=True);p.add_argument('--output',type=pathlib.Path,required=True);args=p.parse_args()
args.output.mkdir(parents=True,exist_ok=False)
fixture=args.output/'fixture.cjs';marker=args.output/'marker.json';fixture.write_text('setInterval(()=>{},100);')
child=subprocess.Popen([args.node,str(fixture)]);session=script=None;events=[];result={'passed':False}
try:
 time.sleep(1);session=frida.attach(child.pid);script=session.create_script(pathlib.Path(__file__).with_name('v8-windows.js').read_text(),runtime='qjs');script.on('message',lambda m,d:events.append(m));script.load();result['agent']=script.exports_sync.start(args.node)
 code="process.mainModule.require('fs').writeFileSync("+json.dumps(str(marker))+",JSON.stringify({answer:6*7,pid:process.pid,platform:process.platform}));"
 script.exports_sync.execute('fixture',code)
 for _ in range(100):
  if marker.exists()or child.poll()is not None:break
  time.sleep(.1)
 if marker.exists():result['marker']=json.loads(marker.read_text());result['passed']=result['marker']=={'answer':42,'pid':child.pid,'platform':'win32'}
except Exception as e:result['error']=str(e)
finally:
 if script:
  try:result['detach']=script.exports_sync.stop()
  except Exception as e:result['detach_error']=str(e)
 if session:
  try:session.detach()
  except Exception:pass
 if child.poll()is None:child.terminate()
 child.wait();result['events']=events;(args.output/'result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))
