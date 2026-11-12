"""Optional Windows target verification via SSH; source stays on the Mac controller.

Runs owned scheduled interactive launches, never quits baseline user processes.
Inventory/paths, remote endpoint and output roots are explicit inputs, not adapters.
"""
from types import SimpleNamespace
import argparse, base64, gzip, json, re, os, pathlib, socket, subprocess, time, urllib.request, uuid

from windows_remote import run_powershell, quoted

def main():
    parser=argparse.ArgumentParser(__doc__)
    parser.add_argument('--host',required=True);parser.add_argument('--user',required=True);parser.add_argument('--ssh-port',type=int,default=22)
    parser.add_argument('--plan',type=pathlib.Path,required=True);parser.add_argument('--output',type=pathlib.Path,required=True)
    parser.add_argument('--native-python');parser.add_argument('--native-root');parser.add_argument('--native-node');parser.add_argument('--native-target-dir');
    parser.add_argument('--remote-root',required=True);parser.add_argument('--dom',action='store_true');parser.add_argument('--only');parser.add_argument('--timeout',type=int,default=18)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    control=pathlib.Path(os.environ.get('INJECTRA_CACHE_DIR',pathlib.Path.home()/'.cache/injectra'));control.mkdir(parents=True,exist_ok=True)
    ssh=['ssh','-o','ControlMaster=auto','-o','ControlPersist=300','-o','ControlPath='+str(control/'windows-test-%C'),'-o','BatchMode=yes','-o','ConnectTimeout=12','-p',str(args.ssh_port),args.user+'@'+args.host]
    def remote(script,timeout=60):return run_powershell(ssh,script,timeout)
    for spec in json.loads(args.plan.read_text()):
        if args.only and spec['name'] not in args.only.split(','):continue
        run=args.output/(spec['name']+'-'+uuid.uuid4().hex[:8]);run.mkdir()
        report={'application':spec,'target_platform':'windows','controller_platform':'windows-native' if args.native_root else 'macos','orchestrator_platform':'macos','route':'CDP/SSH','passed':False}
        if not spec.get('exe'):
            report['status']=spec.get('status','missing');(run/'result.json').write_text(json.dumps(report,ensure_ascii=False,indent=2));continue
        spec['exe']=str(pathlib.PureWindowsPath(spec['exe']))
        marker=uuid.uuid4().hex;root=args.remote_root+'\\'+marker;task='Injectra-'+marker;tunnel=None
        report['remote_root']=root;report['task']=task
        try:
            port=remote("$l=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$l.Start();$n=$l.LocalEndpoint.Port;$l.Stop();@{port=$n}|ConvertTo-Json -Compress")["port"]
            if not args.native_root:
                with socket.socket() as listener:listener.bind(('127.0.0.1',port))
            report['port']=port
            profile=root+'\\profile';out=root+'\\launch.json'
            flags=[f'--remote-debugging-port={port}','--remote-debugging-address=127.0.0.1',f'--user-data-dir="{profile}"','--no-first-run','--no-default-browser-check']+spec.get('args',[])
            if spec.get('launch_args'):flags=[v.format(port=port,profile=profile)for v in spec['launch_args']]
            protocol=spec.get('protocol','cdp')
            preferences=dict(spec.get('preferences',{}))
            if protocol=='marionette':preferences['marionette.port']=port
            preset=''
            if preferences:
                content='\n'.join('user_pref('+json.dumps(k)+','+json.dumps(v)+');'for k,v in preferences.items())
                preset='New-Item -ItemType Directory -Force '+quoted(profile)+'|Out-Null;[IO.File]::WriteAllText('+quoted(profile+'\\user.js')+','+quoted(content)+');'
            env=''
            for key,value in spec.get('env',{}).items():
                if not re.fullmatch('[A-Za-z_][A-Za-z0-9_]*',key):raise ValueError('Invalid environment variable name')
                env+='$env:'+key+'='+quoted(value.format(port=port,profile=profile))+';'
            launch=(preset+env+"$p=Start-Process -FilePath "+quoted(spec['exe'])+" -WorkingDirectory "+quoted(str(pathlib.PureWindowsPath(spec['exe']).parent))+" -ArgumentList @("+','.join(quoted(v) for v in flags)+") -PassThru;"+
                    "$stamp=0;try{$stamp=$p.StartTime.ToUniversalTime().Ticks}catch{};[IO.File]::WriteAllText("+quoted(out)+",(@{pid=$p.Id;started=$stamp;exe="+quoted(spec['exe'])+";profile="+quoted(profile)+"}|ConvertTo-Json -Compress));"+
                    "for($i=0;$i -lt 180;$i++){if(Test-Path "+quoted(root+'\\stop')+"){break};Start-Sleep -Seconds 1};$p.Refresh();$closed=$false;if(!$p.HasExited){$closed=$p.CloseMainWindow()};[IO.File]::WriteAllText("+quoted(root+'\\cleanup.json')+",(@{closed_requested=$closed}|ConvertTo-Json -Compress))")
            launch="try{"+launch+"}catch{[IO.File]::WriteAllText("+quoted(root+'\\launch-error.txt')+",[string]$_.Exception.Message)}"
            encoded=base64.b64encode(launch.encode('utf-16le')).decode()
            create=("New-Item -ItemType Directory -Force "+quoted(root)+"|Out-Null;"+
                    "$a=New-ScheduledTaskAction -Execute 'powershell.exe' -Argument '-NoProfile -EncodedCommand "+encoded+"';"+
                    "$p=New-ScheduledTaskPrincipal -UserId ([Security.Principal.WindowsIdentity]::GetCurrent().Name) -LogonType Interactive -RunLevel Limited;"+
                    "Register-ScheduledTask -TaskName "+quoted(task)+" -Action $a -Principal $p -Force|Out-Null;Start-ScheduledTask -TaskName "+quoted(task)+";"+
                    "for($i=0;$i -lt 60;$i++){if((Test-Path "+quoted(out)+") -or (Test-Path "+quoted(root+'\\launch-error.txt')+")){break};Start-Sleep -Milliseconds 250};"+
                    "if(Test-Path "+quoted(root+'\\launch-error.txt')+"){throw ([IO.File]::ReadAllText("+quoted(root+'\\launch-error.txt')+"))};if(!(Test-Path "+quoted(out)+")){throw 'Interactive launch report not created'};[IO.File]::ReadAllText("+quoted(out)+")")
            launch_result=remote(create);report['launch']=launch_result
            if not args.native_root:tunnel=subprocess.Popen(ssh[:-1]+['-o','ExitOnForwardFailure=yes','-N','-L',f'{port}:127.0.0.1:{port}',ssh[-1]],stdout=subprocess.DEVNULL,stderr=(run/'tunnel.log').open('w'))
            start=time.monotonic();pages=[];signature=None;stable_since=start;ready=False;opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
            if protocol!='cdp':
                if not args.native_root:raise ValueError('Gecko requires native Windows test client')
                pages=[{'id':protocol,'type':'probe'}];ready=True
            while protocol=='cdp' and time.monotonic()-start<args.timeout:
                try:
                    if args.native_root:
                        pages=remote("try{$q=[Net.HttpWebRequest]::Create('http://127.0.0.1:"+str(port)+"/json/list');$q.Proxy=$null;$q.Timeout=1500;$response=$q.GetResponse();$reader=[IO.StreamReader]::new($response.GetResponseStream());$text=$reader.ReadToEnd();$reader.Close();$response.Close();[string]$text}catch{'[]'}")
                    else:
                        with opener.open(f'http://127.0.0.1:{port}/json/list',timeout=2) as response:pages=json.load(response)
                    pages=[p for p in pages if p.get('type')=='page' and not p.get('url','').startswith('devtools:')]
                    current=json.dumps(sorted((p['id'],p.get('url',''))for p in pages))
                    if current!=signature:signature=current;stable_since=time.monotonic()
                    if pages and time.monotonic()-stable_since>=1 and time.monotonic()-start>=3:ready=True;break
                except Exception:pass
                time.sleep(.5)
            report['pages']=[{k:p.get(k) for k in ['id','type','title','url']} for p in pages]
            if not pages:report['status']='no-cdp-page'
            elif not ready:report['status']='page-not-stable'
            else:
                page=next((p for p in pages if p.get('url','').startswith(('file:','app:','chrome:','edge:'))),pages[0]);report['selected']=page['id']
                verify_args=['scripts/verify-live.mjs','--no-screenshot','--port',str(port),'--target',page['id']]+(['--dom'] if args.dom else [])
                if protocol!='cdp':
                    verify_args=['scripts/windows-gecko-verify.mjs','--port',str(port),'--protocol',protocol]
                    report['route']='Gecko/'+protocol;report['product_backend']=False
                executable=args.native_node
                if protocol=='win32-hook':
                    if not args.native_python:raise ValueError('native-python required')
                    verify_args=['-B','scripts/windows-frida-hook.py','--owner',out];executable=args.native_python
                    report['route']='Frida/Win32-hook';report['product_backend']=False;report['page_injection']=False
                if args.native_root:
                    if not args.native_node or not args.native_target_dir:raise ValueError('native-node and native-target-dir required')
                    native=("$env:CARGO_TARGET_DIR="+quoted(args.native_target_dir)+";$env:INJECTRA_CACHE_DIR="+quoted(args.remote_root+'\\cache')+";Set-Location "+quoted(args.native_root)+";$ErrorActionPreference='Continue';$lines=& "+quoted(executable)+" "+' '.join(quoted(v) for v in verify_args)+" 2> "+quoted(root+'\\verify.stderr')+";$code=$LASTEXITCODE;@{returncode=$code;stdout=([string]($lines -join [Environment]::NewLine));stderr=[IO.File]::ReadAllText("+quoted(root+'\\verify.stderr')+")}|ConvertTo-Json -Compress")
                    result=SimpleNamespace(**remote(native,timeout=110 if protocol!='cdp' else 60))
                else:
                    result=subprocess.run(['node']+verify_args,cwd=pathlib.Path(__file__).resolve().parents[1],capture_output=True,text=True,timeout=75)
                (run/'verify.stdout').write_text(result.stdout);(run/'verify.stderr').write_text(result.stderr)
                try:report['verification']=json.loads(result.stdout)
                except ValueError:pass
                report['passed']=result.returncode==0;report['status']='passed' if report['passed'] else 'contract-failed'
        except Exception as error:report['status']='error';report['error']=str(error)
        finally:
            if tunnel:tunnel.terminate();tunnel.wait(timeout=5)
            if 'launch' in report:
                try:
                    cleanup=("[IO.File]::WriteAllText("+quoted(root+'\\stop')+",'stop');for($i=0;$i -lt 20;$i++){if(Test-Path "+quoted(root+'\\cleanup.json')+"){break};Start-Sleep -Milliseconds 200};"+
                        "Unregister-ScheduledTask -TaskName "+quoted(task)+" -Confirm:$false -ErrorAction SilentlyContinue;"+
                        "if(Test-Path "+quoted(root+'\\cleanup.json')+"){[IO.File]::ReadAllText("+quoted(root+'\\cleanup.json')+")}else{'{\"closed_requested\":false,\"reason\":\"interactive_cleanup_unconfirmed\"}'}")
                    report['cleanup']=remote(cleanup)
                except Exception as error:report['cleanup_error']=str(error)
            (run/'result.json').write_text(json.dumps(report,ensure_ascii=False,indent=2))
            print(json.dumps({'application':spec['name'],'status':report.get('status'),'run':str(run)},ensure_ascii=False),flush=True)
if __name__=='__main__':main()
