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
    launch=subprocess.run([str(python),"-B",str(ROOT/"scripts/frida-launch.py"),
        "--app",str(app),"--gadget",str(gadget),"--route","webkit"],env=fixture_env,capture_output=True,text=True,timeout=35)
    (run/"launch.log").write_text(launch.stdout+launch.stderr)
    records=[json.loads(l) for l in launch.stdout.splitlines() if l.startswith("{")]
    if records:pid=records[0].get("launcher_pid")
    if launch.returncode:raise RuntimeError("Launch failed; see "+str(run))
    session=records[-1]["session"];pid=records[-1]["pid"]
    descriptor=json.loads(Path(session).read_text())
    assert descriptor["schema"]==2 and descriptor["process"]["pid"]==pid
    assert descriptor["device"]["provider"]=="macos-local"
    ready_deadline=time.monotonic()+25
    while not (run/"ready").exists():
        if time.monotonic()>=ready_deadline:raise RuntimeError("Owned WKWebView pages did not finish navigation")
        time.sleep(.05)
    binary=Path(os.environ.get("CARGO_TARGET_DIR",cache/"cargo-target"))/"debug/injectra"
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
