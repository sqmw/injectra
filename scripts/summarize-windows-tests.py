"""Render an evidence snapshot; never modifies raw runs or overall Task state."""
import argparse,json,pathlib,ntpath
p=argparse.ArgumentParser();p.add_argument('--evidence-root',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
plan=json.loads((a.evidence_root/'plan.json').read_text());reports=[]
for file in a.evidence_root.rglob('result.json'):
 try:value=json.loads(file.read_text())
 except ValueError:continue
 if isinstance(value,dict)and isinstance(value.get('application'),dict):reports.append((file,value))
normal=lambda s:ntpath.normcase(ntpath.normpath(s or ''))
bootstrap={}
for case in (a.evidence_root/'bootstrap').glob('*'):
 try:
  value=json.loads((case/'result.json').read_text());prepare=json.loads((case/'prepare.json').read_text())
 except (OSError,ValueError):continue
 name=case.name.rsplit('-',1)[0]
 if value.get('passed')and value.get('target')=='windows':bootstrap[name]=(case,value)
lines=['# Windows逐应用证据快照','','此表由独立raw run生成，不表示所有路线/版本均支持；正式状态见Task。控制端与目标平台分别记录，Hook不计作页面成功。','','| 应用 | 当前入口/阶段结果 | 控制方式 | 证据 |','| --- | --- | --- | --- |'];positive=0
for spec in plan:
 candidates=[(f,r)for f,r in reports if r['application']['name']==spec['name']and normal(r['application'].get('exe'))==normal(spec.get('exe'))]
 good=[(f,r)for f,r in candidates if r.get('passed')and r.get('page_injection')is not False and r.get('verification',{}).get('pageInjection')is not False]
 if good:
  f,r=max(good,key=lambda x:('windows' in x[1].get('controller_platform',''),x[0].stat().st_mtime));v=r.get('verification',{});mode=v.get('mode');status='JS DOM/CSS通过，HTML字符串受限'if mode=='js-dom'else'已测页面合同通过';control=r.get('controller_platform','未记录');evidence=str(f.relative_to(a.evidence_root));positive+=1
 elif spec['name']in bootstrap:
  case,value=bootstrap[spec['name']];status='pre-runtime页面更新/清理通过（实验）';control='Mac Rust Bootstrap测试控制端';evidence=str((case/'result.json').relative_to(a.evidence_root));positive+=1
 elif candidates:
  f,r=max(candidates,key=lambda x:x[0].stat().st_mtime);status={'passed':'原生函数Hook通过，页面注入未证明','no-cdp-page':'CDP未取得页面','contract-failed':'合同未完整通过，详见原始失败','error':'启动/传输异常，待核对','page-not-stable':'页面未稳定，未执行'}.get(r.get('status'),str(r.get('status')));control=r.get('controller_platform','未记录');evidence=str(f.relative_to(a.evidence_root))
 else:status=spec.get('status','待对应路线尝试');control='—';evidence='—'
 lines.append(f"| {spec['name']} | {status} | {control} | {evidence} |")
lines+=['',f'页面正向证据覆盖 {positive} 个应用名；相同应用多次run不重复计数。', '', 'pre-runtime Bootstrap、Frida Hook、安装/入口纠正和缺失项需要与主报告合读；此表不会用Hook成功冒充页面支持。']
a.output.write_text('\n'.join(lines)+'\n');print(json.dumps({'page_positive':positive,'listed':len(plan),'output':str(a.output)}))
