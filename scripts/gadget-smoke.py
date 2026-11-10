"""Owned Electron Gadget smoke; never attaches to or exits a user's app."""
import json
import sys
import os
import subprocess
import tempfile
import time
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
cache=Path(os.environ.get("INJECTRA_CACHE_DIR",Path(os.environ.get("XDG_CACHE_HOME",Path.home()/".cache"))/"injectra"))
python=Path(os.environ.get("INJECTRA_FRIDA_PYTHON",sys.executable))
gadget=Path(os.environ["INJECTRA_TEST_GADGET"]).resolve()
electron=cache/"gadget-test-tools/node_modules/electron/dist/Electron.app/Contents/MacOS/Electron"
app=electron.parents[2]
env=os.environ.copy()
run=Path(tempfile.mkdtemp(prefix="gadget-electron-",dir=cache/"runs"))
env["INJECTRA_FIXTURE_PROFILE"]=str(run/"profile")
env.pop("INJECTRA_FIXTURE_PORT",None)
pid=None
try:
    command=[str(python),"-B",str(ROOT/"scripts/frida-launch.py"),"--app",str(app),
        "--gadget",str(gadget),"--argument",str(ROOT/"tests/electron/main.cjs")]
    launch=subprocess.run(command,env=env,capture_output=True,text=True,timeout=35)
    (run/"launch.log").write_text(launch.stdout+launch.stderr)
    records=[json.loads(l) for l in launch.stdout.splitlines() if l.startswith("{")]
    if records:pid=records[0].get("launcher_pid")
    if launch.returncode:raise RuntimeError("Owned launch failed; see "+str(run))
    session=records[-1]["session"];pid=records[-1]["pid"]
    binary=Path(env.get("CARGO_TARGET_DIR",cache/"cargo-target"))/"debug/injectra"
    for _ in range(20):
        p=subprocess.run([str(binary),"targets","--backend","electron-gadget","--session",session],env=env,capture_output=True,text=True,timeout=30)
        if p.returncode:raise RuntimeError(p.stderr)
        pages=json.loads(p.stdout)
        target=next((p for p in pages if p["title"]=="Injectra · Electron fixture"),None)
        if target:break
        time.sleep(.1)
    else:raise RuntimeError("Fixture page unavailable")
    env["INJECTRA_TEST_SESSION"]=session;env["INJECTRA_TEST_TARGET"]=target["id"]
    with (run/"test.log").open("w") as log:
        p=subprocess.run(["sh","scripts/cargo.sh","test","--locked","-p","injectra-electron-gadget","--test","live","--","--ignored","--nocapture"],cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
    if p.returncode==0:
        subprocess.run([str(python),"-B",str(ROOT/"tests/support/cli_backend_smoke.py"),"--binary",str(binary),"--session",session,"--target",target["id"],"--crash"],env=env,check=True)
    result={"passed":p.returncode==0,"route":"electron-gadget","run":str(run),"session":session,"target":target["id"]}
    (run/"result.json").write_text(json.dumps(result,indent=2))
    print(json.dumps(result))
    if p.returncode:raise RuntimeError((run/"test.log").read_text()[-3000:])
finally:
    if pid:
        # Only the exact process created by this fixture runner.
        try:os.kill(pid,15)
        except ProcessLookupError:pass
