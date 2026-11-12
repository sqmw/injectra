"""Optional Windows target verification via SSH; source stays on the Mac controller.

Runs owned scheduled interactive launches, never quits baseline user processes.
Inventory/paths, remote endpoint and output roots are explicit inputs, not adapters.
"""
import argparse, base64, gzip, json, re, os, pathlib, socket, subprocess, time, urllib.request, uuid

def quoted(value):
    return "'" + str(value).replace("'", "''") + "'"

def main():
    parser=argparse.ArgumentParser(__doc__)
    parser.add_argument('--host',required=True);parser.add_argument('--user',required=True);parser.add_argument('--ssh-port',type=int,default=22)
    parser.add_argument('--plan',type=pathlib.Path,required=True);parser.add_argument('--output',type=pathlib.Path,required=True)
    parser.add_argument('--remote-root',required=True);parser.add_argument('--dom',action='store_true');parser.add_argument('--only');parser.add_argument('--timeout',type=int,default=18)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    ssh=['ssh','-o','BatchMode=yes','-o','ConnectTimeout=12','-p',str(args.ssh_port),args.user+'@'+args.host]
    def remote(script):
        script="$ProgressPreference='SilentlyContinue';$ErrorActionPreference='Stop';[Console]::OutputEncoding=[Text.Encoding]::UTF8;"+script
        packed=base64.b64encode(gzip.compress(script.encode())).decode()
        bootstrap="$b=[Convert]::FromBase64String('"+packed+"');$m=[IO.MemoryStream]::new($b,0,$b.Length);$g=[IO.Compression.GZipStream]::new($m,[IO.Compression.CompressionMode]::Decompress);$r=[IO.StreamReader]::new($g,[Text.Encoding]::UTF8);Invoke-Expression ($r.ReadToEnd())"
        command='powershell.exe -NoProfile -NonInteractive -EncodedCommand '+base64.b64encode(bootstrap.encode('utf-16le')).decode()
        if len(command)>7800:raise RuntimeError('Diagnostic command exceeds Windows command-line limit')
        result=subprocess.run(ssh+[command],capture_output=True,text=True,errors="replace",timeout=50)
        if result.returncode:raise RuntimeError(result.stderr[-1500:])
        return json.loads(result.stdout.lstrip('\ufeff'))
    for spec in json.loads(args.plan.read_text()):
        if args.only and spec['name'] not in args.only.split(','):continue
        run=args.output/(spec['name']+'-'+uuid.uuid4().hex[:8]);run.mkdir()
        report={'application':spec,'target_platform':'windows','controller_platform':'macos','route':'CDP/SSH','passed':False}
        if not spec.get('exe'):
            report['status']=spec.get('status','missing');(run/'result.json').write_text(json.dumps(report,ensure_ascii=False,indent=2));continue
        spec['exe']=str(pathlib.PureWindowsPath(spec['exe']))
        marker=uuid.uuid4().hex;root=args.remote_root+'\\'+marker;task='Injectra-'+marker;tunnel=None
        report['remote_root']=root;report['task']=task
        try:
            port=remote("$l=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$l.Start();$n=$l.LocalEndpoint.Port;$l.Stop();@{port=$n}|ConvertTo-Json -Compress")["port"]
            with socket.socket() as listener:listener.bind(('127.0.0.1',port))
            report['port']=port
            profile=root+'\\profile';out=root+'\\launch.json'
            flags=[f'--remote-debugging-port={port}','--remote-debugging-address=127.0.0.1',f'--user-data-dir="{profile}"','--no-first-run','--no-default-browser-check']+spec.get('args',[])
            env=''
            for key,value in spec.get('env',{}).items():
                if not re.fullmatch('[A-Za-z_][A-Za-z0-9_]*',key):raise ValueError('Invalid environment variable name')
                env+='$env:'+key+'='+quoted(value.format(port=port,profile=profile))+';'
            launch=(env+"$p=Start-Process -FilePath "+quoted(spec['exe'])+" -WorkingDirectory "+quoted(str(pathlib.PureWindowsPath(spec['exe']).parent))+" -ArgumentList @("+','.join(quoted(v) for v in flags)+") -PassThru;"+
                    "$stamp=0;try{$stamp=$p.StartTime.ToUniversalTime().Ticks}catch{};[IO.File]::WriteAllText("+quoted(out)+",(@{pid=$p.Id;started=$stamp;exe="+quoted(spec['exe'])+";profile="+quoted(profile)+"}|ConvertTo-Json -Compress));"+
                    "for($i=0;$i -lt 120;$i++){if(Test-Path "+quoted(root+'\\stop')+"){break};Start-Sleep -Seconds 1};$p.Refresh();$closed=$false;if(!$p.HasExited){$closed=$p.CloseMainWindow()};[IO.File]::WriteAllText("+quoted(root+'\\cleanup.json')+",(@{closed_requested=$closed}|ConvertTo-Json -Compress))")
            launch="try{"+launch+"}catch{[IO.File]::WriteAllText("+quoted(root+'\\launch-error.txt')+",[string]$_.Exception.Message)}"
            encoded=base64.b64encode(launch.encode('utf-16le')).decode()
            create=("$reserve=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,"+str(port)+");$reserve.Start();$reserve.Stop();New-Item -ItemType Directory -Force "+quoted(root)+"|Out-Null;"+
                    "$a=New-ScheduledTaskAction -Execute 'powershell.exe' -Argument '-NoProfile -EncodedCommand "+encoded+"';"+
                    "$p=New-ScheduledTaskPrincipal -UserId ([Security.Principal.WindowsIdentity]::GetCurrent().Name) -LogonType Interactive -RunLevel Limited;"+
                    "Register-ScheduledTask -TaskName "+quoted(task)+" -Action $a -Principal $p -Force|Out-Null;Start-ScheduledTask -TaskName "+quoted(task)+";"+
                    "for($i=0;$i -lt 60;$i++){if((Test-Path "+quoted(out)+") -or (Test-Path "+quoted(root+'\\launch-error.txt')+")){break};Start-Sleep -Milliseconds 250};"+
                    "if(Test-Path "+quoted(root+'\\launch-error.txt')+"){throw ([IO.File]::ReadAllText("+quoted(root+'\\launch-error.txt')+"))};if(!(Test-Path "+quoted(out)+")){throw 'Interactive launch report not created'};[IO.File]::ReadAllText("+quoted(out)+")")
            launch_result=remote(create);report['launch']=launch_result
            tunnel=subprocess.Popen(ssh[:-1]+['-o','ExitOnForwardFailure=yes','-N','-L',f'{port}:127.0.0.1:{port}',ssh[-1]],stdout=subprocess.DEVNULL,stderr=(run/'tunnel.log').open('w'))
            start=time.monotonic();pages=[];opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
            while time.monotonic()-start<args.timeout:
                try:
                    with opener.open(f'http://127.0.0.1:{port}/json/list',timeout=2) as response:pages=json.load(response)
                    pages=[p for p in pages if p.get('type')=='page' and not p.get('url','').startswith('devtools:')]
                    if pages:break
                except Exception:pass
                time.sleep(.5)
            report['pages']=[{k:p.get(k) for k in ['id','type','title','url']} for p in pages]
            if not pages:report['status']='no-cdp-page'
            else:
                page=next((p for p in pages if p.get('url','').startswith(('file:','app:','chrome:','edge:'))),pages[0]);report['selected']=page['id']
                result=subprocess.run(['node','scripts/verify-live.mjs','--no-screenshot','--port',str(port),'--target',page['id']]+(['--dom'] if args.dom else []),cwd=pathlib.Path(__file__).resolve().parents[1],capture_output=True,text=True,timeout=75)
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
