# 任务入口

当前权威来源为本地main。T-021/T-022原交付记录保留当时口径，后续接收状态以T-023为准；更早任务亦保留原验收事实。保留完整定义与证据，默认从[任务索引](tasks.md)按Task ID定向读取。

## 最近完成的集成

<a id="t-023"></a>
### T-023 · Gecko 验证成果合入 main

```tracking-task
{
  "id": "T-023",
  "name": "Gecko 验证成果合入 main",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "将Firefox和Thunderbird已记录的实机验证成果快进合入本地main，保持支持边界和原始证据。",
  "scope": "本地Git快进、主分支回归、当前集成状态文档；不推送、不重新启动应用、不改正式产品功能或历史验收结论。",
  "acceptance": "main包含9531cad及前置2c0117c；make check/test-frida、脚本语法与文档任务检查通过；Thunderbird partial和正式Gecko后端未交付的边界保持；来源标记核验、两工作树干净。",
  "execution_ref": "archive/T-023.md#t-023",
  "acceptance_evidence": [
    "main从885b8b5快进至9531cad，包含Firefox2c0117c和Thunderbird9531cad；无冲突，两边原未提交内容为空。",
    "main上make check/test-frida通过：18 Rust、5 Web、8 WK host、23 Python；新增脚本语法与JSON、已测源码无差异核验通过。",
    "Firefox验证通过、Thunderbird严格合同partial及HTML/BiDi限制原样保留；正式CLI未新增Gecko后端，未重启应用或重跑实机验证。",
    "T-021/T-022提交来源标记核验；本条所在集成记录提交也带来源标记，随后快进main并核验两工作树干净；未推送。"
  ],
  "unplanned_reason": "用户明确要求合并到main。"
}
```

本窗口/root、single-writer；输入9531cad，main输入885b8b5，均已核对干净。main已接收验证成果；当前记录随本次提交移交main，本窗口分支作为接收快照。[执行与核验](archive/T-023.md#t-023)。


## 本工作树最近完成

<a id="t-022"></a>
### T-022 · macOS Thunderbird 安装与实机验证

```tracking-task
{
  "id": "T-022",
  "name": "macOS Thunderbird 安装与实机验证",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "在Mac安装官方Thunderbird，在独立空配置验证宿主接入、共享runtime执行与清理。",
  "scope": "官方正式版下载校验、用户Applications安装；独立HOME/profile下应用窗口及自有内容页的接入、JS/CSS/HTML、交互、更新移除、Promise/异常与隔离；脚本、文档和提交。不登录邮箱、不导入账号、不发送邮件、不改默认客户端，不新增公开产品后端。",
  "acceptance": "精确版本/来源/hash/签名可追溯；真实Thunderbird实例有目标识别和逐项结果；失败及未测如实保留；只退出本次实例，保留安装；自审和项目检查通过。",
  "execution_ref": "archive/T-022.md#t-022",
  "acceptance_evidence": [
    "Thunderbird159.0官方安装完成，用户Applications保留；SHA512、Mozilla签名与spctl验签通过，精确来源和下载参数锁定。",
    "最终thunderbird-gecko-Ta3rSb有27项成功检查：内容页JS/CSS/HTML、主界面JS/CSS+DOM控件、更新移除、异常、隔离及失效目标拒绝通过；主界面HTML按钮字符串被清空，严格整组为partial/退出1，未伪报全通过。",
    "已确认标准BiDi getTree不枚举Thunderbird标签页；原环境执行依赖显式系统访问、Thunderbird宿主枚举与Gecko调试接口，未修改正式后端或共享runtime。",
    "两张实际截图已查看；10次检查/尝试的自有PID全部退出，路径复查无残留。测试账号均为空，安装及初始化默认配置保留；未导入账号、发送邮件或设置默认客户端。",
    "make check、Firefox默认安装准备回归及脚本语法检查通过；原失败run、项目检查和自审留档；本条所在提交含来源标记并核验。成果在本工作分支，未合入main/推送。"
  ],
  "unplanned_reason": "用户要求Thunderbird在Mac安装并验证，并要求继续。"
}
```

本窗口/root、single-writer，输入2c0117c；交付记录在本工作分支。已完成安装及范围内验证，严格整组为partial；该结果不冒充全量支持。[验证说明](thunderbird-validation.md) · [执行与关闭上下文](archive/T-022.md#t-022)。


## 本工作树最新完成

<a id="t-021"></a>
### T-021 · macOS Gecko 实机接入验证

```tracking-task
{
  "id": "T-021",
  "name": "macOS Gecko 实机接入验证",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "下载官方Firefox，在Mac独立配置实例上验证Gecko页面接入与共享Web runtime执行，明确正式产品适配缺口。",
  "scope": "官方macOS Firefox下载、校验及仓库外保存；新增可复用验证脚本；独立配置/本地测试页上的BiDi发现、JS/CSS/HTML、Promise/异常、更新清理及目标隔离测试；文档和提交。不新增公开后端/crate，不改已有架构，不操作原账号和默认浏览器设置。",
  "acceptance": "记录精确版本/来源/hash和Mac架构；实际页面验证有逐项结果，失败与未测如实保留；只退出本次实例；说明能否复用共享runtime及正式CLI是否支持；文档、review和检查通过。",
  "execution_ref": "archive/T-021.md#t-021",
  "acceptance_evidence": [
    "官方Firefox160.0已下载到Mac仓库外缓存，SHA512、Mozilla签名及spctl验签通过；实际会话PID/profile与自有启动实例一致。",
    "firefox-bidi-Js5cnC的15项真实页面检查通过，包括共享runtime的JS/CSS/HTML、交互/Promise/异常、更新清理、页面隔离与失效目标拒绝；截图已查看，测试进程退出并复查无残留。",
    "初次初始化超时和旧realm错误码断言失误的run均保留；当前CDP CLI接入该Firefox为404，正式BiDi后端尚未实现，不外推Thunderbird。",
    "make check及脚本语法通过；文档/任务检查、自审和带来源标记提交完成。本工作分支保存验证脚本和记录，未合入main/推送。"
  ],
  "unplanned_reason": "用户明确要求下载Mac上的Gecko桌面软件并实际测试。"
}
```

本窗口/root、single-writer；基线885b8b5，交付记录在本工作分支。下载副本和运行证据保留在Mac外部缓存，正式后端未变。[验证说明](gecko-validation.md) · [执行与关闭上下文](archive/T-021.md#t-021)。


## 最近完成的集成

<a id="t-020"></a>
### T-020 · 设备边界解耦成果合入 main

```tracking-task
{
  "id": "T-020",
  "name": "设备边界解耦成果合入 main",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "将T-019已验证成果快进合入本地main并确认主分支状态。",
  "scope": "本地Git快进、集成回归及当前状态文档；不推送、不重新操作用户真实应用、不修改功能。",
  "acceptance": "main包含c6d0cf3和全部T-019实现；make check/test-frida及文档任务检查通过；来源标记核验；两工作树干净。",
  "execution_ref": "archive/T-020.md#t-020",
  "acceptance_evidence": [
    "main从74844e7快进至c6d0cf3，完整包含24ca687、01cbf7f和T-019关闭记录；无冲突，无未提交内容覆盖。",
    "main上make check/test-frida通过：18 Rust、5 Web、8 WK host、23 Python；功能源码与c6d0cf3无差异；未重新启动真实应用。",
    "三个T-019提交来源标记核验；本条所在集成记录提交也带来源标记，随后快进main并核验两工作树干净；未推送。"
  ],
  "unplanned_reason": "用户明确要求合并到main。"
}
```

本窗口/root、single-writer；输入c6d0cf3，main输入74844e7，均已核对干净。main已接收全部实现；当前状态与集成记录随本次提交移交main，本窗口分支只作接收快照。[执行与核验](archive/T-020.md#t-020)。


## 本工作树最近完成

<a id="t-019"></a>
### T-019 · 控制端与目标设备边界解耦

```tracking-task
{
  "id": "T-019",
  "name": "控制端与目标设备边界解耦",
  "version": "v1",
  "mode": "staged",
  "status": "done",
  "goal": "消除控制端本机与目标设备混用，建立设备扩展接口，保持现有Mac行为。",
  "scope": "Frida内部控制端/会话/目标provider/连接器分离；v2会话并兼容v1；页面ID加入设备进程作用域；Mac宿主发现与WK执行分离；测试和文档。不新增crate、不改core/Web公开契约、CLI命令，不实现手机/USB/ADB或自动恢复。",
  "acceptance": "未知设备在本机核验/连接前拒绝；不同设备/进程的相同局部ID不互认；旧会话无损读取；Mac目标身份核验及既有独立宿主回归通过；文档、自审及带来源标记提交完成。",
  "execution_ref": "archive/T-019.md#t-019",
  "acceptance_evidence": [
    "24ca687分离控制端执行、目标身份provider、连接器及v2会话；01cbf7f分离WK宿主并完成回调、输入与诊断补强。core/Web及CDP源码、crate依赖和CLI命令保持。",
    "18 Rust、5 Web、8 WK host、23 Python及独立CDP/Gadget/WK宿主验证通过；旧v1抖音会话发现7页且源SHA未变。原targets超时与后续通过分别保留，见validation/device-boundaries.md。",
    "自审、文档/任务检查和来源标记核验完成；当前只有Mac实现，手机入口是可替换接口而非兼容声明；旧页面ID需重新发现。本工作分支交付，未合入main或推送。"
  ],
  "priority": "P1",
  "unplanned_reason": "用户明确要求review并解决控制端/目标设备及扩展入口问题，同时不实现手机端。"
}
```

[审查与设计](architecture/device-boundaries.md) · [关闭上下文](archive/T-019-context.md) · [执行](archive/T-019.md#t-019)

## 最近完成的集成

<a id="t-018"></a>
### T-018 · macOS 多引擎成果合入 main

```tracking-task
{
  "id": "T-018",
  "name": "macOS 多引擎成果合入 main",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "将T-017已验证成果快进合入本地main，同步当前交付状态并核验主分支。",
  "scope": "仅本地Git集成、当前状态文档及集成回归；不推送，不批量改写历史，不重新操作用户真实应用，不改变架构和功能。",
  "acceptance": "main包含c2f090f及其全部功能提交；make check/test-frida通过；当前说明不再误称未合入；新提交具有来源trailer；main和本窗口工作树干净。",
  "execution_ref": "archive/T-018.md#t-018",
  "acceptance_evidence": [
    "本地main快进接收c2f090f及完整实现链，无冲突、无覆盖；功能源码与eb42869一致。",
    "main上的make check/test-frida通过（18 Rust、5 Web、6 Python），文档/任务校验通过；当前交付状态已同步。",
    "所有本次新建提交均含Commit-Origin: chinami-mac-work并核验；main及本窗口分支收口至同一提交，未推送。"
  ],
  "unplanned_reason": "用户明确要求合入main；T-017原交付不包含主分支集成。"
}
```



本窗口/root为唯一Parent，single-writer；main已快进接收2bc763d（包含c2f090f及全部功能提交），当前权威写入位置为main。本窗口分支跟随接收结果，作为main的已采用快照。原主树基线62c1676，无未提交内容被覆盖。 [执行与核验](archive/T-018.md#t-018)。

## 本工作树已关闭任务

<a id="t-017"></a>
### T-017 · macOS 多引擎接入与正式工具整合

```tracking-task
{
  "id": "T-017",
  "name": "macOS 多引擎接入与正式工具整合",
  "version": "v1",
  "mode": "staged",
  "status": "done",
  "goal": "在macOS arm64统一交付Chromium/CDP、Electron Gadget与WebKit主页面注入，按引擎/宿主/路线分别验收。",
  "scope": "保留现有核心契约与共享runtime；新增并列后端、共享Frida桥接、CLI选择、锁定可选环境、CEF/NW.js/WKWebView验证；不推进Windows/Gecko/iframe/自动重注入，不改第三方安装包/系统保护，不自动重启用户应用。",
  "acceptance": "既有CDP回归通过；Gadget正式CLI在独立Electron及抖音真实页面通过；CEF/NW.js独立宿主通过；WebKit独立宿主与至少一个真实应用通过；结果/异常/Unknown/资源生命周期明确；文档、review和可恢复提交完成。",
  "execution_ref": "archive/T-017.md#t-017",
  "acceptance_evidence": [
    "正式CLI默认CDP兼容，新增Electron Gadget/WKWebView并列后端；core/Web契约与唯一runtime保持。",
    "macOS arm64：抖音8.7.0、独立Electron36、CEF156/NW.js0.119.2、独立双WKWebView及用户批准的真实Codex Switcher0.2.11通过对应合同/CLI检查；原CDP Electron45回归通过。",
    "最终18 Rust/5 Web/6 Python测试、原生清理/控制端退出、可移出源码运行、文档/任务检查与自审通过；实现eb42869及前置提交可恢复，运行证据仓库外保存。",
    "已按授权退出本次真实测试新实例，未导入原账号或执行账号切换；Gadget ABI/加载条件、未验证平台及极端阻塞边界如实记录；未合入main/推送，不声称所有Mac软件可注入。"
  ],
  "priority": "P1",
  "unplanned_reason": "2026-11-10用户确认Mac优先、Chromium+WebKit、正式工具交付、并列后端扩展、仅主页面、复用Python/Frida，并明确Implement the plan。"
}
```

[关闭上下文](archive/T-017-context.md) · [实施与验收](macos-delivery.md) · [执行](archive/T-017.md#t-017)

## 已关闭任务

<a id="t-016"></a>
### T-016 · 插件集中迁移与注入核心职责隔离

```tracking-task
{
  "id": "T-016",
  "name": "插件集中迁移与注入核心职责隔离",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "将全部已识别的Injectra用户插件和可复用插件载荷集中到独立web-ui-plugins库，移除旧位置并禁止核心仓库保存插件。",
  "scope": "Injectra与统一插件库的插件迁移、论文旧项目删除和Git元数据备份、两组临时定制脚本、测试解耦、引用与边界文档；不操作运行应用，不迁移注入桥接或删除历史run证据。",
  "acceptance": "迁入文件hash一致且无覆盖；插件目录集中、旧活跃源码位置删除；Injectra无可交付插件包、测试不依赖插件库；两库构建/测试及文档任务检查通过；review、迁移清单和可恢复提交完成。",
  "execution_ref": "archive/T-016.md#t-016",
  "acceptance_evidence": [
    "7项插件集中到指定web-ui-plugins库，53文件迁入前SHA核对，5个旧活跃目录删除且论文Git备份可恢复；统一库67908ba，22插件测试通过；核心无可交付插件Bundle，16 Rust/5 Web、Electron10项及参考离线检查通过；review及文档/任务检查完成。"
  ],
  "unplanned_reason": "用户明确要求插件全部集中并在旧位置删除，核心项目不得含插件。"
}
```

[迁移与验收](migrations/plugin-boundary.md) · [执行与review证据](archive/T-016.md#t-016)

<a id="t-015"></a>
### T-015 · 当前文档精简与历史下沉

```tracking-task
{
  "id": "T-015",
  "name": "当前文档精简与历史下沉",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "按用户批准清单减少默认阅读与重复维护，保留独有契约、已关闭任务定义及历史实测证据。",
  "scope": "任务入口及历史任务目录；架构/契约合并；旧评估和验收正文归档；T-002/T-008关闭上下文合并删除；修复导航与引用。",
  "acceptance": "已关闭任务及Execution/Step/Decision完整且唯一；契约独有条款保留；历史验收/评估正文可追溯；默认首读明显缩短；删除仅限批准文件；review及文档/任务校验通过。",
  "execution_ref": "archive/T-015.md#t-015",
  "acceptance_evidence": [
    "重复契约/关闭上下文3份已合并删除；旧评估与原验收正文移入归档；默认任务入口改为轻量tasks.md，TODO保留校验器要求的权威状态；40条历史tracking记录逐字段一致，原历史正文/上下文/契约条款保全检查通过；文档和任务校验通过。"
  ],
  "unplanned_reason": "用户明确批准文档清理清单，并要求慎重清理。"
}
```

[执行证据](archive/T-015.md#t-015)

<a id="t-014"></a>
### T-014 · 应用平台与内核兼容性总表

```tracking-task
{
  "id": "T-014",
  "name": "应用平台与内核兼容性总表",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "用一张当前兼容性总表统一应用、平台、版本、宿主/内核、接入路线与验证状态，替代碎片化状态入口。",
  "scope": "兼容性矩阵、README/文档导航、旧报告入口及外部REPORT.txt指向；原始历史正文留底，不改尝试/源码/应用状态。",
  "acceptance": "原40候选无遗漏；合并豆包历史成功、抖音新路线、Figma/双宿主后续证据与分类勘误；不同版本/平台/路线不混算；总表成为默认入口；原REPORT正文可恢复；review、文档与任务检查通过。",
  "execution_ref": "archive/T-014.md#t-014",
  "acceptance_evidence": [
    "当前总表收录41应用名/44版本路线组合，原40候选覆盖检查通过；合并豆包、抖音、Figma、双宿主后续成果与勘误；README/docs导航及旧REPORT收敛到总表，旧正文SHA逐字节留底；review和文档/任务检查通过，仅文档和报告入口改动。"
  ],
  "unplanned_reason": "用户明确要求合并每个应用的平台/内核等为一张表，旧碎片入口不再作为当前阅读入口。"
}
```

[当前总表](compatibility/current-matrix.md) · [执行与review](archive/T-014.md#t-014)

<a id="t-013"></a>
### T-013 · 软件注入参考实现归档

```tracking-task
{
  "id": "T-013",
  "name": "软件注入参考实现归档",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "将已确认的软件接入参考源码和启动样例归档到主worktree，按软件导航并可靠统计，保持产品边界。",
  "scope": "references中的抖音Gadget源码、共享桥接/生成器、Figma/优酷/WorkBuddy启动样例、独立依赖与离线检查及文档/来源/统计；不集成为产品后端、不启动或附加应用、不迁入运行数据。",
  "acceptance": "四款软件入口、来源版本与验证边界可检索；共享代码只存/计一次；解除旧Probe运行依赖并保持唯一Web runtime；参考目录独立于产品workspace/默认代码检查；静态/离线载荷、产品回归、文档/任务及review通过后提交。",
  "execution_ref": "archive/T-013.md#t-013",
  "acceptance_evidence": [
    "四款软件参考入口及来源/历史范围已归档；抖音共享agent/bridge和Web程序生成器单列，CDP案例只保存各自启动条件，不复制后端或Web runtime。",
    "来源SHA核对、独立Rust生成器、Python/JS/Shell静态检查、3个agent离线编译、DOM载荷smoke通过；生成apply与历史实测产物逐字节一致。",
    "主仓库make check通过：8 crate架构、格式/clippy、16 Rust和5 Web测试；参考目录不在产品workspace，未启动或附加任何目标应用。",
    "分组统计每文件唯一归属，共享173/应用入口163/验证188/维护153行（本提交口径），原Probe历史保留；文档/任务检查、自审与独立main提交完成。"
  ],
  "unplanned_reason": "2026-11-04用户明确确认references参考归档方案，并要求合入主worktree后提交；不额外创建产品发布里程碑。"
}
```

[参考入口](../references/README.md) · [执行与review](archive/T-013.md#t-013)

<a id="t-012"></a>
### T-012 · 抖音非 CDP 阶段性成果归档

```tracking-task
{
  "id": "T-012",
  "name": "抖音非 CDP 阶段性成果归档",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "将用户认可的抖音非CDP阶段性成果采纳到main，形成准确、可复核、可检索的结果与代码来源记录。",
  "scope": "非CDP实测报告、脱敏证据、导航/验证/决策与本次归档执行；不整体迁入探针代码、不改产品行为、不重写旧实测或普查统计、不推送。",
  "acceptance": "8.7真实页面成功、7.9失败和后续复验失败分别记录；13项来源及源码差异可复核；阶段性用户认可不扩大为产品交付；review、文档/任务检查通过并独立提交main。",
  "execution_ref": "archive/T-012.md#t-012",
  "acceptance_evidence": [
    "已核对认可run的13项全部通过及8.7运行环境，生成不含凭据/profile/原始日志的证据摘要；完整Probe提交及验证脚本差异可追溯。",
    "报告、导航、验证与D-005统一记录非CDP阶段性成功，保留版本漂移/旧帧/失败边界；main仍只装配CDP产品后端。",
    "自审、文档/任务/证据一致性及差异检查通过；本记录所在提交为独立main归档提交，未推送。"
  ],
  "unplanned_reason": "2026-11-04用户明确认可本轮抖音成功并要求整理提交main；本任务归档阶段性成果，不额外创建产品发布里程碑。"
}
```

[实测报告](compatibility/douyin-gadget.md) · [脱敏证据](compatibility/douyin-gadget-evidence.json) · [执行与review](archive/T-012.md#t-012)

<a id="t-011"></a>
### T-011 · Figma CDP 验证结果归档

```tracking-task
{
  "id": "T-011",
  "name": "Figma CDP 验证结果归档",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "将侧会话Figma接入与黑屏排查证据归档，明确仍复用现有CDP注入实现。",
  "scope": "新增Figma兼容性报告与归档执行，补充导航和验证入口；不改产品代码、安装包、历史普查结果或应用进程，不推送。",
  "acceptance": "入口条件、外壳注入、在线页面恢复及未验证范围分开记录；根因推断不过度承诺；历史证据保留；完成review、文档和任务校验并独立本地提交。",
  "execution_ref": "archive/T-011.md#t-011",
  "acceptance_evidence": [
    "已核对三个独立run摘要；明确CDP分类、内部开关副作用、首轮实际点击、重开黑屏和代理加重启后的登录页证据；不回填全组基线通过统计。",
    "自审、make doc-check、project-tracking-check及git diff --check通过；本记录所在独立提交仅含文档，不重跑真实应用或扩大兼容性结论。"
  ],
  "unplanned_reason": "2026-11-03用户要求先提交本侧会话结果；本任务只负责归档，保留SIDE-FIGMA-001各次实测身份。"
}
```

[兼容性报告](compatibility/figma.md) · [执行及review](archive/T-011.md#t-011)

<a id="t-010"></a>
### T-010 · 抖音与双宿主 CDP 接入排查

```tracking-task
{
  "id": "T-010",
  "name": "抖音与双宿主 CDP 接入排查",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "查明抖音、WorkBuddy、优酷此前未完成验证的具体阶段与原因，并在可控条件下复验当前注入核心。",
  "scope": "定向检查三款应用启动代码、既有日志及必要的新建测试实例；复验现有CLI与自有面板；记录CocosCreator观察线索、Figma后续方向及微信开发者工具跳过决定；不改安装包或注入核心。",
  "acceptance": "三款应用各有基于证据的根因或明确未决边界；可达页面完成受控注入/交互/更新/清理，无法验证者准确记录停止原因；历史尝试保留，测试进程有清理审计，文档与任务检查通过。",
  "execution_ref": "archive/T-010.md#t-010",
  "acceptance_evidence": [
    "WorkBuddy与优酷经宿主目录/沙箱路径调整后使用原CLI完成整组基线验证；抖音3次新尝试未取得调试入口，具体参数处理机制仍未确认。新增6条脱敏记录，原31次历史保留；53个记录PID的定向清理复查无残留；自审与文档/任务检查通过。"
  ],
  "unplanned_reason": "2026-11-03用户要求重点排查抖音、WorkBuddy和优酷；微信开发者工具保留历史并跳过。"
}
```

[排查报告](compatibility/macos-cdp-followup.md) · [新增尝试](compatibility/macos-cdp-followup-attempts.csv) · [执行与review](archive/T-010.md#t-010)

<a id="t-009"></a>
### T-009 · 本机应用 CDP 注入普查结果归档

```tracking-task
{
  "id": "T-009",
  "name": "本机应用 CDP 注入普查结果归档",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "将本轮本机应用普查的可复核结果、复验原因与适用边界整理为仓库证据并提交。",
  "scope": "新增兼容性报告与脱敏尝试清单，更新文档导航、验证入口和任务记录；不改源码、不重跑应用、不提交原始运行数据。",
  "acceptance": "222个应用包、40个候选、26个尝试应用和31次尝试口径一致；首次与最终结果分开；失败和未测边界明确；review、文档及任务检查通过，形成独立本地提交。",
  "execution_ref": "archive/T-009.md#t-009",
  "acceptance_evidence": ["兼容性报告覆盖40个候选及方法边界；31条脱敏尝试与原始结果逐条核对，首次16个、最终20个完整通过的口径保留；自审、make doc-check、任务结构检查及git diff --check通过。仅文档与证据摘要变更，本条所在提交为独立本地归档提交。"],
  "unplanned_reason": "用户要求将已完成的本机应用CDP注入普查结果整理并提交。"
}
```

[普查报告](compatibility/macos-cdp-survey.md) · [逐次尝试](compatibility/macos-cdp-survey-attempts.csv) · [执行与review证据](archive/T-009.md#t-009)

<a id="t-008"></a>
### T-008 · 长期注入架构基础重构

```tracking-task
{
  "id": "T-008",
  "name": "长期注入架构基础重构",
  "version": "v1",
  "mode": "staged",
  "status": "done",
  "goal": "现在建立跨执行域注入基础，迁移现有CDP Web能力，并以原生契约测试证明长期扩展位置。",
  "scope": "core/Web/native/application契约及用例、CDP迁移、平台宿主拆分、兼容回归、架构与任务文档；不实现真实原生后端或跨平台兼容。",
  "acceptance": "公共core无Web/JSON/系统实现依赖；后端实例身份隔离；Web页面与原生内存契约分离；部分操作和执行未知准确；CLI及manifest兼容；make check、Electron及独立构建/依赖检查通过；review与分步提交可追溯。",
  "priority": "P1",
  "unplanned_reason": "用户批准并明确要求实施长期架构基础重构计划。",
  "execution_ref": "archive/T-008.md#t-008",
  "acceptance_evidence": [
    "8 crate长期执行域架构落实；目标身份/能力/错误确定性、独立原生内存契约、平台宿主拆分及兼容迁移完成。16 Rust、5 Web、真实Electron10核心项及兼容断言、独立构建与架构依赖门通过；自审完成，三阶段提交可追溯。原生仅契约与测试缓冲区。"
  ]
}
```

[验收报告](validation/architecture-foundation.md) · [执行与review证据](archive/T-008.md#t-008) · [关闭上下文](archive/T-008.md#t-008-context)

<a id="t-007"></a>
### T-007 · 长期扩展架构适配评估

```tracking-task
{
  "id": "T-007",
  "name": "长期扩展架构适配评估",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "评估当前仓库是否能承接多平台、多Web运行时及未来原生/内存注入，给出有证据的结构调整必要性和时机。",
  "scope": "现有模块、依赖、接口、构建与测试结构的只读审阅；新增评估报告及任务导航，不实施源码重构或新增后端。",
  "acceptance": "给出保留项、扩展障碍、调整优先级/触发条件/代价及验收方法；分清已证实结构与未验证兼容性；报告和任务检查通过。",
  "execution_ref": "archive/T-007.md#t-007",
  "acceptance_evidence": [
    "已完成5个crate、共享Web和构建测试入口审阅；offline Cargo metadata证实依赖方向；报告列出A-01至A-05、条件化调整顺序及验收；review和文档/任务检查通过。仅形成候选评估，无源码重构或新增兼容性结论。"
  ],
  "unplanned_reason": "用户在明确长期发展路线后要求评估当前仓库结构。"
}
```

[评估报告](archive/architecture-evolution-assessment.md) · [执行与review证据](archive/T-007.md#t-007)

<a id="t-006"></a>
### T-006 · Xmind 架构图排版演示

```tracking-task
{
  "id": "T-006",
  "name": "Xmind 架构图排版演示",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "在用户打开的Xmind中，以保留原图的方式演示可读的模块结构与runtime注入路径。",
  "scope": "通过Xmind界面新建或复制演示画布并排版；仅新增副本/演示，不覆盖原图，不改产品代码。",
  "acceptance": "原画布保留；演示画布可见、结构及当前执行路径可读；核对界面结果并记录保存状态。",
  "execution_ref": "archive/T-006.md#t-006",
  "acceptance_evidence": [
    "已通过Xmind界面制作01·模块结构（22主题、六层逻辑图）及02·runtime路径（7主题、5条直线箭头）；原画布40主题保留；另存演示文件，UI无Edited标识且文件存在；review、文档及任务检查通过。"
  ],
  "unplanned_reason": "用户明确要求直接操作当前打开的Xmind进行演示。"
}
```

[界面操作与review证据](archive/T-006.md#t-006)

<a id="t-005"></a>
### T-005 · runtime 注入产品边界校准

```tracking-task
{
  "id": "T-005",
  "name": "runtime 注入产品边界校准",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "按用户批注与定位图片统一当前产品为代码注入工具／核心，仅交付runtime注入能力。",
  "scope": "项目定位、需求、架构职责、使用与验收说明、导航及决策记录；不修改源码、命令行为或历史验收证据。",
  "acceptance": "当前注入合同、使用者责任、后续候选方向明确且跨文档一致；已有清理接口仅记为实现事实；review、文档与任务结构检查通过。",
  "execution_ref": "archive/T-005.md#t-005",
  "acceptance_evidence": ["定位、责任及当前验收已统一，D-003记录用户纠正；review、make doc-check、任务结构检查与git diff --check通过；仅Markdown变更。"],
  "unplanned_reason": "用户纠正产品范围并明确要求同步文档与当前产品边界。"
}
```

[产品边界](requirements.md) · [执行与review证据](archive/T-005.md#t-005)

<a id="t-004"></a>
### T-004 · MVP 验收收口与仓库推送

```tracking-task
{
  "id": "T-004",
  "name": "MVP 验收收口与仓库推送",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "记录用户已完成MVP验收，整理可追溯交付提交并推送用户建立的远端仓库。",
  "scope": "验收与版本映射文档、现有工作区和历史审阅、必要检查、提交和Git推送；无产品功能扩展。",
  "acceptance": "用户验收结论已落盘；review与检查通过；提交及MVP版本标识可追溯；远端目标正确且推送成功，确认远端提交与本地一致。",
  "execution_ref": "archive/T-004.md#t-004",
  "acceptance_evidence": [
    "用户验收已记录；review、make check与任务检查通过；main与v0.1.0-mvp已推送sqmw/injectra，ls-remote确认交付提交a763157一致；远端初始化LICENSE历史保留。"
  ],
  "unplanned_reason": "用户确认MVP通过并要求整理提交推送。"
}
```

[交付记录](releases.md) · [执行证据](archive/T-004.md#t-004)

<a id="t-003"></a>
### T-003 · 豆包 Electron 兼容性验证

```tracking-task
{
  "id": "T-003",
  "name": "豆包 Electron 兼容性验证",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "以本机已安装豆包验证Injectra核心链路，并如实识别其宿主和接入边界。",
  "scope": "应用版本/宿主识别、调试入口探测、真实页面CSS/HTML/JS注入与清理、兼容性记录；优先独立profile，不改包或账号操作。",
  "acceptance": "给出有证据的宿主类型和CDP可达性结论；可达则验证发现、注入、交互、重复应用与清理；不可达保留失败证据及具体阻塞；记录运行副作用和恢复状态。",
  "execution_ref": "archive/T-003.md#t-003",
  "acceptance_evidence": [
    "已识别为豆包2.33.5定制Chromium147，非标准Electron；现有CLI真实页面注入、交互、更新与清理通过，原调试端口关闭并恢复正常豆包。详见compatibility/doubao.md及归档执行。"
  ],
  "unplanned_reason": "用户指定真实豆包兼容性验证。"
}
```

名称沿用任务建立时的用户目标；实测宿主已纠正为定制Chromium，不计作标准Electron。
[实测报告](compatibility/doubao.md) · [执行记录](archive/T-003.md#t-003)

<a id="t-002"></a>
### T-002 · macOS Electron 注入核心 MVP

```tracking-task
{
  "id": "T-002",
  "name": "macOS Electron 注入核心 MVP",
  "version": "v1",
  "mode": "staged",
  "status": "done",
  "goal": "提供可运行CLI，向显式开放本地调试端口的Electron页面注入共享CSS/HTML/JS并移除受管理改动。",
  "scope": "Rust CLI；核心契约；共享Web载荷；CDP适配；macOS启动参数规划；独立Electron验证；文档与测试。",
  "acceptance": "页面发现和精确目标选择；CSS/HTML/JS注入与移除实测；重复应用不累积节点；脚本异常可见；拒绝非本地调试地址；核心不依赖具体适配；构建、测试、review与Electron闭环通过。",
  "priority": "P1",
  "unplanned_reason": "首次CLI垂直切片，暂不增加独立Milestone聚合。",
  "execution_ref": "archive/T-002.md#t-002",
  "acceptance_evidence": [
    "CLI核心验收通过：共享CSS/HTML/JS在真实macOS Electron fixture完成发现、注入、交互、更新、清理与失败路径；6项Rust、5项Web、10项Electron检查通过；review及文档完成，见validation.md和归档执行。"
  ]
}
```

[恢复上下文](archive/T-002.md#t-002-context) · [执行与步骤](archive/T-002.md#t-002)

<a id="t-001"></a>
### T-001 · Injectra 项目初始化

```tracking-task
{
  "id": "T-001",
  "name": "Injectra 项目初始化",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "建立可追踪的项目基础并核对 macOS Electron MVP 的需求与架构输入。",
  "scope": "Git 初始化；项目文档与任务骨架；读取分享需求；确定 MVP 工程初始化边界。",
  "acceptance": "Git、README、docs 入口、AGENTS、gitignore 可用；架构明确共享 Web 与可替换适配边界；已核对分享中的功能与技术栈，工程初始化范围可据此确定；完成适配当前产物的检查与 review。",
  "execution_ref": "archive/T-001.md#t-001",
  "acceptance_evidence": [
    "d55ffa5 保存基础骨架；make check、29条本地链接及tracking检查通过；分享正文已读取并写入requirements，用户确认CLI核心优先，实际实现归属T-002。"
  ],
  "unplanned_reason": "首次项目初始化，需求正文尚未取得，暂不建立无依据的 Milestone。"
}
```

[初始化证据](archive/T-001.md#t-001)
