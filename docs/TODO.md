# 任务入口

当前权威来源为本地main。T-021/T-022原交付记录保留当时口径，后续接收状态以T-023为准；更早任务亦保留原验收事实。保留完整定义与证据，默认从[任务索引](tasks.md)按Task ID定向读取。

## 当前执行

<a id="t-071"></a>
### T-071 · Git 分叉成果整合

```tracking-task
{
  "id": "T-071",
  "name": "Git 分叉成果整合",
  "version": "v1",
  "mode": "lightweight",
  "status": "in_progress",
  "goal": "将英文论文和GitHub展示/CI两路已提交成果整合到统一main，并同步远端和当前工作树。",
  "scope": "保留原提交与恢复分支，非破坏性合并共享README/导航/检查入口；不重写论文正文或历史、不重置/强推、不删除工作树或分支，保持GitHub private。",
  "acceptance": "main包含两路原提交且与origin/main同步，工作区无冲突；中英文稿/作者与原证据保留，双语项目介绍和CI能力保留；文档/任务/元数据检查及双语构建通过。",
  "execution_ref": "TODO.md#t-071",
  "acceptance_evidence": [],
  "unplanned_reason": "2026-11-16用户提供两agent分叉图并要求处理；本地main独有英文稿，远端独有5个GitHub完善提交。"
}
```

上下文r1：本窗口在主工作区main负责整合，基线efe50c2与origin/main的92abfee已各自保留恢复分支。两个执行者的Task69/70原状态及证据合并保留；无未提交用户改动。

```tracking-execution
{
  "id": "T-071/E-001",
  "name": "合并论文与 GitHub 两路提交",
  "task_id": "T-071",
  "task_version": "v1",
  "status": "running",
  "authorization": "用户明确要求处理两agent并行工作导致的分支问题，采用保留历史的合并与同步。",
  "scope": "保留原提交与恢复分支，非破坏性合并共享README/导航/检查入口；不重写论文正文或历史、不重置/强推、不删除工作树或分支，保持GitHub private。",
  "steps": [],
  "evidence": [
    "本地main efe50c2、origin/main 92abfee，共同基线0cfd758；两个工作区均干净。只读merge-tree预演仅README冲突，其余共享文件自动合并。",
    "已保留两条恢复分支及仓库外before.json；旧tag和原始成果不变。"
  ],
  "stop_reason": ""
}
```

<a id="t-070"></a>
### T-070 · GitHub 研究仓库展示与协作完善

```tracking-task
{
  "id": "T-070",
  "name": "GitHub 研究仓库展示与协作完善",
  "version": "v1",
  "mode": "lightweight",
  "status": "blocked",
  "goal": "完善private GitHub仓库的双语介绍、协作入口、自动检查及研究材料导航，并核对远端。",
  "scope": "英文主README与完整中文README、About/Topics、贡献/支持/安全/行为规范、Issue/PR模板、软件引用及CI；保留private、现有许可和tag，不改论文主张/正文/作者、不承诺发表信息、不设置强制审批或公开发布。",
  "acceptance": "两种语言的能力/命令/数字和局限一致，引用不虚构论文信息，协作文件/YAML/CFF/文档检查通过，CI实际运行结果可核对；GitHub默认分支和About更新、private保持。",
  "execution_ref": "TODO.md#t-070",
  "acceptance_evidence": [
    "双语项目介绍、About/11项Topics、论文导航、CFF软件引用、贡献/安全/支持/行为规范、Issue/PR模板、Linguist分类、5个领域标签、Dependabot与两套CI已发布默认main，仓库PRIVATE回读确认；GitHub社区文件完整度100%。",
    "本地make check、GitHub YAML/CFF官方schema、actionlint 1.7.12、GitHub GFM双语渲染与独立目录make paper均通过；不改变论文科研主张或macOS汇总口径。",
    "云端CI未产生测试作业：GitHub UI明确显示账户Actions启动限制（计费/额度），故不能标为CI通过。待账户所有者处理后重跑，Task保留阻塞；其余仓库配置已交付。"
  ],
  "unplanned_reason": "2026-11-16用户要求审阅最新提交、准备论文并全面完善private GitHub；另确认引用信息准备中。"
}
```

上下文r3：T-071已接收并整合两路已提交成果；T-070后续状态以main本文件为唯一权威，原codex/github-research-presentation保留已接收快照，不独立更新同一状态。T-069英文稿及作者信息保留；T-070云端Actions仍受既有账户级限制，未改计费设置。

```tracking-execution
{
  "id": "T-070/E-001",
  "name": "完善私有仓库的双语介绍、协作入口与自动检查",
  "task_id": "T-070",
  "task_version": "v1",
  "status": "paused",
  "authorization": "用户明确要求GitHub上所有相关入口完善，不仅README，保持private；署名与论文信息准备中。",
  "scope": "英文主README与完整中文README、About/Topics、贡献/支持/安全/行为规范、Issue/PR模板、软件引用及CI；保留private、现有许可和tag，不改论文主张/正文/作者、不承诺发表信息、不设置强制审批或公开发布。",
  "steps": [],
  "evidence": [
    "只读核对GitHub为PRIVATE且有admin权限；近期提交至0cfd758主要为论文写作、图表及文献构建。",
    "主工作区存在T069 DeepL英文稿未提交改动；本任务在已附加worktree的codex/github-research-presentation分支独立写入，基线0cfd758，不提交/覆盖主工作区WIP。",
    "主工作区T069未提交翻译/作者/构建改动保留；本分支基于0cfd758独立修改并非强制快进推送main，不提交对方WIP。",
    "检查新增根Markdown与.github导航，修正2处跨私有相邻仓库链接为保留原定位信息的外部工件说明；没有放宽仓库内链接校验。",
    "About/Topics/private通过API核对；GitHub GFM实际渲染两语标题/表格/Mermaid入口；CFF通过官方1.2 schema、工作流经actionlint校验。本地make check和独立cache make paper通过。",
    "远端默认main eea322e已包含展示/协作/工作流；社区文件完整度100%，CFF和中文README均在默认分支；已有license/tag/权限不动。",
    "初次job级runner上下文错误已修正。后续运行43750655874启动失败，浏览器仅只读查看到GitHub账户Actions计费/额度阻塞；未操作计费或支付，不伪报云端测试通过。"
  ],
  "stop_reason": "仓库配置及默认分支发布完成；云端CI因账户Actions启动限制无法开始测试，等待账户所有者处理后继续验证。"
}
```

<a id="t-044"></a>
### T-044 · Windows 未通过入口泛化评估

```tracking-task
{
  "id": "T-044",
  "name": "Windows 未通过入口泛化评估",
  "version": "v2",
  "mode": "lightweight",
  "status": "in_progress",
  "goal": "按Windows实际引擎与入口继续九款未通过项的通用注入验证，取得页面执行证据或可复现入口缺口。",
  "scope": "复用协议、宿主和资源格式策略；允许自有派生副本与已确认测试进程操作，不新增品牌执行分支、不改原安装和系统安全策略、不终止原有用户实例。",
  "acceptance": "九款均按实际进程/引擎/入口完成新验证；页面成功须覆盖注入、更新和清理；无法通过保留原始证据及缺口，通用工具完成相应回归。",
  "execution_ref": "TODO.md#t-044",
  "acceptance_evidence": [
    "E001完成只读Windows宿主分类，旧v1定义与run保留。",
    "E002按Windows实际入口新增QQ、WorkBuddy、Figma、抖音、Cocos Creator五款页面通过，累计34/38；其余4款的启动/原生窗口/CEF入口缺口见validation/windows-retries.md，Task继续保留。",
    "E003网易云音乐3.1.38：用户正常退出后，标准CEF开关在新主进程开放回环CDP；现有Windows Rust CLI完成JS/CSS/HTML、交互、更新、清理，累计35/38，剩3项。"
  ],
  "unplanned_reason": "T043全清单测试完成，但9项尚无页面成功，需要保留后续跟进入口。",
  "previous_versions": [
    "v1"
  ]
}
```




[宿主核查](validation/windows-host-technology.md) · [v1执行与定义](archive/T-044-hosts.md#t-044) · [v2复验报告](validation/windows-retries.md)。

恢复上下文 r3：本窗口为Parent，权威写入在本地main；初次起始a586804；E003续接f96dfbb后的本地main。T-044@v2本轮E-002为九项复验，不等于九项全部兼容。剩余4项从复验报告及仓库外原始run恢复；不重复旧成功、不转为品牌适配、不改原安装/安全策略，不自动登录。E003已获正常重启授权，用户手动退出后，标准CEF参数启动的新实例通过完整Windows CLI/CDP页面合同；累计35/38，剩余优酷/百度网盘/企业微信。当前原数据实例保持运行，证据见windows-retries的E003收口。Windows验证checkout按Git接收源码；其他旧平台工作树未接收或改写。

## E-002 · Windows 通用入口复验

```tracking-execution
{
  "id": "T-044/E-002",
  "name": "九款 Windows 通用入口复验",
  "task_id": "T-044",
  "task_version": "v2",
  "status": "succeeded",
  "authorization": "2026-11-13用户明确要求继续九款，按Windows实际引擎/入口且不得每App独立适配；既有授权允许仅结束核验过的本轮测试进程。",
  "scope": "复用协议、宿主和资源格式策略；允许自有派生副本与已确认测试进程操作，不新增品牌执行分支、不改原安装和系统安全策略、不终止原有用户实例。",
  "steps": [],
  "evidence": [
    "完成九项Windows新尝试；共享x64 V8/Bootstrap取得QQ主页面、WorkBuddy页面和Figma登录页证据；同一ASAR引导抖音通过，Cocos正确空项目配置走现有CLI/CDP通过。",
    "其余4项未计页面成功：优酷Widevine404（包内CDM配置仍失败）；企业微信原生登录/百度原生欢迎且本轮未加载CEF；网易云新进程在关闭前已退出、退出码时间未知。原始失败与Unknown不改写/不自动重放。",
    "make check、独立Electron及7项Windows格式测试通过；Win x64/ia32 Node ABI夹具通过；后端/共享runtime未按品牌复制，代码检索未发现测试实现中的应用名分支。报告validation/windows-retries.md。",
    "按既有授权核对PID、exe及创建世代结束4个已确认测试进程；挂起的自有CEF测试控制进程另按精确启动记录收尾，原有QQ/WorkBuddy/Figma实例保留。Git拆分提交6c3b09c至本条所在提交；运行数据与密钥位于仓库外。"
  ],
  "stop_reason": ""
}
```

## E-003 · 网易云音乐复验

```tracking-execution
{
  "id": "T-044/E-003",
  "name": "网易云音乐启动与页面入口复验",
  "task_id": "T-044",
  "task_version": "v2",
  "status": "succeeded",
  "authorization": "2026-11-13用户明确要求网易云音乐再次执行；沿用T044不改系统安全策略/不强停已有会话/不按品牌写执行分支边界。 用户随后明确允许正常退出并重启网易云；仅此App的正常重启获批，仍不强制结束现有用户会话。",
  "scope": "核对当前Windows网易云运行状态与实际启动入口，重试通用Web接入并验证页面执行；保留旧失败记录。",
  "steps": [],
  "evidence": [
    "当前安装3.1.38.205386；初次核查有原始基线内自动更新安装器47408/47756，未终止它们。后续安装器已不在运行，观测到新的活动主实例及GPU/renderer。",
    "正常Shell启动取得退出码0；活动主进程62312、renderer60828实际加载CEF91.2.2/Chromium91.0.4472.169，均存活；当前没有调试监听。只凭原先快速退出不能判为Web不支持。",
    "只读CEF上下文观察请求超时，未取得结果，未执行页面JS/HTML/CSS载荷；后续核查App/renderer仍存活，无自有观察控制进程。",
    "原始新证据windows-validation-20261112/t044-cloud-retry-inventory-2602d7ab.json、t044-cloud-running-engine-1433212d.json、cloudmusic-e003/cloud-shell-launch-86536960b4.json等；旧run不覆盖。",
    "用户批准正常退出重启；远程普通关闭只缩到托盘，未强制终止。用户随后明确回复已退出，确认旧主实例结束后才启动新实例。",
    "以原数据和正确cwd经ShellExecute启动，仅添加标准CEF remote-debugging-port/address；主进程45412在127.0.0.1:58887提供真实orpheus://orpheus/pub/app.html页面，无需Frida或应用专用后端。",
    "Windows本机Rust CLI完整html-string合同通过：applied、按钮0→1、updated且节点不重复、removed后节点0；cleaned=true。新证据cloudmusic-e003/result-final.json和t044-cloud-verify-native-5198300e.json。",
    "App保持运行，回环地址已核对；原exe SHA256与接入前一致。没有修改产品源码、安装包、沙箱或系统安全策略；只同步文档并检查链接/任务/diff，旧失败run保留。"
  ],
  "stop_reason": ""
}
```

<a id="t-034"></a>
### T-034 · 阿里云盘 pre-runtime 入口核查

```tracking-task
{
  "id": "T-034",
  "name": "阿里云盘 pre-runtime 入口核查",
  "version": "v1",
  "mode": "lightweight",
  "status": "blocked",
  "goal": "承接用户经侧聊转交的要求，将pre-runtime纳入阿里云盘注入候选，核查原包实际提供的早期扩展接口及其可执行条件。",
  "scope": "只读核对安装包入口/扩展元数据与厂商文档；仅在明确提供受支持入口时执行验证。已有失败/保护条件保留，不修改签名、fuse、完整性校验或系统保护，不访问内部私有仓库或账号凭据。",
  "acceptance": "候选按实际入口/依赖/证据分类；找到受支持入口则验证正确上下文和可观察结果，否则明确缺少的接口或调试构建，不将自有fixture、云API或内部preload当原客户端注入成功。",
  "execution_ref": "TODO.md#t-034",
  "acceptance_evidence": [
    "已核查当前6.9.3的package入口、三个内部preload封装、内置syncapp清单、官方三方应用/Beta/PDS资料及公开菜单；未取得外部早期代码加载合同，不计新实机注入成功。",
    "用户已回复暂时没有官方开发/调试构建或桌面扩展文档；当前缺少可执行的目标侧入口，实机验证未完成。",
    "继续核查发现旧启动观察缺口，新增只读观察器与7项边界/交接测试；3次阿里6.9.3新观察取得监听、原始HTTP及线程栈证据，但仍无可注入页面。当时窗口核查被Mac锁定阻止。",
    "用户手动解锁后补做实际菜单/偏好设置检查及第4次无载荷观察；CDP启动约0.706秒退出0，完整45秒未取得页面。普通首页已恢复；阻塞仍为缺少可执行的目标侧入口，退出根因未确认，不再以桌面锁定作为当前阻塞。"
  ],
  "unplanned_reason": "用户在侧聊要求继续寻找阿里云盘方案，将pre-runtime也纳入候选，并明确由主窗口推进。"
}
```

<a id="t-034-execution"></a>
```tracking-execution
{
  "id": "T-034/E-001",
  "name": "早期入口与适用条件核对",
  "task_id": "T-034",
  "task_version": "v1",
  "status": "paused",
  "authorization": "侧聊转交用户原话：继续找寻阿里云的注入方案，当前runtime也可考虑pre-runtime，并让主窗口推进。此前正常退出测试授权保留；本轮先只读查找明确扩展入口。 用户再次明确要求继续深入找寻阿里云盘方案。",
  "scope": "只读核对安装包入口/扩展元数据与厂商文档；仅在明确提供受支持入口时执行验证。已有失败/保护条件保留，不修改签名、fuse、完整性校验或系统保护，不访问内部私有仓库或账号凭据。",
  "steps": [],
  "evidence": [
    "基线abbf9da，工作区干净；当前aDrive6.9.3，包main为dist/main/index.js，URL scheme smartdrive；旧runtime/Gadget/attach失败保持。",
    "包内preload封装分别引用mainWindow/authWindow/transferWindow.compiled.jsc及bytenode；主入口引用index.compiled.jsc。仅文件/引用证据，未执行、解码或替换这些载荷。",
    "plugin.json为内置syncapp0.2.16生命周期清单，不视为任意第三方页面代码接口，未执行其中命令。官方三方应用介绍的是Open接口客户端；Beta和PDS开发者版不冒充个人客户端调试构建。",
    "用户明确回复暂时没有官方开发/调试构建或客户端扩展文档；没有进行新注入试验。",
    "CUA核对帮助菜单及应用菜单；未见明确调试入口。偏好设置点击后未出现设置页，不将未读取内容伪报为已排除。最终仍为普通主页，未改设置或发送咨询。",
    "本轮只读核查及文档更新；pre-runtime进入候选范围但不自动绕过已观察的加载/附加/完整性保护。未修改应用包、签名、fuse、系统保护或产品代码。",
    "补充只读核对macOS外部接口元数据：未声明文档类型/脚本字典或AppleScript开关，未见sdef资源；smartdrive URL scheme存在但无早期代码注册合同。",
    "自审保持静态证据/实机执行分开；doc-check、tracking-check、diff-check通过，execution_ref按Task锚点规范修正。任务仍受阻于实际外部入口，没有修改目标保护来制造成功。",
    "续接82018ad，工作区干净。新核查重点为既有CDP监听后退出的启动工具/环境证据，以及已安装包完整性；只读子Agent adrive_harness_audit审查缓存工具与旧run，Parent唯一写入。当前进程环境未设置所检查的代理/Node/DYLD变量，不能回填旧run环境。",
    "并行只读审查确认旧主PID退出0早于清理，但旧工具会马上清子树且丢弃原始HTTP/过滤理由，不能排除交接。新增观察器不提交载荷、不自动清子树，记录进程世代/当前父子关系/原始回执及时间。",
    "review修复当前父PID/组快照、HTTP总截止与HTTPError保存、响应归属时序及僵尸计数；新工具7项边界和真实HTTP进程交接fixture测试通过。实际App兼容性不由fixture替代。",
    "startup-observe-hhw0kuu3：隔离HOME，主PID41886及监听49303保持45秒，14次HTTP总截止。约150秒后的sample证明该PID当时主线程在SecKeychainAddGenericPassword/AuthorizationCopyRights同步等待，不推定全过程或密码写入成功。",
    "startup-observe-s_gnv4xu：系统HOME＋临时Chromium profile（部分隔离）取得一次HTTP200/空列表，随后约1.09秒主PID43244退出0；归属复核未确认，无载荷。startup-observe-ci2dr51u无调试参数持续30秒，PID44943与renderer存在，后续sample见正常NSApplication事件循环。",
    "当前190条封存资源（141hash2、10nested cdhash、39symlink）未发现差异；codesign明确列出4个额外同步运行文件，未读取内容、移动或删除；不将其直接当CDP退出根因。环境当前无代理env，系统代理存在但127.0.0.1绕过true，不回填旧run。",
    "主入口编译资源仅作指定字面计数，openDevTools/toggleDevTools各1次；无控制流解码或调用，不当作可用接口。未知wss9227只作未识别服务线索，没有盲发操作。",
    "CUA核对界面时报告Mac已锁定且自动解锁失败，已请用户手动解锁；此前钥匙串窗口状态问题仍未得到确认。未用其他渠道绕过锁屏或代替用户操作系统授权。",
    "PID44943经独立profile/启动世代复核后仅SIGTERM结束主进程；最终同世代测试进程无残留，未自动清整树或结束原同步服务。原普通窗口恢复等待解锁，不能称已恢复。",
    "make check通过18 Rust/5 Web/8 WK/3 Gecko HTML；make test-frida共38项通过。两位只读子Agent与Parent完成诊断/review，Source/文档同步；代码仍只是可选观察工具，没有新增正式注入后端或成功载荷。",
    "用户回复钥匙串窗口没注意到（保留未知），并明确已手动解锁继续。续接d210ec9，工作区干净，当前aDrive6.9.3无主进程；先恢复普通窗口和核对实际公开菜单入口。",
    "解锁后CUA恢复普通首页；修改/Window菜单未见开发者工具，首页标题右键未见上下文菜单。此次实际打开偏好设置，所见普通锁定/隐私/传输/更新/账号项无调试或外部扩展入口；未改选项，返回首页。仅覆盖所查界面。",
    "按已有授权正常退出普通实例后，startup-observe-yudc69f4以system HOME/新Chromium profile/CDP运行45秒；PID74757约0.706秒被观察到退出0，约1.055秒跟踪集合为空。日志有DevTools listening但未捕获可探测监听，HTTP回执0，未提交载荷；不能写成超时、空列表或从未监听。",
    "只读子Agent adrive_harness_audit复核新旧run主程序SHA/启动逻辑一致，观察器整体源码不同；旧桌面状态及共享系统状态未控制，不作严格锁屏对照。新run无sample，不能套用旧钥匙串等待栈。匹配request哈希的源码已保存source快照。",
    "最终正常恢复普通实例PID75682，CUA确认首页/最近看过，进程无本次profile或调试参数；新run跟踪过的4个PID同世代无残留，未发送终止信号或清同步服务，restoration-check.json已保存。没有确认新的系统授权提示。",
    "本次只同步实测记录与任务上下文，无产品/观察器代码变更。Parent自审与只读子Agent文档review完成，修正矩阵中初次静态阶段的时态歧义；doc-check、tracking-check及diff-check通过，不重复外推既有测试为注入成功。"
  ],
  "stop_reason": "已排除当前桌面锁定这一操作阻塞，但仍没有可执行的外部早期接口或稳定CDP页面；根因证据不足。获得新的明确目标侧入口/调试构建或能解释启动退出的诊断证据后恢复，不重复请求用户已表示没有的资料，也不改保护制造成功。"
}
```

上下文r6：Parent/main，single-writer，续接d210ec9。用户已解锁，菜单/偏好设置及新run yudc69f4完成，普通首页已恢复；钥匙串提示状态与快速退出根因仍未知。当前缺口是可执行的目标侧入口，任务保持受阻，不计注入成功。证据见[启动条件复核](compatibility/adrive-startup.md)；仅有新的入口或诊断证据时继续，不调用未确认服务、修改保护或重放Unknown。

## 最近完成的验证

<a id="t-046"></a>
### T-046 · 通用 CLI 与 Windows 验证里程碑发布

```tracking-task
{
  "id": "T-046",
  "name": "通用 CLI 与 Windows 验证里程碑发布",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "为当前通用CLI与Windows验证进展创建可追溯注解式里程碑tag，并推送和核对远端。",
  "scope": "新增milestone/generic-cli-windows-validation，发布覆盖CLI prepare/dev、Windows35/38及剩余3项边界；仅推送新tag，不移动旧tag、不改CLI版本、不创建安装包或GitHub Release。",
  "acceptance": "注解tag指向含交付映射的本轮里程碑提交；远端tag对象与解引用SHA均与本地一致，已有tag保持不变。",
  "execution_ref": "archive/T-046.md#t-046",
  "acceptance_evidence": [
    "注解tag milestone/generic-cli-windows-validation，对象b4e82e3955c1d947cf583f903fc1881221c8033e，精确指向aaa254a3ddc0d48ff61bc3109cd2b736b45eb36e；含当前CLI/Windows35/38边界与发布映射。",
    "显式推送此tag成功；ls-remote回读注解对象和解引用SHA均与本地相同，原两个tag对象未改变。",
    "Parent核对本地工作区干净、能力/证据基线134cf12、CLI/Windows正式与实验范围及剩余3项；本轮仅元数据变更，doc-check/tracking-check/diff-check通过。"
  ],
  "unplanned_reason": "2026-11-13用户明确要求给当前通用CLI和Windows验证进展打里程碑tag并推远端。"
}
```

[执行与远端核对](archive/T-046.md#t-046)。

<a id="t-045"></a>
### T-045 · Windows 豆包双插件应用

```tracking-task
{
  "id": "T-045",
  "name": "Windows 豆包双插件应用",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "将独立插件库的豆包快捷窗口多主题及论文阅读悬浮聊天应用到win-lenovo豆包，保留可用效果。",
  "scope": "使用现有外部Bundle和CLI，分别定位launcher/chat页面；不复制插件进入Injectra源码、不发消息/读聊天、不改安装包；需要退出已有用户实例时另确认。",
  "acceptance": "两个Bundle在正确Windows页面完成注入并检查插件状态；论文未打开时明确区分等待PDF与实际阅读效果；目标会话保持可供用户查看。",
  "execution_ref": "archive/T-045.md#t-045",
  "acceptance_evidence": [
    "外部插件库eeaca6a的两个发布Bundle已部署且SHA256一致；用户授权后通过托盘退出菜单正常结束旧豆包，原数据目录重启，调试监听仅127.0.0.1。",
    "Windows本机Rust CLI分别向launcher/chat精确目标返回applied；主题0.1.1为sakura，背景rgb(255,241,246)/正文rgb(74,36,53)；论文0.3.0为waiting-for-pdf，未验证PDF实际布局。",
    "插件与豆包会话保留供用户查看；未改注入器/插件源码、不读聊天/凭据、不发送消息。单项证据compatibility/doubao.md，原始结果doubao-windows/apply-53d21ba01182/verification-final.json。"
  ],
  "unplanned_reason": "用户在Windows复验后要求实际应用已有豆包两插件。"
}
```

[执行与恢复记录](archive/T-045.md#t-045)。

<a id="t-043"></a>
### T-043 · Windows 全应用注入复验

[恢复上下文 r1](tasks/T-043-context.md) · [逐应用证据](validation/windows-apps.md)。

```tracking-task
{
  "id": "T-043",
  "name": "Windows 全应用注入复验",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "对照Mac已尝试的全部应用，在Windows逐款复验接入、共享Web注入合同及适用的后续路线；FlowUs、Manus跳过。",
  "scope": "Windows已安装应用盘点、runtime/CDP及后续Gadget/pre-runtime/Gecko候选；Mac编写工具，远端构建/运行，独立测试实例，既有用户进程保留；控制端与目标平台分别标明。",
  "acceptance": "Mac应用清单逐项有Windows结果/缺项，区分安装/入口/真实注入；支持页面覆盖JS/CSS/HTML、交互、更新、清理；其他路线条件/失败及剩余工作明确，不能只测CDP就宣称全部路线完成。",
  "execution_ref": "archive/T-043.md#t-043",
  "acceptance_evidence": [
    "38安装项全清单尝试完成，29页面正向、9本轮未取得页面；2用户跳过、4原Mac静态候选另列。完整方法、限制及raw run在validation/windows-apps.md。",
    "Windows原生CLI编译及选定合同单测/doc tests通过；Gecko/Hook/pre-runtime相互区分，不外推全平台全App兼容。安装与入口错误纠正，旧run和源应用保留。"
  ],
  "unplanned_reason": "用户明确要求Windows重走Mac全部应用注入流程。"
}
```

[关闭执行与证据](archive/T-043.md#t-043)。


<a id="t-042"></a>
### T-042 · 嵌套归档与不透明入口接入

```tracking-task
{
  "id": "T-042",
  "name": "嵌套归档与不透明入口接入",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "修复通用准备器将Electron嵌套ASAR入口当磁盘文件的覆盖缺口，以通用包入口委托策略接入QQ并验证CLI dev。",
  "scope": "只改通用入口解析/策略及测试文档；保留原入口字节及原加载器，验证主模块身份/解析；派生副本、QQ已批准shared数据模式，不动原实例/原包。",
  "acceptance": "普通目录/ASAR回归、嵌套归档与委托夹具保持main身份/相对解析；QQ真实主页面至少两版JS/CSS/HTML动态更新并验证包摘要；不增加品牌判断。",
  "execution_ref": "archive/T-042.md#t-042",
  "acceptance_evidence": [
    "5846000通用入口/框架元数据覆盖修复；QQ实际主页面三条Executed、两版增量与清理通过，原归档字节不改。",
    "make check、3项入口单测、三布局真实Electron回归及文档/任务/差异检查通过；没有品牌分支，旧失败结果保留。"
  ],
  "unplanned_reason": "用户指出QQ已有成功路线，新准备器方法覆盖不完整，要求继续纠正。"
}
```

[执行与验收](archive/T-042.md#t-042)。


<a id="t-040"></a>
### T-040 · 通用注入核心与 pre-runtime dev 模式

```tracking-task
{
  "id": "T-040",
  "name": "通用注入核心与 pre-runtime dev 模式",
  "version": "v3",
  "mode": "lightweight",
  "status": "done",
  "goal": "选择性撤回平台化代码，保留通用runtime并交付CLI prepare/dev页面增量闭环。",
  "scope": "删除工作台、Studio、完整workspace及启动增量/导出；提取Bootstrap、最小来源及执行正确性；精简准备器，只改派生副本，保留历史和运行数据。",
  "acceptance": "runtime合同回归；目录/ASAR准备、CLI动态更新、多目标/Unknown正确性；QQ/Claude优先实机验证并如实限定证据；无平台依赖。",
  "execution_ref": "archive/T-040-cli.md#t-040",
  "acceptance_evidence": [
    "CLI平台删减与prepare/dev闭环完成，代码提交a403982、323db43；源代码与历史WIP有仓库外恢复快照。",
    "make check、Electron、双布局dev、多窗口/错误/Unknown合同、core/native独立构建及Frida回归通过。Claude本地页面实机通过，QQ Unsupported及console限制如实记录。"
  ],
  "unplanned_reason": "用户纠正查看平台含义：先注入少量dev代码，在原App看实时更新，先更新文档。",
  "previous_versions": [
    "v1",
    "v2"
  ]
}
```



[当前范围](architecture/dev-mode.md) · [v1历史与退役执行](archive/T-040.md#t-040)。v3已按用户批准的CLI收敛计划完成；[v2定义](archive/T-040-v2.md)保留。

[CLI收敛执行与证据](archive/T-040-cli.md#t-040)。


<a id="t-041"></a>
### T-041 · 通用注入 dev 模式文档校准

```tracking-task
{
  "id": "T-041",
  "name": "通用注入 dev 模式文档校准",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "将文档和任务导航统一到用户澄清的通用注入核心与轻量dev模式，消除工作台仍在推进的误导。",
  "scope": "仅更新项目文档、任务/决策及当前工作树的只读同步；保留旧方案与代码事实，不删除/回退代码，不恢复实现，不操作应用进程。",
  "acceptance": "README/项目约束/需求/架构/开发/代码地图/实施专题/任务索引一致；T040版本迁移有历史映射；文档与任务校验、只读review通过。",
  "execution_ref": "archive/T-041.md#t-041",
  "acceptance_evidence": [
    "当前范围、入口导航与旧版实现状态已统一；T-040 v1定义/证据保留，v2待启动；未修改源码或应用。",
    "只读review发现的早期runtime措辞及旧Parent执行指令已修正；main与server文档检查、tracking-check、diff-check通过。"
  ],
  "unplanned_reason": "用户明确要求先更新之前的相关文档。"
}
```

[执行与文档校验证据](archive/T-041.md#t-041)。

<a id="t-039"></a>
### T-039 · QQ粉紫主题资源补丁预览

```tracking-task
{
  "id": "T-039",
  "name": "QQ粉紫主题资源补丁预览",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "通过已验证QQ资源启动加载方案，在真实窗口预览粉色与紫色主题并保留用户查看。",
  "scope": "按用户明确选择共享原QQ数据；仅新副本资源/签名与插件装配，不修改原安装包，不登录/发送消息；主题源码归独立插件库。",
  "acceptance": "两套主题在真实QQ窗口可见且可切换，原生输入/DOM保持，恢复入口有效；测试/review/文档与提交完成。",
  "execution_ref": "archive/T-039.md#t-039",
  "acceptance_evidence": [
    "User-approved shared-data QQ preview loaded qq.pastel-themes from the separate plugin library. Native UI switching through pink, purple, original and pink passed, computed colors matched, editorSame=true. Preview PID90625 retained.",
    "Plugin commit eeaca6a contains the implementation and bundle. Plugin make check: 47 passed, including 5 QQ tests. Injectra make check and 10 resource-patch tests passed; documentation, tracking and diff reviewed."
  ],
  "unplanned_reason": "用户明确共享QQ原数据无问题，要求直接实现QQ粉色、紫色主题。"
}
```

[Live validation](compatibility/qq-pastel-preview.md) - [Execution](archive/T-039.md#t-039)

<a id="t-038"></a>
### T-038 · QQ腾讯视频Claude资源补丁顺序验证

```tracking-task
{
  "id": "T-038",
  "name": "QQ腾讯视频Claude资源补丁顺序验证",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "按QQ、腾讯视频、Claude顺序在独立测试副本验证资源补丁代码执行。",
  "scope": "仅测试副本资源/入口/完整性和本地签名修改；保留原安装包、原会话与账号数据；不登录或发送消息，不修改系统保护；不注册正式后端。",
  "acceptance": "按指定顺序三款均有真实页面执行与交互证据，或准确保存实际失败阶段；保留原会话，核对原关键文件及测试副本进程。",
  "execution_ref": "archive/T-038.md#t-038",
  "acceptance_evidence": [
    "按QQ、腾讯视频、Claude串行实际启动：QQ旧共享账号条件下取得标记/回执，不能计隔离通过；新身份去共享组后Mach注册拒绝。腾讯动画HTML候选首页未见执行。Claude重建完整性摘要后本地壳页面代码、可读中文及交互回执通过。",
    "原QQ69729、Claude680原世代保留，所有已测副本路径无进程；各run原关键文件哈希一致，不代表整个原包逐字节核对。Claude远端内容与登录业务未测试；没有操作账号/消息。",
    "make check及9项资源工具测试通过，Parent自审源/证据/范围，文档/代码地图同步；失败及旧原始结果保留。"
  ],
  "unplanned_reason": "用户明确要求推送里程碑tag，再依次尝试三款应用。"
}
```

[执行与证据](archive/T-038.md#t-038)

<a id="t-037"></a>
### T-037 · pre-runtime 里程碑远端同步

```tracking-task
{
  "id": "T-037",
  "name": "pre-runtime 里程碑远端同步",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "推送并核验已创建的里程碑tag。",
  "scope": "仅推送milestone/pre-runtime-web-injection并核对远端对象；不移动tag。",
  "acceptance": "远端注解tag对象及解引用提交与本地一致。",
  "execution_ref": "archive/T-037.md#t-037",
  "acceptance_evidence": [
    "git push成功；远端tag对象75a2b12a867866c2f07db648e5337127cd3fe732，解引用04a7013e2b3838db635850daf12edb810f064b36，与本地一致。"
  ],
  "unplanned_reason": "用户明确要求推送里程碑tag，再依次尝试三款应用。"
}
```

[执行与证据](archive/T-037.md#t-037)

<a id="t-036"></a>
### T-036 · pre-runtime 里程碑标记

```tracking-task
{
  "id": "T-036",
  "name": "pre-runtime 里程碑标记",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "为已验证的pre-runtime真实页面资源补丁进展创建可追溯的Git里程碑tag。",
  "scope": "仅创建本地注解tag并同步映射和执行记录；标记已验证提交04a7013，不改变版本号、历史tag或远端。",
  "acceptance": "tag对象为annotated tag，解引用精确指向04a7013e2b3838db635850daf12edb810f064b36，说明载明实机证据及适用条件。",
  "execution_ref": "archive/T-036.md#t-036",
  "acceptance_evidence": [
    "milestone/pre-runtime-web-injection已创建，tag对象75a2b12a867866c2f07db648e5337127cd3fe732，类型tag；解引用04a7013e2b3838db635850daf12edb810f064b36。",
    "注解说明真实本地登录壳页面loaded/applied/clicked证据、资源补丁/测试签名及隔离条件；本地标记，不外推生产后端或登录后业务。"
  ],
  "unplanned_reason": "用户明确将此进展认定为pre-runtime里程碑并要求打tag。"
}
```

[里程碑映射](releases.md) · [执行记录](archive/T-036.md#t-036)

<a id="t-035"></a>
### T-035 · 阿里云盘测试副本资源补丁验证

```tracking-task
{
  "id": "T-035",
  "name": "阿里云盘测试副本资源补丁验证",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "在阿里云盘独立测试副本中修改真实Web资源入口，验证最小bootstrap能否在原应用页面执行。",
  "scope": "仅修改仓库外新建测试副本的Web资源、ASAR完整性信息和测试签名；独立HOME/profile，保留原安装包与账号数据；不修改fuse或系统保护，不注册正式产品后端，不加载实际插件。",
  "acceptance": "真实副本页面正常打开，页面脚本返回本次随机标识并出现匹配的可见标记；或明确保存失败阶段及诊断。记录副本/原包身份、修改差异、签名与资源完整性条件；原普通窗口恢复，历史CDP/Frida结果不覆盖。",
  "execution_ref": "archive/T-035.md#t-035",
  "acceptance_evidence": [
    "独立测试副本adrive-resource-ZEAQtJ在真实dist/index.html#/login执行共享runtime，nonce bfde8ea5e8；loaded/applied/clicked回执及CUA可见标记、按钮交互通过；范围限本地登录壳页面。",
    "保留ASAR/fuses/runtime，副本采用ad-hoc签名并显式允许本地库加载；独立目录与测试钥匙串。失败run及原始回执保留，新增严格顺序/颜色检查复核通过。原普通首页80119恢复，副本进程无残留，4个原关键文件哈希一致。",
    "make check及5项可选资源补丁单测通过，原包保护/归档写入完成/回执误报review问题已修正，文档与代码地图同步；无正式产品后端或插件变更。"
  ],
  "unplanned_reason": "用户明确要求直接在阿里云盘测试副本实施资源补丁，先验证代码在真实页面执行；与T-034只读外部入口核查分开。"
}
```

[真实页面结果与构建条件](compatibility/adrive-resource-patch.md) · [执行与review](archive/T-035.md#t-035)

<a id="t-033"></a>
### T-033 · 阿里云盘 Frida Gadget 与原生 Hook 验证

```tracking-task
{
  "id": "T-033",
  "name": "阿里云盘 Frida Gadget 与原生 Hook 验证",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "按用户要求改走Frida Gadget/原生Hook，验证阿里云盘原包的接入、原生方法和页面注入，或定位阻止后续阶段的实际条件。",
  "scope": "当前6.9.3、受控新HOME/profile、Gadget required及文档化原生接入；沿用正常退出原实例授权，结束后恢复普通窗口；不修改原包签名、fuse或系统保护。",
  "acceptance": "取得真实身份及原生Hook/页面效果证据，或明确加载/附加前置失败；保留新run与包状态，清理自有实例并恢复普通窗口，不将CDP失败外推到Frida，也不把配置当实际加载成功。",
  "execution_ref": "archive/T-033.md#t-033",
  "acceptance_evidence": [
    "阿里6.9.3真实Gadget required启动未建立监听或身份回执；同一核验测试PID直接Frida attach返回PermissionDeniedError。未进入原生Hook/页面执行，前置失败与静态权限条件分开记录。",
    "临时控制器清理抛错导致初次附加回执未落盘，保留未知；后续单独保存附加结果。核验身份后结束未响应终止的自有测试PID31707，无该profile残留，普通窗口恢复并确认首页导航；文档与任务检查通过。"
  ],
  "unplanned_reason": "用户明确要求阿里云盘尝试Frida Gadget或Hook，不继续只测CDP。"
}
```

[Frida实测](compatibility/adrive-frida.md) · [执行与review](archive/T-033.md#t-033)

<a id="t-032"></a>
### T-032 · macOS 十款应用顺序注入验证

```tracking-task
{
  "id": "T-032",
  "name": "macOS 十款应用顺序注入验证",
  "version": "v1",
  "mode": "staged",
  "status": "done",
  "goal": "按用户指定顺序对10款macOS应用逐一尝试接入与实际runtime注入，获得可复现成功证据或准确的实测阻塞及下一条件。",
  "scope": "当前Apple Silicon本机应用；依次尝试CDP/已有原生页面后端，检查独立目录与已有实例；只用自有临时UI载荷验证并清理，保留账号/同步任务/文档；不改签名/系统保护或未授权结束已有进程。",
  "acceptance": "10款均有当前版本/路径/运行条件及明确尝试结果；取得页面时验证JS/CSS/HTML或如实记录HTML限制、交互、更新与清理；失败保留阶段与证据，不以肯定可行为结论，不以仅识别宿主作为注入成功。",
  "execution_ref": "archive/T-032.md#t-032",
  "acceptance_evidence": [
    "10款已按顺序实际尝试；百度网盘、千问、Anytype、Xmind、网易云在已测页面上通过；阿里、企业微信、Claude、腾讯视频、QQ接入未完成，逐项限制及证据见报告。",
    "用户QQ/阿里退出授权已落实并恢复普通窗口；阿里恢复时为6.9.3，新增Node/CDP补测未取得页面，Node调试开关关闭有只读证据；旧6.8.0结果保留。",
    "18次新进程实验及原实例接入观察完成，成功项面板清理、测试进程退出、日志备份与签名状态分别核验；任务/文档/差异检查和自审通过，没有将批次完成误报为全部应用支持。"
  ],
  "unplanned_reason": "用户明确要求对旧列表中的10款应用依次实际尝试。"
}
```

[逐应用结果](validation/macos-ten-apps.md) · [执行与review](archive/T-032.md#t-032)

<a id="t-031"></a>
### T-031 · macOS 应用未验清单与硬件范围确认

```tracking-task
{
  "id": "T-031",
  "name": "macOS 应用未验清单与硬件范围确认",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "从原应用列表提取未完成验证的真实应用，并记录用户排除Intel Mac的硬件范围。",
  "scope": "只读核对兼容性表及后续WebKit记录，更新macOS硬件范围与任务/决策；不启动或重试应用，不改变保留的抖音窗口。",
  "acceptance": "14款未进入完整注入验证、2款尝试未取得页面、部分/版本未复验分别列明；后续Easydict单列；Intel硬件从后续范围移除，历史证据不改。",
  "execution_ref": "archive/T-031.md#t-031",
  "acceptance_evidence": [
    "按兼容性表状态列提取14款未验证；微信开发者工具和CocosCreator为尝试未取得页面，Figma部分通过、豆包2.34.6仅入口；后来Easydict接入未完成单列。D-011与README/需求/矩阵记录Apple Silicon硬件范围，Intel不继续列作缺口；无代码或应用操作。"
  ],
  "unplanned_reason": "用户要求回到先前macOS应用列表查看未验证项，并明确不考虑Intel芯片。"
}
```

[清单核对与范围记录](archive/T-031.md#t-031) · [D-011](DECISIONS.md#d-011)

<a id="t-030"></a>
### T-030 · 注入适配复用边界与插件化阈值确认

```tracking-task
{
  "id": "T-030",
  "name": "注入适配复用边界与插件化阈值确认",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "说明当前正式注入实现的复用范围、应用专用部分和Hook预览的实际链路，并记录用户确定的32个适配方案插件化阈值。",
  "scope": "定向只读检查正式后端/Hook测试/预览证据，更新架构与需求决策；不新增插件加载器或改运行代码，不操作保留的抖音窗口。",
  "acceptance": "代码证据支持通用/专用结论；明确方案计数与应用数量区别、第33个方案前插件化要求和当前静态装配状态；文档/任务校验及review通过。",
  "execution_ref": "archive/T-030.md#t-030",
  "acceptance_evidence": [
    "代码核对确认正式核心与后端没有抖音品牌特判，Electron桥接依赖宿主ABI；Hook动态查NSApp类而非硬编码AtomApplication，T-029应用专用协调不作为通用后端。D-010记录用户32方案阈值及第33个方案前插件化要求、计数与验收；文档/任务检查通过。"
  ],
  "unplanned_reason": "用户担忧按应用适配不可持续，并明确将适配方案的插件化阈值设为32。"
}
```

[复用与32方案约束](architecture/boundaries.md) · [执行与review](archive/T-030.md#t-030)

<a id="t-029"></a>
### T-029 · 抖音 Hook 触发主题插件预览

```tracking-task
{
  "id": "T-029",
  "name": "抖音 Hook 触发主题插件预览",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "让实际宿主Hook捕获调用后，通过已有Web桥接向抖音真实页面加载原图片/紫雾主题插件，并保留可见窗口供用户查看。",
  "scope": "显式启动独立实例、读取独立插件库原bundle、Hook触发后CLI apply及页面主题切换/读回；不复制插件到Injectra，不改原包或系统设置，完成后保留窗口。",
  "acceptance": "真实Hook调用非零后才apply，原插件和目标身份可定位；页面读回主题生效并提供可见窗口或截图；明确Hook/页面桥接分工与会话保留。",
  "execution_ref": "archive/T-029.md#t-029",
  "acceptance_evidence": [
    "真实宿主Hook捕获5次调用后触发原主题加载（恢复时6/6完成）。独立实例PID53083的真实精选页面读回0.2.0/purple/enabled/control，原生截图确认紫雾外观及控件，窗口保留用户查看。CLI回执曾Unknown，停止悬挂主导航后原排队注入完成，未重放。"
  ],
  "unplanned_reason": "用户要求用当前Hook方案实际注入之前的抖音主题插件并查看效果。"
}
```

[预览、保留窗口与执行证据](archive/T-029.md#t-029)

<a id="t-028"></a>
### T-028 · 抖音实际宿主方法 Hook 验证

```tracking-task
{
  "id": "T-028",
  "name": "抖音实际宿主方法 Hook 验证",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "验证在抖音原包受控实例内，对真实宿主的Objective-C方法实施保持原行为的分派Hook，并取得应用自然调用及恢复证据。",
  "scope": "Gadget required接入，真实宿主方法的文档化分派API；独立HOME/profile和自有进程，不改原包/系统保护，不读账号或业务载荷；并行只读分析旧optional崩溃，不重跑已知失败。",
  "acceptance": "核对实际接收对象、方法定义类、原IMP所属映像；捕获非探针主动调用的宿主调用，确认原方法继续执行及恢复/卸载；或保存明确新阻塞。禁止把测试对象/主动调用计作自然业务调用，禁止外推Interceptor支持。",
  "execution_ref": "archive/T-028.md#t-028",
  "acceptance_evidence": [
    "抖音8.7原包真实NSApp的AtomApplication.sendEvent:方法分派Hook通过：两次无窗口操作30秒复测为360/360和最终默认命令147/147原调用完成，实际类/原实现Electron Framework/主线程核验一致；恢复、卸载及4个自有进程退出确认，主文件哈希/签名通过。保留初次8秒0调用失败；不外推业务逻辑或Interceptor。make check、31项Frida测试和review通过。"
  ],
  "unplanned_reason": "用户明确要求继续尝试抖音Hook；在原包和系统保护不变的边界内，把已通过的自有对象方法分派验证推进到真实宿主。"
}
```

[真实宿主Hook实测](compatibility/douyin-host-hook.md) · [执行与review](archive/T-028.md#t-028)

<a id="t-027"></a>
### T-027 · 抖音 Frida Interceptor 授权重试

```tracking-task
{
  "id": "T-027",
  "name": "抖音 Frida Interceptor 授权重试",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "在用户可完成系统密码授权的条件下，重试原包外部Frida attach与Gadget optional两条Interceptor路线，区分授权未完成与目标代码签名限制。",
  "scope": "独立home/profile、自有实例，观察系统授权由用户输入密码，纯abs函数Hook不改返回；原包、系统保护与旧证据不变，不接入账号或用户实例。",
  "acceptance": "两条路线有新run及准确阶段结果；若出现授权窗口等待用户完成，记录授权是否实际发生；记录Hook回调/卸载或阻塞、签名/哈希与进程退出，不把缺少授权的失败外推成最终不兼容。",
  "execution_ref": "archive/T-027.md#t-027",
  "acceptance_evidence": [
    "按用户要求重试：抖音外部attach两次PermissionDenied（截止180秒但立即拒绝），首次用户确认没有授权窗口；optional新PID44878在listener前CODESIGNING/Invalid Page。新增自有get-task-allow空进程外部attach及同agent Hook通过，补齐对照证据；4个测试PID退出，3次原包哈希/签名通过。旧run与未唯一定位的历史超时原因保留。"
  ],
  "unplanned_reason": "用户明确要求重试之前失败的两条Hook路线，并指出上次离开时未输入系统授权密码。"
}
```

[复测与授权事实](compatibility/douyin-hook-auth-retry.md) · [执行与review](archive/T-027.md#t-027)

<a id="t-026"></a>
### T-026 · 抖音 Frida Objective-C 分派 Hook 验证

```tracking-task
{
  "id": "T-026",
  "name": "抖音 Frida Objective-C 分派 Hook 验证",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "验证抖音原包在Gadget required模式下能否对专用Objective-C对象实施方法分派Hook，并保持原结果及恢复方法。",
  "scope": "独立home/profile、自有测试对象与实例；复用Gadget接入，新增方法指针Hook；不改应用包、签名、系统保护或业务对象，不重跑旧页面套件。",
  "acceptance": "记录真实进程身份、原方法/Hook调用/恢复证据或明确阻塞阶段；核验主文件哈希、签名与自有进程退出。不得外推为Interceptor、任意业务方法或正式CLI支持。",
  "execution_ref": "archive/T-026.md#t-026",
  "acceptance_evidence": [
    "抖音8.7.0原包内自有NSObject子类hash的IMP Hook实测通过：两次回调，原结果一致，恢复后计数不再增长，agent成功卸载；运行r2hmbl3w和补强清理判定后的eo1jroii保留。主程序哈希/签名通过，自有进程退出；不外推业务方法或Interceptor。make check及26项Frida测试通过。"
  ],
  "unplanned_reason": "用户要求继续推进抖音新Frida Hook方案，前序Interceptor路线受限，测试同保护边界内的方法分派路线。"
}
```

[实机机制验证](compatibility/douyin-objc-hook.md) · [执行与review](archive/T-026.md#t-026)

<a id="t-025"></a>
### T-025 · 抖音 Frida 原生函数 Hook 可行性验证

```tracking-task
{
  "id": "T-025",
  "name": "抖音 Frida 原生函数 Hook 可行性验证",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "验证抖音8.7.0原包的外部Frida attach及Interceptor函数Hook前提、回调和卸载，区别于已验证的Gadget页面桥接。",
  "scope": "签名/进程/可选Frida环境核对，独立home/profile受控实例的无返回值改写Hook测试，证据/文档；不重签名、改系统保护或用户实例，不访问账号和业务数据。",
  "acceptance": "取得真实函数Hook回调和卸载证据，或明确记录阻止进入Hook的实际阶段与错误；身份/签名/自有进程退出确认，不将旧页面注入成功或配置检查当新Hook成功。",
  "execution_ref": "archive/T-025.md#t-025",
  "acceptance_evidence": [
    "新原生Hook可行性验证已执行并得到受限结论：抖音8.7外部attach PermissionDenied；Gadget optional在listener前被CODESIGNING/Invalid Page终止，均未进入Interceptor。自有空进程同agent真实Hook/卸载通过；本轮不交付抖音Hook能力。哈希/签名、退出、证据与回归已核验。"
  ],
  "unplanned_reason": "用户要求抖音测试Frida Hook，并纠正不重复已有方案，要验证新方案。"
}
```

[新方案实测](compatibility/douyin-native-hook.md) · [执行与review](archive/T-025.md#t-025)

<a id="t-024"></a>
### T-024 · Thunderbird HTML 注入实机验证收口

```tracking-task
{
  "id": "T-024",
  "name": "Thunderbird HTML 注入实机验证收口",
  "version": "v1",
  "mode": "lightweight",
  "status": "done",
  "goal": "继续验证Thunderbird159.0主界面HTML注入缺口，以真实页面效果与清理回读闭合可行路线或明确不可行边界。",
  "scope": "独立空配置实例、HTML构造路径诊断与测试适配、原合同回归及证据文档；不新增正式CLI后端、不改安装包/安全设置、不接入邮箱账号。",
  "acceptance": "主界面HTML字符串载荷路线有真实效果与更新/清理证据，或保留无法支持的具体阻塞；旧innerHTML失败与BiDi限制独立保留，不改历史run/冒充全协议支持；自有进程退出，回归/review完成。",
  "execution_ref": "archive/T-024.md#t-024",
  "acceptance_evidence": [
    "测试用DOMParser→importNode装配路线在Thunderbird159.0主界面通过37项检查，gHjeHb及最终DxFYST两次成功；默认合同fjBUxN仍27项成功/原HTML失败/partial。原失败、BiDi限制和正式CLI未适配均保留；5次自有实例已退出，截图已核对；make check和review通过。仅验证阶段收口。"
  ],
  "unplanned_reason": "用户指出截图中的Thunderbird验证未收口，选择先补齐实机验证、正式CLI另行推进。"
}
```

[验收报告](validation/thunderbird-html.md) · [执行与review证据](archive/T-024.md#t-024)

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
  "acceptance_evidence": [
    "兼容性报告覆盖40个候选及方法边界；31条脱敏尝试与原始结果逐条核对，首次16个、最终20个完整通过的口径保留；自审、make doc-check、任务结构检查及git diff --check通过。仅文档与证据摘要变更，本条所在提交为独立本地归档提交。"
  ],
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
  "acceptance_evidence": [
    "定位、责任及当前验收已统一，D-003记录用户纠正；review、make doc-check、任务结构检查与git diff --check通过；仅Markdown变更。"
  ],
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
