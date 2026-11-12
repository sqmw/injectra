"""Reconcile only recorded Windows test launches. Default is dry-run; never match by app name."""
import argparse,json,pathlib,re,uuid
from windows_remote import run_powershell,quoted
p=argparse.ArgumentParser(__doc__);p.add_argument('--evidence-root',type=pathlib.Path,required=True);p.add_argument('--host',required=True);p.add_argument('--user',required=True);p.add_argument('--ssh-port',default='22');p.add_argument('--terminate-known',action='store_true');args=p.parse_args()
baseline=json.loads((args.evidence_root/'inventory.json').read_text());protected=[x['ProcessId']for x in baseline['running']]
records={}
for file in args.evidence_root.rglob('result.json'):
 try:value=json.loads(file.read_text())
 except ValueError:continue
 if isinstance(value,dict)and value.get('remote_root')and value.get('task')and re.search(r'[a-f0-9]{32}$',value['remote_root']):records[value['remote_root']]={'root':value['remote_root'],'task':value['task']}
ssh=['ssh','-o','BatchMode=yes','-o','ControlMaster=auto','-o','ControlPersist=300','-o','ControlPath='+str(pathlib.Path.home()/'.cache/injectra/windows-test-%C'),'-p',args.ssh_port,args.user+'@'+args.host]
results=[];rows=list(records.values());run_id=uuid.uuid4().hex[:8]
for offset in range(0,len(rows),8):
 payload=json.dumps({'records':rows[offset:offset+8],'protected':protected})
 script="$data=ConvertFrom-Json -InputObject "+quoted(payload)+";$terminate="+('$true'if args.terminate_known else '$false')+";$all=Get-CimInstance Win32_Process;$out=@(foreach($r in $data.records){$file=$r.root+'\\launch.json';if(!(Test-Path $file)){continue};$v=[IO.File]::ReadAllText($file)|ConvertFrom-Json;if(!$v.started){continue};$marker=[IO.Path]::GetFileName($r.root);$expected=[IO.Path]::GetFullPath($v.exe);$candidates=@($all|Where-Object {$_.ProcessId -eq $v.pid -or ($_.CommandLine -like ('*'+$marker+'*') -and $_.ExecutablePath -and [IO.Path]::GetFullPath($_.ExecutablePath) -eq $expected)});foreach($c in $candidates){if($data.protected -contains $c.ProcessId -or !$c.CreationDate){continue};$actual=Get-Process -Id $c.ProcessId -ErrorAction SilentlyContinue;if(!$actual){continue};try{$image=$actual.Path;$created=$actual.StartTime;if(!$image -or !$created){continue};$samePath=($image -eq $expected);$stamp=$created.ToUniversalTime().Ticks;$sameTime=if($c.ProcessId -eq $v.pid){$stamp -eq $v.started}else{[Math]::Abs($stamp-$c.CreationDate.ToUniversalTime().Ticks) -lt 10 -and $stamp -ge $v.started};if($samePath -and $sameTime){$record=[pscustomobject]@{pid=$actual.Id;started=$stamp;path=$image;root=$r.root;termination_requested=$terminate};if($terminate){Stop-Process -Id $actual.Id -Force -ErrorAction SilentlyContinue};$record}}catch{continue}};if($terminate){[IO.File]::WriteAllText(($r.root+'\\stop'),'stop');Unregister-ScheduledTask -TaskName $r.task -Confirm:$false -ErrorAction SilentlyContinue}});ConvertTo-Json -InputObject $out -Compress"
 result=run_powershell(ssh,script,timeout=90);(args.evidence_root/f'cleanup-{run_id}-batch-{offset}.json').write_text(json.dumps(result,indent=2));results.extend(result if isinstance(result,list)else[result])
 print(json.dumps({'batch':offset,'matched':len(result)}),flush=True)
file=args.evidence_root/(('cleanup-requested-'if args.terminate_known else 'cleanup-dry-run-')+uuid.uuid4().hex[:8]+'.json');file.write_text(json.dumps(results,indent=2));print(json.dumps({'file':str(file),'matched':len(results),'termination_requested':args.terminate_known}))
