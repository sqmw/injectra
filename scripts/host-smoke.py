"""Run CDP conformance against isolated, pinned CEF and NW.js fixtures."""
import functools,http.server,json,os,socket,subprocess,tempfile,threading,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
cache=Path(os.environ.get("INJECTRA_CACHE_DIR",Path(os.environ.get("XDG_CACHE_HOME",Path.home()/".cache"))/"injectra"))
entries=json.loads((ROOT/"tests/hosts/downloads.json").read_text())
binary=Path(os.environ.get("CARGO_TARGET_DIR",cache/"cargo-target"))/"debug/injectra"
class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self,*args):pass

results=[]
for name,entry in entries.items():
    run=Path(tempfile.mkdtemp(prefix=name+"-",dir=cache/"runs"))
    home=run/"home";(home/"Library/Application Support").mkdir(parents=True)
    web=run/"web";web.mkdir()
    (web/"index.html").write_text("<!doctype html><title>Injectra host fixture</title><h1>Isolated "+name+" fixture</h1>")
    server=http.server.ThreadingHTTPServer(("127.0.0.1",0),functools.partial(Quiet,directory=str(web)))
    threading.Thread(target=server.serve_forever,daemon=True).start()
    url="http://127.0.0.1:"+str(server.server_port)+"/index.html"
    with socket.socket() as s:s.bind(("127.0.0.1",0));port=s.getsockname()[1]
    distribution=cache/"host-tests"/name/entry["version"]/"distribution"
    env={**os.environ,"HOME":str(home),"CFFIXED_USER_HOME":str(home)}
    # Test commands retain the developer's tool environment and cache, not the fixture HOME.
    testenv={**os.environ,"INJECTRA_TEST_PORT":str(port)}
    child=None
    result={"host":name,"version":entry["version"],"run":str(run),"passed":False}
    try:
        if name=="cef":
            app=next(distribution.rglob("cefclient.app"))
            command=[str(app/"Contents/MacOS/cefclient"),"--url="+url,"--cache-path="+str(run/"profile"),
                "--root-cache-path="+str(run/"cache"),"--remote-debugging-port="+str(port),"--remote-debugging-address=127.0.0.1"]
        else:
            app=next(distribution.rglob("nwjs.app"))
            package=run/"app";package.mkdir()
            (package/"package.json").write_text(json.dumps({"name":"injectra-owned-fixture","main":url,
                "window":{"show":False},"node-remote":[]}))
            command=[str(app/"Contents/MacOS/nwjs"),"--user-data-dir="+str(run/"profile"),
                "--remote-debugging-port="+str(port),"--remote-debugging-address=127.0.0.1",str(package)]
        with (run/"host.log").open("w") as log:child=subprocess.Popen(command,env=env,stdout=log,stderr=log)
        for _ in range(100):
            if child.poll() is not None:raise RuntimeError("Fixture exited "+str(child.returncode))
            p=subprocess.run([str(binary),"targets","--port",str(port)],capture_output=True,text=True,timeout=6)
            if p.returncode==0:
                pages=json.loads(p.stdout);target=next((p for p in pages if p["url"]==url),None)
                if target:break
            time.sleep(.1)
        else:raise RuntimeError("No exact fixture page discovered")
        testenv["INJECTRA_TEST_TARGET"]=target["id"]
        with (run/"contract.log").open("w") as log:
            p=subprocess.run(["sh","scripts/cargo.sh","test","--locked","-p","injectra-cdp","--test","live","--","--ignored","--nocapture"],cwd=ROOT,env=testenv,stdout=log,stderr=subprocess.STDOUT)
        result.update(passed=p.returncode==0,target=target["id"])
        if p.returncode:result["failure"]=(run/"contract.log").read_text()[-2500:]
    except Exception as error:
        result["failure"]=str(error)
        if (run/"host.log").exists():result["host_log_tail"]=(run/"host.log").read_text(errors="replace")[-1800:]
    finally:
        if child and child.poll() is None:
            child.terminate()
            try:child.wait(timeout=5)
            except subprocess.TimeoutExpired:child.kill();child.wait()
        server.shutdown();server.server_close()
    (run/"result.json").write_text(json.dumps(result,indent=2))
    results.append(result);print(json.dumps(result),flush=True)
if not all(r["passed"] for r in results):raise SystemExit(1)
