# 决策记录

<a id="d-007"></a>
## D-007 · 插件库与注入器职责隔离

```tracking-decision
{
  "id": "D-007",
  "date": "2026-11-05",
  "status": "accepted",
  "conclusion": "所有实际插件、示例插件与可交付Bundle统一归web-ui-plugins；Injectra仅保留注入核心、接入参考和最小临时测试数据，不持有插件副本；人工验证显式读取外部manifest，核心回归不得依赖插件库。",
  "trigger": "用户明确要求插件集中到指定库、旧位置删除、当前项目不得混入插件。",
  "context": "T-016@v1；Injectra基线722c4f3，插件库d9f58c0，论文旧项目20e6fe0；迁入前均干净，无冻结。",
  "affects": [
    "插件源码归属",
    "examples及参考payload移出",
    "测试输入和参考验证入口",
    "统一插件catalog"
  ],
  "alternatives": [
    "继续维护外部论文项目索引：不满足统一存放",
    "在核心库保留插件样例或副本：混淆职责",
    "核心测试直接依赖外部插件：破坏核心独立回归"
  ],
  "commit": "本条所在提交",
  "supersedes": []
}
```

D-006参考归档边界补充：接入、桥接与验证工具可以保留，可交付面板插件移出；其共享单一来源、默认构建隔离和历史证据规则继续有效。插件搬迁采用先核对后删除，原Git历史保留在仓库外缓存；回退需协调插件库与核心两个提交，不能恢复为长期双份源码。

<a id="d-006"></a>
## D-006 · 在主仓库归档按软件组织的参考源码

```tracking-decision
{
  "id": "D-006",
  "date": "2026-11-04",
  "status": "accepted",
  "conclusion": "按用户确认，将软件注入参考源码与启动样例归档到references/apps，共享Electron/V8桥接、Web程序生成器和CDP调用入口只保留一份；与产品workspace、默认代码测试及发布隔离。按共享/应用专属/验证辅助统计源码，维护工具另列，不将某软件案例直接作为专用产品后端。",
  "trigger": "用户提出主worktree保存每款软件实现以便参考统计，确认目录方案并明确要求合入后提交。",
  "context": "T-013@v1；main基线caa6039。D-005此前只采纳结论，本次是新的源码参考归档授权；原独立Probe及其历史提交保留。",
  "affects": ["references参考源码导航", "软件案例元数据与统计口径", "可选的独立离线检查"],
  "alternatives": [
    "只在外部Probe保存代码：主仓库无法直接浏览/统计参考实现",
    "整体合入产品后端：用户当前只要求参考归档，且跨应用/ABI支持未完成",
    "每款软件复制CDP/共享Web或V8桥接：重复实现和统计，违反单一来源边界",
    "带入原始日志/profile/token/二进制：不属于参考源码，运行数据继续仓库外保存"
  ],
  "commit": "本条所在main提交",
  "supersedes": []
}
```

参考源码不等于产品交付；历史实机结果仍绑定原版本/提交/run。迁移版只做静态与离线检查，不回填旧兼容性结论。正式后端将来如实施，另按现有执行域合同实现与验收。目录与统计入口见[references](../references/README.md)。

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

<a id="d-008"></a>
## D-008 · macOS多引擎正式接入

```tracking-decision
{
  "id": "D-008",
  "date": "2026-11-10",
  "status": "accepted",
  "conclusion": "Mac优先，Chromium+WebKit、主页面；在现有分层新增并列后端和共享Frida通信，CLI统一入口，Python/Frida作为锁定可选依赖。公共契约或职责变更另行与用户讨论。",
  "trigger": "用户纠正按软件数衡量进展，确认Mac先完成、结构变化须讨论，并逐项批准计划后要求实施。",
  "context": "T-017@v1，基线62c1676；既有MVP和探针历史证据保留，不改写D-005/D-006当时的交付范围。",
  "affects": [
    "macOS后端交付",
    "CLI显式后端选择",
    "可选依赖与统一验收"
  ],
  "alternatives": [
    "先推进Windows/Gecko：不在本阶段",
    "仅验证探针：用户要求统一正式工具",
    "立即纯Rust绑定：用户选择复用Python/Frida"
  ],
  "commit": "本条所在提交",
  "supersedes": []
}
```

<a id="d-009"></a>
## D-009 · 控制端与目标设备边界解耦

```tracking-decision
{
  "id": "D-009",
  "date": "2026-11-10",
  "status": "accepted",
  "conclusion": "保留现有crate及core/Web公开契约，在Frida内部区分控制端执行、目标身份provider与传输connector；会话v2分域，v1只读兼容；Frida页面ID绑定设备/进程/路由；WK执行使用独立宿主发现/调度接口。当前不实现手机端。",
  "trigger": "用户要求review并解决控制端与目标设备未充分分开的问题，为不同于桌面的手机实现保留入口。",
  "context": "T-019@v1；输入74844e7；用户要求保持当前结构，内部解耦沿用既有边界，不增加公开分层或移动端功能。",
  "affects": [
    "Frida内部接口与会话描述",
    "目标身份和页面作用域",
    "WK宿主适配",
    "扩展验收边界"
  ],
  "alternatives": [
    "把控制端OS/PID检查复用为任意目标检查：无法区分远端设备",
    "现在实现手机或USB/ADB：超出当前授权",
    "把设备管理写入core/Web或新增crate：当前内部接口足够，无需改变公开结构",
    "接受旧未限定页面ID：有跨会话误选风险"
  ],
  "commit": "本条所在提交；会话/目标分离前置24ca687",
  "supersedes": []
}
```

[接口与支持范围](architecture/device-boundaries.md) · [验证与恢复](validation/device-boundaries.md)。设备/provider、连接器及宿主profile分别注册；未知实现明确Unsupported，禁止回退本机。旧页面ID重新发现；测试接口不作手机支持声明。

<a id="d-010"></a>
## D-010 · 适配复用与32方案插件化阈值

```tracking-decision
{
  "id": "D-010",
  "date": "2026-11-11",
  "status": "accepted",
  "conclusion": "注入实现按协议/宿主及实际ABI差异复用，应用专有接入例外隔离为适配策略；用户确定适配方案插件化阈值32，新增第33个正式独立方案前完成独立适配包注册/装配，避免继续扩展核心或CLI分发分支。",
  "trigger": "用户担忧每个app单独维护注入实现不可持续，澄清阈值按适配方案泛指并将16改为32。",
  "context": "T-030@v1，基线4309418；当前正式Web后端按契约静态crate装配，T-029为Hook触发既有页面桥接的专用预览。",
  "affects": [
    "适配方案计数与扩展准入",
    "平台/宿主/应用例外边界",
    "后续插件装配验收"
  ],
  "alternatives": [
    "每个应用复制完整注入实现：重复维护，无法随应用规模持续扩展",
    "仅把独立应用脚本移到插件目录：改变存放方式不能消除重复适配",
    "把32解释为应用上限或文件/crate数：不符合用户的方案计数要求",
    "本轮立即实现动态ABI/插件市场：本轮为边界确认，具体实现方案尚未选择"
  ],
  "commit": "本条所在提交",
  "supersedes": []
}
```

[计数、现状与验收](architecture/boundaries.md) · [用户边界](requirements.md)。此决策补充未来扩展约束，不覆盖既有试验或静态装配历史；变更阈值或插件化验收需另行记录用户决策。

<a id="d-011"></a>
## D-011 · macOS仅考虑Apple Silicon硬件

```tracking-decision
{
  "id": "D-011",
  "date": "2026-11-11",
  "status": "accepted",
  "conclusion": "macOS硬件支持及后续验证范围仅Apple Silicon，排除Intel Mac；历史未验证Intel记录保留，不继续作为待补项。硬件范围不自动扩张或变更Rosetta/x86_64目标支持声明。",
  "trigger": "用户明确mac不考虑Intel芯片，要求回到先前真实应用列表查看未验证项。",
  "context": "T-031@v1；基线35eb610；当前实机证据来自arm64主机。",
  "affects": [
    "macOS硬件支持范围",
    "后续验证清单",
    "README与需求入口"
  ],
  "alternatives": [
    "继续列Intel硬件为验证缺口：与用户范围决定冲突",
    "改写历史未验证证据或删除架构代码：非本轮要求",
    "将Intel硬件与Apple Silicon上的x86_64进程混为一谈：混淆设备与目标架构"
  ],
  "commit": "本条所在提交",
  "supersedes": []
}
```

[范围](requirements.md) · [原应用清单](compatibility/current-matrix.md)；本次仅确定硬件范围，未修改构建目标或真实后端实现。

<a id="d-012"></a>
## D-012 · 通用注入与轻量dev模式范围校准

```tracking-decision
{
  "id": "D-012",
  "date": "2026-11-12",
  "status": "accepted",
  "conclusion": "当前方向为通用注入核心与pre-runtime dev加载模式：一次准备小型入口，现有编辑器修改外置增量，真实App查看效果，必要重载/重启但普通修改免重打包。工作台、内置编辑器、文件管理和完整工作区系统退出当前范围；保留增量文件/版本/执行回执等必要来源信息。",
  "trigger": "用户明确指出工作台过度工程化，并要求先全面更新文档。",
  "context": "T-040由v1通用注入与增量开发平台 v1升级为v2通用注入核心与 pre-runtime dev 模式；T-041仅负责文档校准。",
  "affects": [
    "产品边界与文档导航",
    "T-040名称/定义版本与旧执行退役",
    "pre-runtime开发反馈流程及来源边界",
    "工作台/工作区留存实现的状态标识"
  ],
  "alternatives": [
    "继续完整Web工作台：超出用户澄清范围",
    "独立浏览器模拟App：不符合真实App查看效果的要求",
    "因收敛文档直接删除已有代码：未获此授权",
    "按品牌继续复制接入脚本：不满足通用注入目标"
  ],
  "commit": "本条所在文档提交",
  "supersedes": []
}
```

[当前边界](architecture/dev-mode.md) · [T-040旧版记录](archive/T-040.md#t-040)。补充既有长期边界，不改写D-003/D-004/D-010及已取得的实测/tag证据。

<a id="d-013"></a>
## D-013 · CLI收敛与平台化代码删减

```tracking-decision
{
  "id": "D-013",
  "date": "2026-11-12",
  "status": "accepted",
  "conclusion": "仅提供CLI，runtime与pre-runtime分路；选择性删除Studio/前端/完整workspace，保留通用准备器、Bootstrap和最小增量来源/执行正确性。页面保存直接应用，暂不做用户startup、导出、自动重启。",
  "trigger": "用户明确批准Injectra CLI路线收敛与平台化代码删减计划。",
  "context": "T-040 v3接续v1退役与v2文档范围；4cf736e为对照，不整段git revert。",
  "affects": [
    "CLI",
    "准备器和Bootstrap",
    "平台模块删除",
    "构建与验证"
  ],
  "alternatives": [
    "恢复工作台：用户明确排除",
    "整段回退：会丢失有效通道及正确性改进",
    "只改文档：达不到获批闭环"
  ],
  "commit": "本条所在提交",
  "supersedes": []
}
```

补充D-012；其文档阶段事实保留，本轮按后续明确授权删除代码。[当前范围](architecture/dev-mode.md)。

<a id="d-014"></a>
## D-014 · 原宿主主模块委托

```tracking-decision
{
  "id": "D-014",
  "date": "2026-11-12",
  "status": "accepted",
  "conclusion": "准备入口按布局选择cjs-insert或main-delegate；嵌套/不透明CJS入口字节保留，shim通过宿主Module._load(original,null,true)建立原主模块身份。Electron框架按元数据识别，拒绝品牌名称分支。",
  "trigger": "用户指出QQ既有成功路线，T040的Unsupported只是准备器覆盖不足。",
  "context": "T042对照旧QQ成功包装器，发现嵌套ASAR与框架文件名假设；采用保持身份的通用委托而非普通require。",
  "affects": [
    "preparation入口探测/资源准备",
    "框架身份识别",
    "兼容性与验证记录"
  ],
  "alternatives": [
    "将QQ永久列为不支持：与旧成功证据矛盾",
    "按品牌写专用包装器：违背通用目标",
    "普通require原入口：不保持主模块身份",
    "解密改写原载荷：无必要且增加风险"
  ],
  "commit": "本条所在提交",
  "supersedes": []
}
```

补充D-013的准备器覆盖；保留旧Unsupported证据。[验证](validation/qq-cli-dev.md)。
