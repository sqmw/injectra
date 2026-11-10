"""Explicit macOS-local Gadget launcher; never restarts an existing app or launches a mobile target."""
import argparse
import importlib.util
import json
import os
import plistlib
import secrets
import shutil
import socket
import subprocess
import time
import sys
from pathlib import Path
import frida

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"crates/frida/helper"))
from connection import deadline
BOOTSTRAP = """const env=n=>{const p=new NativeFunction(Module.getGlobalExportByName('getenv'),'pointer',['pointer'])(Memory.allocUtf8String(n));return p.isNull()?null:p.readUtf8String();};
rpc.exports={bootstrap(){const home=env('HOME');let foundationHome=null;const homeFn=Module.findGlobalExportByName('NSHomeDirectory');if(homeFn!==null){const object=new NativeFunction(homeFn,'pointer',[])();const selector=new NativeFunction(Module.getGlobalExportByName('sel_registerName'),'pointer',['pointer'])(Memory.allocUtf8String('UTF8String'));const chars=new NativeFunction(Module.getGlobalExportByName('objc_msgSend'),'pointer',['pointer','pointer'])(object,selector);foundationHome=chars.isNull()?null:chars.readUtf8String();}const unset=new NativeFunction(Module.getGlobalExportByName('unsetenv'),'int',['pointer']);unset(Memory.allocUtf8String('DYLD_INSERT_LIBRARIES'));return {pid:Process.id,executable:Process.mainModule.path,home,foundationHome,dyld:env('DYLD_INSERT_LIBRARIES')};}};"""

def launch(app, gadget, arguments, route, launcher="direct"):
    if sys.platform != "darwin":
        raise RuntimeError("This launcher only supports macOS-local .app targets")
    info=plistlib.loads((app/"Contents/Info.plist").read_bytes())
    executable=(app/"Contents/MacOS"/info["CFBundleExecutable"]).resolve()
    processes=subprocess.check_output(["/bin/ps","-axo","comm="],text=True).splitlines()
    if str(executable) in [p.strip() for p in processes]:
        raise RuntimeError("Application already running; no instance was stopped or restarted")
    base=Path(os.environ.get("INJECTRA_CACHE_DIR",Path(os.environ.get("XDG_CACHE_HOME",Path.home()/".cache"))/"injectra"))
    run=base/"frida/runs"/("launch-"+str(time.time_ns()));run.mkdir(parents=True,mode=0o700)
    home=run/"home";(home/"Library/Application Support").mkdir(parents=True)
    library=run/"FridaGadget.dylib";shutil.copy2(gadget,library)
    with socket.socket() as s:
        s.bind(("127.0.0.1",0));port=s.getsockname()[1]
    config={"interaction":{"type":"listen","address":"127.0.0.1","port":port,
        "token":secrets.token_hex(24),"on_load":"wait","on_port_conflict":"fail"},
        "runtime":"qjs","code_signing":"required"}
    (run/"FridaGadget.config").write_text(json.dumps(config));os.chmod(run/"FridaGadget.config",0o600)
    log=run/"startup.log"
    env={**os.environ,"HOME":str(home),"CFFIXED_USER_HOME":str(home),"DYLD_INSERT_LIBRARIES":str(library)}
    env.pop("ELECTRON_RUN_AS_NODE",None)
    for key,relative in (("XDG_CONFIG_HOME",".config"),("XDG_CACHE_HOME",".cache"),("XDG_DATA_HOME",".local/share")):
        env[key]=str(home/relative)
    args=list(arguments)
    if route=="electron-gadget":
        args=["--user-data-dir="+str(home/"profile"),"--no-first-run",*args]
    if launcher=="direct":
        command=[str(executable),*args]
    else:
        command=["/usr/bin/open","-n","-a",str(app),"--env","HOME="+str(home),
            "--env","CFFIXED_USER_HOME="+str(home),"--env","DYLD_INSERT_LIBRARIES="+str(library),
            "--stdout",str(log),"--stderr",str(log),"--args",*args]
    with log.open("w") as output:
        child=subprocess.Popen(command,env=env,stdout=output,stderr=output,start_new_session=True)
    state={"pid":None,"port":port,"token":config["interaction"]["token"],"executable":str(executable),"launcher_pid":child.pid,"phase":"launching"}
    statefile=run/"state.json";statefile.write_text(json.dumps(state));os.chmod(statefile,0o600)
    print(json.dumps({"run":str(run),"launcher_pid":child.pid,"phase":"launching"}),flush=True)
    for _ in range(150):
        if child.poll() is not None and (launcher=="direct" or child.returncode!=0):
            raise RuntimeError("Owned launcher exited; inspect "+str(log))
        try:
            with socket.create_connection(("127.0.0.1",port),timeout=.1):break
        except OSError:time.sleep(.1)
    else:raise TimeoutError("Gadget listener unavailable; inspect "+str(log))
    session=None;script=None
    try:
        with deadline(10):
            device=frida.get_device_manager().add_remote_device("127.0.0.1:"+str(port),token=state["token"])
            pid=device.spawn(["re.frida.Gadget"])
            state.update(pid=pid,phase="verifying")
            statefile.write_text(json.dumps(state))
            session=device.attach(pid)
            script=session.create_script(BOOTSTRAP)
            script.load();receipt=script.exports_sync.bootstrap()
            if receipt!={"pid":pid,"executable":str(executable),"home":str(home),"foundationHome":str(home),"dyld":None}:
                raise RuntimeError("Bootstrap identity or inheritance check failed")
            if launcher=="direct" and pid!=child.pid:raise RuntimeError("Unexpected process identity")
            device.resume(pid)
            state.update(phase="running")
            statefile.write_text(json.dumps(state))
    except Exception:
        state["phase"]="failed"
        statefile.write_text(json.dumps(state))
        raise
    finally:
        with deadline(3):
            if script is not None:script.unload()
            if session is not None:session.detach()
    spec=importlib.util.spec_from_file_location("register",ROOT/"scripts/frida-session.py")
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    descriptor=run/"session.json"
    return module.register(statefile,descriptor,route)

if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--app",type=Path,required=True)
    p.add_argument("--gadget",type=Path,required=True)
    p.add_argument("--argument",action="append",default=[])
    p.add_argument("--route",choices=["electron-gadget","webkit"],default="electron-gadget")
    p.add_argument("--launcher",choices=["direct","launchservices"],default="direct")
    a=p.parse_args()
    print(json.dumps(launch(a.app.resolve(),a.gadget.resolve(),a.argument,a.route,a.launcher)),flush=True)
