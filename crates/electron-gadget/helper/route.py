"""Electron-specific Node acknowledgements and guarded main-page execution."""
import hashlib
import json
import secrets
import time
from pathlib import Path
from connection import BridgeError

ELECTRON = """(()=>{const key=Symbol.for('injectra.electron.v1');if(!globalThis[key]){
let e;if(typeof require==='function')e=require('electron');
else if(process.mainModule?.require)e=process.mainModule.require('electron');
else e=process.getBuiltinModule('module').createRequire(process.execPath)('electron');
if(!e.webContents)throw Error('Electron main-process API unavailable');globalThis[key]=e;}
return globalThis[key];})()"""
PAGES = ELECTRON + """.webContents.getAllWebContents().filter(w=>!w.isDestroyed()&&
['window','browserView','webview'].includes(w.getType())).map(w=>({
native_id:w.id,title:w.getTitle(),url:w.getURL(),routing:w.mainFrame.routingId,
process:w.mainFrame.processId}))"""

class Route:
    name = "electron-gadget"
    @staticmethod
    def validate_binding(state):
        binding = state.route("electron-gadget")
        if (binding.profile != "macos-electron-v8" or state.device.platform != "macos"
                or state.device.provider != "macos-local"):
            raise BridgeError("unsupported", "Electron host profile is not implemented for this target")
        ack = binding.options.get("acknowledgements", {})
        if (not isinstance(ack, dict) or ack.get("kind") != "controller-file"
                or not isinstance(ack.get("path"), str) or not Path(ack["path"]).is_absolute()):
            raise BridgeError("invalid_input", "This host profile requires an explicit controller-file acknowledgement source")

    def __init__(self, connection, state, root):
        self.connection, self.state, self.root = connection, state, root
        self.script = None
        self.submitted = False
        self.messages = []
        self.initialized = False

    def open(self):
        self.script = self.connection.load((self.root / "agent.js").read_text())
        self.script.on("message", lambda message, data: self.messages.append(message))
        setup = self.script.exports_sync.setup()
        if setup.get("error"):
            raise BridgeError("unsupported", setup["error"])
        deadline = time.monotonic() + 4
        while time.monotonic() < deadline:
            state = self.script.exports_sync.status()
            if state["setupError"]:
                raise BridgeError("unsupported", state["setupError"])
            if state["initialized"]:
                self.initialized = True
                return
            time.sleep(.02)
        raise BridgeError("timeout", "Main-loop initialization did not complete")

    def node(self, expression, timeout=5):
        request = secrets.token_hex(16)
        log = self.state.route(self.name).options["acknowledgements"]["path"]
        with open(log, "rb") as stream:
            stream.seek(0, 2)
            offset = stream.tell()
        source = 'void(async()=>{try{const value=await ('+expression+');process.stdout.write("INJECTRA_REPLY:"+JSON.stringify({request:'+json.dumps(request)+',pid:process.pid,ok:true,value:value===undefined?null:value})+"\\n");}catch(error){process.stdout.write("INJECTRA_REPLY:"+JSON.stringify({request:'+json.dumps(request)+',pid:process.pid,ok:false,message:String(error),stack:String(error?.stack??"")})+"\\n");}})()'
        if not self.script.exports_sync.execute(request, source)["accepted"]:
            raise BridgeError("connection", "Bridge is not accepting requests")
        self.submitted = True
        deadline = time.monotonic() + timeout
        pending = b""
        with open(log, "rb") as stream:
            stream.seek(offset)
            while time.monotonic() < deadline:
                pending += stream.read(256 * 1024)
                while b"\n" in pending:
                    line, pending = pending.split(b"\n", 1)
                    if b"INJECTRA_REPLY:" not in line:
                        continue
                    try:
                        reply = json.loads(line.split(b"INJECTRA_REPLY:", 1)[1])
                    except (ValueError, UnicodeDecodeError):
                        continue
                    if reply.get("request") != request or reply.get("pid") != self.state.process.pid:
                        continue
                    if not reply.get("ok"):
                        raise BridgeError("execution", reply.get("message", "Node exception"), "Executed")
                    return reply.get("value")
                if len(pending) > 4 * 1024 * 1024:
                    raise BridgeError("result_encoding", "Acknowledgement exceeds limit", "Unknown")
                if any(m.get("payload", {}).get("id") == request and m["payload"].get("event") == "v8-error" for m in self.messages):
                    raise BridgeError("execution", "V8 dispatch failed", "Unknown")
                time.sleep(.02)
        raise BridgeError("timeout", "Node/page result timeout; no replay", "Unknown")

    def pages(self):
        result = self.node(PAGES)
        for page in result:
            fingerprint = json.dumps([page["url"], page["routing"], page["process"]])
            page["id"] = self.state.page_id(self.name, str(page["native_id"]) + ":" + hashlib.sha256(fingerprint.encode()).hexdigest()[:20])
        return result

    def request(self, request):
        pages = self.pages()
        if request["operation"] == "targets":
            return [{k:p[k] for k in ("id", "title", "url")} for p in pages]
        payload = request["payload"]
        matches = [p for p in pages if p["id"] == payload.get("target")]
        if len(matches) != 1:
            raise BridgeError("target_not_found", "Page identity changed; run targets again")
        page = matches[0]
        source = payload["source"]
        wrapped = '(async()=>{try{const value=await ('+source+');try{return JSON.stringify({ok:true,value:value===undefined?null:value});}catch(e){return JSON.stringify({ok:false,kind:"result_encoding",message:String(e),certainty:"Executed"});}}catch(e){return JSON.stringify({ok:false,kind:"execution",message:String(e),stack:String(e?.stack??""),certainty:"Executed"});}})()'
        guard = json.dumps(page)
        expression = '(async()=>{const e='+ELECTRON+';const t='+guard+';const w=e.webContents.fromId(t.native_id);if(!w||w.isDestroyed()||w.getURL()!==t.url||w.mainFrame.routingId!==t.routing||w.mainFrame.processId!==t.process)return {ok:false,kind:"target_not_found",message:"Page changed before execution",certainty:"NotExecuted"};try{return JSON.parse(await w.executeJavaScript('+json.dumps(wrapped)+'));}catch(e){return {ok:false,kind:"connection",message:String(e),certainty:"Unknown"};}})()'
        result = self.node(expression)
        if not result.get("ok"):
            raise BridgeError(result.get("kind", "execution"), result.get("message", "Page failed"), result.get("certainty", "Unknown"))
        return result.get("value")

    def close(self):
        if not self.script:
            return
        self.script.exports_sync.shutdown()
        deadline = time.monotonic() + 4
        while time.monotonic() < deadline:
            state = self.script.exports_sync.status()
            if state["closed"] or (not state["initialized"] and state["setupError"]):
                return
            time.sleep(.02)
        raise BridgeError("connection", "Native callback closure unconfirmed; agent remains pinned", "Unknown")
