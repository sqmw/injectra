"""Authenticated Gadget -> native libuv/V8 -> original Node environment."""
from pathlib import Path
import json,secrets,time
import frida

class NodeBridge:
    def __init__(self,root:Path,run:Path):
        self.run=run;self.state=json.loads((run/'state.json').read_text());self.messages=[]
        if self.state.get('phase') not in {'ready-to-resume','running'}:raise RuntimeError('Bootstrap not verified; no Node request sent')
        self.device=frida.get_device_manager().add_remote_device('127.0.0.1:'+str(self.state['port']),token=self.state['token'])
        self.session=self.device.attach(self.state['pid'])
        self.script=self.session.create_script((Path(__file__).resolve().parent/'agent/v8.js').read_text())
        self.script.on('message',lambda m,d:self.messages.append(m))
        self.script.load();self.setup=self.script.exports_sync.setup()
        if 'error' in self.setup:raise RuntimeError(self.setup['error'])
        for _ in range(60):
            s=self.script.exports_sync.status()
            if s['initialized']:break
            if s['setupError']:raise RuntimeError(s['setupError'])
            time.sleep(.1)
        else:raise TimeoutError('Main-loop bridge not ready')

    def evaluate(self,expression,timeout=10):
        request=secrets.token_hex(12)
        # Only trusted probe code enters V8. JSON envelope is emitted by real Node.
        source='void(async()=>{try{const value=await ('+expression+');process.stdout.write("INJECTRA_REPLY:"+JSON.stringify({request:'+json.dumps(request)+',pid:process.pid,ok:true,value:value===undefined?null:value})+"\\n");}catch(error){process.stdout.write("INJECTRA_REPLY:"+JSON.stringify({request:'+json.dumps(request)+',pid:process.pid,ok:false,error:{kind:"target_exception",certainty:"Executed",message:String(error),stack:String(error?.stack??"")}})+"\\n");}})()'
        accepted=self.script.exports_sync.execute(request,source)
        if not accepted['accepted']:return {'ok':False,'error':{'certainty':'NotExecuted','message':'Bridge not accepting requests'}}
        deadline=time.monotonic()+timeout
        while time.monotonic()<deadline:
            for line in (self.run/'startup.log').read_text(errors='replace').splitlines():
                if 'INJECTRA_REPLY:' not in line:continue
                try:r=json.loads(line.split('INJECTRA_REPLY:',1)[1])
                except ValueError:continue
                if r.get('request')==request:
                    assert r['pid']==self.state['pid'],'Wrong Node process'
                    return r
            errors=[m for m in self.messages if m.get('payload',{}).get('id')==request and m.get('payload',{}).get('event')=='v8-error']
            if errors:return {'ok':False,'error':{'certainty':'Unknown','message':'V8 dispatch failed','diagnostic':errors}}
            time.sleep(.05)
        return {'ok':False,'error':{'certainty':'Unknown','message':'Node acknowledgement timeout; no replay','request':request}}

    def value(self,expression,timeout=10):
        r=self.evaluate(expression,timeout)
        if not r.get('ok'):raise RuntimeError(json.dumps(r,ensure_ascii=False))
        return r['value']

    def close(self):
        self.script.exports_sync.shutdown()
        for _ in range(50):
            if self.script.exports_sync.status()['closed']:break
            time.sleep(.1)
        else:raise RuntimeError('Callback handle not closed; unsafe unload refused')
        self.script.unload();self.session.detach()

ELECTRON='''(()=>{const key=Symbol.for('injectra.probe.electron.v1');if(!globalThis[key]){
 let e;if(typeof require==='function')e=require('electron');
 else if(process.mainModule?.require)e=process.mainModule.require('electron');
 else e=process.getBuiltinModule('module').createRequire(process.execPath)('electron');
 if(!e.webContents)throw Error('Electron main-process API unavailable');
 globalThis[key]=e;}return globalThis[key];})()'''

class PageBridge:
    """Page-world envelope preserves target exceptions independently of Node."""
    def __init__(self,node,target):
        self.node=node;self.target=target
    def evaluate(self,source,timeout=10):
        t=self.target
        wrapped='(async()=>{try{const value=await ('+source+');return JSON.stringify({ok:true,value:value===undefined?null:value});}catch(error){return JSON.stringify({ok:false,error:{kind:"target_exception",certainty:"Executed",message:String(error),stack:String(error?.stack??"")}});}})()'
        expression='(async()=>{const e='+ELECTRON+';const w=e.webContents.fromId('+str(t['id'])+');if(!w||w.isDestroyed()||w.getURL()!=='+json.dumps(t['url'])+')return {ok:false,error:{kind:"invalid_target",certainty:"NotExecuted",message:"Target identity changed"}};try{return JSON.parse(await w.executeJavaScript('+json.dumps(wrapped)+'));}catch(error){return {ok:false,error:{kind:"native_page_error",certainty:"Unknown",message:String(error)}}}})()'
        reply=self.node.evaluate(expression,timeout)
        if not reply.get('ok'):
            return {'ok':False,'error':{'kind':'transport','certainty':'Unknown','diagnostic':reply.get('error')}}
        return reply['value']
    def value(self,source,timeout=10):
        result=self.evaluate(source,timeout)
        if not result.get('ok'):raise RuntimeError(json.dumps(result,ensure_ascii=False))
        return result['value']
