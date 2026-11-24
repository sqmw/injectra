"""Build and run an owned WKWebView host with the formal WebKit backend."""
import json,os,plistlib,subprocess,tempfile,time,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
cache=Path(os.environ.get("INJECTRA_CACHE_DIR",Path(os.environ.get("XDG_CACHE_HOME",Path.home()/".cache"))/"injectra"))
python=Path(os.environ.get("INJECTRA_FRIDA_PYTHON",sys.executable))
gadget=Path(os.environ.get("INJECTRA_TEST_GADGET",cache/"frida/gadget/FridaGadget.dylib"))
run=Path(tempfile.mkdtemp(prefix="webkit-",dir=cache/"runs"))
app=run/"InjectraWebKitFixture.app";macos=app/"Contents/MacOS";macos.mkdir(parents=True)
(app/"Contents/Info.plist").write_bytes(plistlib.dumps({
    "CFBundleIdentifier":"io.injectra.fixture.webkit","CFBundleExecutable":"fixture",
    "CFBundleName":"InjectraWebKitFixture","CFBundleVersion":"1",
    "CFBundlePackageType":"APPL","NSHighResolutionCapable":True}))
modules=cache/"webkit-test-tools/swift-modules";modules.mkdir(parents=True,exist_ok=True)
subprocess.run(["/usr/bin/xcrun","swiftc",str(ROOT/"tests/webkit/main.swift"),
    "-o",str(macos/"fixture"),"-module-cache-path",str(modules),"-framework","AppKit","-framework","WebKit"],check=True)
pid=None
fixture_env={**os.environ,"INJECTRA_WK_FIXTURE_READY":str(run/"ready")}
try:
    binary=Path(os.environ.get("CARGO_TARGET_DIR",cache/"cargo-target"))/"debug/injectra"
    inspected=subprocess.run([str(binary),"--machine","inspect-app","--app",str(app)],env=fixture_env,capture_output=True,text=True,timeout=15)
    if inspected.returncode:raise RuntimeError("inspect-app failed: "+inspected.stdout+inspected.stderr)
    inspected_data=json.loads(inspected.stdout)["data"]
    assert inspected_data["host_family"]=="system-webview" and inspected_data["engine"]=="wkwebview"
    launch=subprocess.run([str(binary),"--machine","session","launch",
        "--app",str(app),"--gadget",str(gadget),"--route","webkit"],env=fixture_env,capture_output=True,text=True,timeout=45)
    (run/"launch.log").write_text(launch.stdout+launch.stderr)
    if launch.returncode:raise RuntimeError("Launch failed; see "+str(run))
    launched=json.loads(launch.stdout)["data"]
    session=launched["session"];pid=launched["pid"]
    descriptor=json.loads(Path(session).read_text())
    assert descriptor["schema"]==2 and descriptor["process"]["pid"]==pid
    assert descriptor["device"]["provider"]=="macos-local"
    verified=subprocess.run([str(binary),"--machine","session","verify","--session",session,"--route","webkit"],env=fixture_env,capture_output=True,text=True,timeout=25)
    if verified.returncode:raise RuntimeError("Session verify failed: "+verified.stdout+verified.stderr)
    verified_data=json.loads(verified.stdout)["data"]
    assert verified_data["valid"] is True and verified_data["process"]["pid"]==pid
    assert "token" not in verified.stdout
    ready_deadline=time.monotonic()+25
    while not (run/"ready").exists():
        if time.monotonic()>=ready_deadline:raise RuntimeError("Owned WKWebView pages did not finish navigation")
        time.sleep(.05)
    for _ in range(20):
        p=subprocess.run([str(binary),"targets","--backend","webkit","--session",session],capture_output=True,text=True,timeout=30)
        if p.returncode:raise RuntimeError(p.stderr)
        pages=json.loads(p.stdout);target=next((p for p in pages if p["title"]=="Injectra WKWebView main"),None)
        if target:break
        time.sleep(.1)
    else:raise RuntimeError("Main WKWebView not discovered: "+str(pages))
    (run/"pages.json").write_text(json.dumps(pages,indent=2))
    env={**os.environ,"INJECTRA_TEST_SESSION":session,"INJECTRA_TEST_TARGET":target["id"]}
    with (run/"contract.log").open("w") as log:
        p=subprocess.run(["sh","scripts/cargo.sh","test","--locked","-p","injectra-webkit","--test","live","--","--ignored","--nocapture"],cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
    if p.returncode==0:
        subprocess.run([str(python),"-B",str(ROOT/"tests/support/cli_backend_smoke.py"),"--backend","webkit","--binary",str(binary),"--session",session,"--target",target["id"],"--crash"],env=env,check=True)
    result={"passed":p.returncode==0,"backend":"webkit","run":str(run),"pages":len(pages)}
    (run/"result.json").write_text(json.dumps(result,indent=2));print(json.dumps(result),flush=True)
    if p.returncode:raise RuntimeError((run/"contract.log").read_text()[-3000:])
finally:
    if pid:
        try:os.kill(pid,15)
        except ProcessLookupError:pass
