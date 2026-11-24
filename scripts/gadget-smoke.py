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
    binary=Path(env.get("CARGO_TARGET_DIR",cache/"cargo-target"))/"debug/injectra"
    inspected=subprocess.run([str(binary),"--machine","inspect-app","--app",str(app)],env=env,capture_output=True,text=True,timeout=15)
    if inspected.returncode:raise RuntimeError("inspect-app failed: "+inspected.stdout+inspected.stderr)
    inspected_data=json.loads(inspected.stdout)["data"]
    assert inspected_data["host_family"]=="electron"
    assert next(r for r in inspected_data["routes"] if r["route"]=="electron-gadget")["supported"] is True
    command=[str(binary),"--machine","session","launch","--app",str(app),
        "--gadget",str(gadget),"--route","electron-gadget","--arg",str(ROOT/"tests/electron/main.cjs")]
    launch=subprocess.run(command,env=env,capture_output=True,text=True,timeout=45)
    (run/"launch.log").write_text(launch.stdout+launch.stderr)
    if launch.returncode:raise RuntimeError("Owned launch failed; see "+str(run))
    launched=json.loads(launch.stdout)["data"]
    session=launched["session"];pid=launched["pid"]
    descriptor=json.loads(Path(session).read_text())
    assert descriptor["schema"]==2 and descriptor["process"]["pid"]==pid
    assert descriptor["device"]["provider"]=="macos-local"
    verified=subprocess.run([str(binary),"--machine","session","verify","--session",session,"--route","electron-gadget"],env=env,capture_output=True,text=True,timeout=25)
    if verified.returncode:raise RuntimeError("Session verify failed: "+verified.stdout+verified.stderr)
    verified_data=json.loads(verified.stdout)["data"]
    assert verified_data["valid"] is True and verified_data["process"]["pid"]==pid
    assert "token" not in verified.stdout
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
