"""WKWebView adapter: enumerate native views, execute only in the page content world."""
import hashlib,json,os,time
from pathlib import Path
import frida
from connection import BridgeError

class Route:
    name="webkit"
    def __init__(self,connection,state,root):
        self.connection,self.state,self.root=connection,state,root
        self.script=None;self.submitted=False
    def open(self):
        cache=Path(os.environ.get("INJECTRA_CACHE_DIR",Path(os.environ.get("XDG_CACHE_HOME",Path.home()/".cache"))/"injectra"))
        modules=cache/"frida/npm/node_modules"
        package=modules/"frida-objc-bridge/package.json"
        if not package.exists() or json.loads(package.read_text()).get("version")!="8.0.6":
            raise BridgeError("unsupported","Optional Objective-C bridge missing; run make setup-frida")
        link=self.root/"node_modules"
        if link.is_symlink():
            if link.resolve()!=modules.resolve():raise BridgeError("invalid_input","Unexpected helper dependency location")
        elif link.exists():raise BridgeError("invalid_input","Unexpected helper node_modules")
        else:link.symlink_to(modules,target_is_directory=True)
        bundle=self.root/"webkit-agent.bundle.js"
        if not bundle.exists():
            code=frida.Compiler().build("agent.js",project_root=str(self.root))
            temporary=self.root/("bundle-"+str(os.getpid())+".tmp")
            temporary.write_text(code);temporary.replace(bundle)
        self.script=self.connection.load(bundle.read_text())
        if not self.script.exports_sync.probe()["available"]:
            raise BridgeError("unsupported","Target has no supported WKWebView/Objective-C runtime")
    def unwrap(self,result):
        if not result.get("ok"):
            raise BridgeError(result.get("kind","connection"),result.get("message","WKWebView operation failed"),result.get("certainty","Unknown"))
        return result.get("value")
    def pages(self):
        pages=self.unwrap(self.script.exports_sync.targets())
        for page in pages:
            page["id"]=page["native_id"]+":"+hashlib.sha256(page["url"].encode()).hexdigest()[:20]
        return pages
    def request(self,request):
        pages=self.pages()
        if request["operation"]=="targets":return [{k:p[k] for k in ("id","title","url")} for p in pages]
        payload=request["payload"]
        found=[p for p in pages if p["id"]==payload.get("target")]
        if len(found)!=1:raise BridgeError("target_not_found","WKWebView target changed; run targets again")
        self.submitted=True;p=found[0]
        return self.unwrap(self.script.exports_sync.evaluate(p["native_id"],p["url"],payload["source"]))
    def close(self):
        if not self.script:return
        self.script.exports_sync.shutdown()
        deadline=time.monotonic()+4
        while time.monotonic()<deadline:
            if self.script.exports_sync.status()["closed"]:return
            time.sleep(.02)
        raise BridgeError("connection","Native WebKit callback still outstanding; unload refused","Unknown")
