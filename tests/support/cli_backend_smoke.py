"""CLI wire contract and controller-exit check; use only an owned fixture for --crash."""
import argparse,json,os,subprocess,tempfile,time
from pathlib import Path

def verify(binary,session,target,crash=False,backend="electron-gadget"):
    flags=["--backend",backend,"--session",str(session),"--target",target]
    def cli(op,*args):
        p=subprocess.run([str(binary),op,*flags,*args],capture_output=True,text=True,timeout=30)
        if p.returncode:raise RuntimeError(p.stderr)
        return json.loads(p.stdout)
    with tempfile.TemporaryDirectory(prefix="injectra-cli-contract-") as name:
        root=Path(name);ident="injectra.test.cli"+str(time.time_ns())
        manifest=root/"manifest.json"
        manifest.write_text(json.dumps({"version":1,"id":ident,"js":"run.js"}))
        script=root/"run.js"
        script.write_text("await Promise.resolve();")
        try:
            assert cli("apply","--bundle",str(manifest))["status"]=="applied"
            assert cli("apply","--bundle",str(manifest))["status"]=="updated"
            assert cli("remove","--id",ident)["status"]=="removed"
            assert cli("remove","--id",ident)["status"]=="absent"
            if crash:
                cache=Path(os.environ.get("INJECTRA_CACHE_DIR",Path(os.environ.get("XDG_CACHE_HOME",Path.home()/".cache"))/"injectra"))
                requests=cache/"frida/requests"
                before=set(requests.iterdir())
                script.write_text("await new Promise(resolve=>setTimeout(resolve,2000));")
                child=subprocess.Popen([str(binary),"apply",*flags,"--bundle",str(manifest)],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
                selected=None
                for _ in range(120):
                    for run in set(requests.iterdir())-before:
                        try:data=json.loads((run/"request.json").read_text())
                        except (OSError,ValueError):continue
                        if data.get("operation")=="evaluate" and ident in data.get("payload",{}).get("source",""):
                            selected=run;break
                    if selected:break
                    if child.poll() is not None:raise RuntimeError("CLI exited before test interruption")
                    time.sleep(.05)
                if selected is None:raise RuntimeError("No execution request found")
                time.sleep(.25)
                child.kill();child.wait(timeout=3) # This runner owns this CLI process only.
                for _ in range(160):
                    try:reply=json.loads((selected/"response.json").read_text())
                    except (OSError,ValueError):time.sleep(.05);continue
                    break
                else:raise RuntimeError("Orphaned helper did not complete cleanup")
                assert reply["ok"] and reply["value"]["status"]=="applied",reply
                assert cli("remove","--id",ident)["status"]=="removed"
        finally:
            cli("remove","--id",ident)
    return {"cli_apply_update_remove":True,"controller_exit_cleanup":bool(crash)}

if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("--binary",type=Path,required=True);p.add_argument("--session",type=Path,required=True)
    p.add_argument("--backend",choices=["electron-gadget","webkit"],default="electron-gadget");p.add_argument("--target",required=True);p.add_argument("--crash",action="store_true")
    a=p.parse_args()
    print(json.dumps(verify(a.binary,a.session,a.target,a.crash,a.backend)))
