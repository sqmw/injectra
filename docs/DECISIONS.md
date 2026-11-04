# 决策记录

<a id="d-005"></a>
## D-005 · 采纳抖音非 CDP 阶段性实测结论

```tracking-decision
{
  "id": "D-005",
  "date": "2026-11-04",
  "status": "accepted",
  "conclusion": "按用户确认，将独立探针622fe2e的抖音8.7.0非CDP真实页面注入成功采纳为阶段性成果：Gadget启动接入、原V8/Node及Electron页面API可复用共享Web程序。main本轮只接收结论和脱敏证据，产品仍只有CDP后端；不将探针整体迁入产品或承诺所有目标兼容。",
  "trigger": "用户明确确认“这里是成功了”，要求整理抖音结果提交main并记录阶段性成功。",
  "context": "T-012@v1；main基线0e32671；Probe代码源622fe2e94e10164a083c36e34ff1ae3fdfedbc99。原7.9直接attach失败、8.7成功、版本漂移和后续复验失败分别保留。",
  "affects": ["非CDP接入路线证据", "验证导航", "后续Web后端实现参考"],
  "alternatives": [
    "将外部attach拒绝当作所有原生路线失败：启动期Gadget已有成功证据",
    "把Gadget通信或Node回执当页面成功：必须有真实页面及实际效果",
    "整体合入Probe作为生产后端：缺少产品级资源、ABI及跨目标验收，本轮不实施",
    "合并7.9失败与8.7成功统计或改写旧run：不是同版本对照，也不重算旧CDP普查"
  ],
  "commit": "本条所在main提交",
  "supersedes": []
}
```

这是用户授权后的新结论采纳事件，原探针archive终态及提交不改写；参考代码仍保留在独立仓库。D-003当前产品责任和D-004执行域边界继续生效。正式后端若后续实施，在main按契约重新实现与验证；本次文档提交可独立撤销，不改变运行中的应用或缓存数据。实测边界见[抖音Gadget报告](compatibility/douyin-gadget.md)。

<a id="d-004"></a>
## D-004 · 长期注入基础与执行域隔离

```tracking-decision
{
  "id": "D-004",
  "date": "2026-11-03",
  "status": "accepted",
  "conclusion": "长期扩展边界现在建立：公共core去Web化，Web/native分域，application注入后端，系统与宿主分离；本轮迁移现有CDP并验证原生契约，不实现真实原生后端。",
  "trigger": "用户否定以暂时目标推迟架构边界，选定骨架与契约验证并明确批准实施完整计划。",
  "context": "T-008@v1；起始990967b；无冻结基线；D-003当前runtime产品边界继续有效。",
  "affects": [
    "core/Web/native/application契约",
    "适配器和宿主依赖",
    "架构与验证入口"
  ],
  "alternatives": [
    "等原生后端到来再消除核心Web假设：用户明确不接受",
    "所有后端共用万能evaluate或内存方法：执行语义不相同",
    "本轮实现真实Frida/内存注入：超出选定范围"
  ],
  "commit": "本条所在提交",
  "supersedes": []
}
```

术语映射：原Engine/Session/Target迁为WebBackend/WebSession/WebTarget；原payload职责归入web；通用目标身份为TargetRef。当前CLI与manifest v1保持兼容；内部Rust API允许迁移。分步提交回退；不覆盖MVP或已有实测证据。

<a id="d-003"></a>
## D-003 · runtime 注入工具定位与责任边界

```tracking-decision
{
  "id": "D-003",
  "date": "2026-11-02",
  "status": "accepted",
  "conclusion": "Injectra当前定位为代码注入工具／核心，仅交付runtime接入、精确目标注入和结果反馈；使用者负责载荷及应用界面适配。pre-runtime、管理平台、Agent、自动恢复与任意JS撤销不属当前交付；既有remove及清理接口作为实现事实保留。",
  "trigger": "用户批注纠正将注入工具扩张为定制效果管理平台的理解，并提供定位图片、要求更新文档。",
  "context": "T-005@v1；MVP已验收，本次校准产品责任，不修改源码或历史证据。",
  "affects": ["当前产品定位", "后续需求与验收", "README及需求、架构、使用说明"],
  "alternatives": [
    "为每个应用维护控件、选择器与重渲染兼容：用户明确归属载荷作者",
    "默认下一阶段增加生命周期管理或回滚系统：超出当前注入目标",
    "将Hook或pre-runtime直接纳入本轮实现：图片仅提供方向及验收原则，无此实现授权"
  ],
  "commit": "本条所在提交",
  "supersedes": []
}
```

术语归一：README的“可插拔个性化工具”改为“代码注入工具／核心”；代码地图的“Web生命周期”改为“共享注入执行”，仅纠正文档职责描述，不重命名代码。D-001/D-002的首发、共享代码及既有CLI实现边界继续有效，本条补充产品责任、不撤销既有接口。回退可独立恢复本次文档提交；既有发布及验收记录保留。

<a id="d-001"></a>
## D-001 · 首发范围与共享代码边界

```tracking-decision
{
  "id": "D-001",
  "date": "2026-10-31",
  "status": "accepted",
  "conclusion": "首个 MVP 限于 macOS + Electron；共享 Web 注入代码保持单一来源，平台、宿主与应用差异隔离。具体语言与注入机制未决。",
  "trigger": "用户要求初始化多平台、多体系项目并明确 MVP 范围与可插拔架构要求。",
  "context": "T-001；空仓库；分享需求正文尚未读取成功。",
  "affects": ["T-001", "后续 MVP 任务与架构设计"],
  "alternatives": ["为每个平台复制 Web 逻辑：不符合用户要求", "同时实现全部平台：超出首发范围"],
  "commit": "本条所在提交",
  "supersedes": []
}
```

来源：[需求证据](requirements.md)。候选模式及接口见[架构边界](architecture/boundaries.md)。
回退/演进：本次仅创建仓库与文档，无目标应用副作用；后续需求改变本决策时建立替代 Decision，保留原提交和证据。

<a id="d-002"></a>
## D-002 · CLI核心与适配边界

```tracking-decision
{
  "id": "D-002",
  "date": "2026-10-31",
  "status": "accepted",
  "conclusion": "在D-001首发/共享边界内采用Rust CLI、CDP本地运行时适配、独立共享Web载荷；仅实现显式页面发现、apply/remove及只读macOS启动规划。",
  "trigger": "已读取分享正文；用户明确MVP无需图形界面，先做好核心。",
  "context": "T-002@v1。D-001的语言/机制开放项在此收敛，其首发和隔离约束继续有效。",
  "affects": [
    "T-002",
    "workspace crate边界",
    "CLI与Bundle合同"
  ],
  "alternatives": [
    "完整GUI/管理平台：不是当前核心",
    "直接改ASAR或原生Hook：非必要且增加修改/恢复风险",
    "Node CLI：反馈更快，但本项目选择Rust承担未来原生接入与无Node运行时分发，接受编译成本"
  ],
  "commit": "本条所在提交",
  "supersedes": []
}
```

接口与限制见[架构](architecture/boundaries.md)。成本：Rust工具链与测试用Node并存；当前仅一个运行时实现，不承诺零成本新增引擎。恢复：停止CLI不撤销已注入页面内容，使用remove执行受管理清理；源码按独立提交回退，保留历史测试证据。
