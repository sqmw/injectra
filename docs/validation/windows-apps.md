# Windows全应用注入复验

[T-043](../TODO.md#t-043)，2026-11-12持续推进。下表是阶段证据快照，**不是全部路线已完成或整体支持列表**。FlowUs/Manus按用户要求跳过；安装不等于注入。范围按Mac候选与后续Gecko/CodexSwitcher去重对账，遗漏的Windows安装项单列。

## 当前验证方式

Mac编写工具并用现有Rust CLI，经回环SSH转发连接Windows真实桌面App；目前不是Windows原生CLI构建结果。临时交互任务在当前用户桌面启动测试App；只有本次生成报告中的PID/启动身份可请求关闭。原有用户会话保留，未登录、发送消息、同步或播放。未取得CDP不外推Gadget或pre-runtime不可用。

Windows源码主目录尚未同步：Mac项目位于language/rust/code/Injectra，已登记Windows映射为D:\language\rust\code\Injectra；本轮首读验证该Windows目录不存在，不能按旧路径假定代码已部署。

诊断工具先遇到Windows命令长度/标准输入编码和端口预留问题，已在Mac源工具修正；早期工具失败单列，不计为App不支持。端口由Windows操作系统选取，再建立同号回环转发。截图是可选观察，截图超时不等于JS/CSS/HTML失败；受Trusted Types限制时另跑JS创建DOM合同，保持限制可见。

## 阶段快照

| 应用 | 安装版本 | 阶段结果 | 外部run目录 |
| --- | --- | --- | --- |
| Notion | 7.43.0 | 页面合同通过（控制端Mac） | Notion-511b4b2d |
| Logseq | 2.0.2.0 | 页面合同通过（控制端Mac） | Logseq-7cb92d96 |
| Folo | 1.17.0 | CDP未取得页面，其他路线待测 | Folo-4fea06de |
| Anytype | 0.57.7.0 | 页面合同通过（控制端Mac） | Anytype-f5e72031 |
| Xmind | 27.1.5088.202611110744 | 页面合同通过（控制端Mac） | Xmind-ad6f33a3 |
| QQ | 9.9.33.52230-aff854e8 | CDP未取得页面，其他路线待测 | QQ-03bc59c7 |
| Mockitt | 1.6.6 | 页面合同通过（控制端Mac） | Mockitt-7853d3ca |
| SuperProductivity | 19.3.1.0 | 页面合同通过（控制端Mac） | SuperProductivity-d694cb91 |
| CodexSwitcher | 0.2.20 | CDP未取得页面，其他路线待测 | CodexSwitcher-4be2c4e4 |
| WorkBuddy | 5.8.3.0 | CDP未取得页面，其他路线待测 | WorkBuddy-b5d8fbcb |
| Qianwen | 2.3.1.1 | 页面合同未完整通过，待分类 | Qianwen-3cd49078 |
| Antigravity | 2.31.2.0 | 页面合同未完整通过，待分类 | Antigravity-d4901df6 |
| Devin | 3.12.43 | 页面合同未完整通过，待分类 | Devin-6f7a822b |
| Trae | 3.6.13 | 页面合同通过（控制端Mac） | Trae-64918810 |
| TraeCN | 3.4.18 | 页面合同通过（控制端Mac） | TraeCN-9496f1e9 |
| TraeSoloCN | 0.2.2 | 页面合同通过（控制端Mac） | TraeSoloCN-5e6536ad |
| ZCode | 3.17.2.9661 | 页面合同通过（控制端Mac） | ZCode-55d0a0a6 |
| Quark | 7.4.5.1027 | 页面合同未完整通过，待分类 | Quark-54ac3116 |
| Claude | 5.1231.2 | CDP未取得页面，其他路线待测 | Claude-a6d82250 |
| BaiduNetdisk | 8.10.3.102 | CDP未取得页面，其他路线待测 | BaiduNetdisk-0af20472 |
| Youku | 9.2.66.1001 | CDP未取得页面，其他路线待测 | Youku-1091c9bc |
| Bilibili | 1.20.0.5175 | 页面合同通过（控制端Mac） | Bilibili-64981100 |
| aDrive | 6.9.3.1875 | 工具／启动异常，待复核 | aDrive-a92234c9 |
| WeCom | 5.0.13.6005 | 工具／启动异常，待复核 | WeCom-ad0a0ca0 |
| TencentVideo | 11.189.7537.0 | CDP未取得页面，其他路线待测 | TencentVideo-193e3686 |
| Firefox | 160.0 | 待尝试 | — |
| Thunderbird | 160.0 | 待尝试 | — |
| WeixinDevTools | 2.02.0 | 页面合同通过（控制端Mac） | WeixinDevTools-733564d2 |
| Douyin | 8.8.0.0 | CDP未取得页面，其他路线待测 | Douyin-dbc5a055 |
| CocosDashboard | 2.2.2 | 待尝试 | — |
| drawio | 32.6.10.0 | 待尝试 | — |
| Figma | 126.11.8.0 | 待尝试 | — |
| Chrome | 待记录 | 页面合同通过（控制端Mac） | Chrome-27e176c5 |
| Edge | 待记录 | 待尝试 | — |
| Doubao | 待记录 | 待尝试 | — |
| Cursor | 待记录 | 待尝试 | — |
| CloudMusic | 待记录 | 待尝试 | — |
| CocosCreator | 待记录 | 待尝试 | — |
| OBS Studio | 待记录 | not-in-installation-list | — |
| FlowUs | 待记录 | skipped-user | — |
| Manus | 待记录 | skipped-user | — |
| Deskreen | 待记录 | not-in-installation-list | — |
| EasyConnect | 待记录 | mac-inventory-only-network-side-effects-not-started | — |
| ClashVergeRev | 待记录 | existing-network-service-preserved | — |

证据位于仓库外windows-validation-20261112/runs；Windows临时启动/收尾记录位于显式传入的remote-root，各次有独立ID。旧run不覆盖；取新结果需保留失败性质及控制端/目标平台。后续继续Windows本机CLI、Gecko、WebView2及适用Gadget/pre-runtime路线，未完成项保持T-043活动。

## 工具入口

[Windows控制工具](../../scripts/windows-cdp-smoke.py)使用标准库和已有SSH；Python沿用项目Frida venv或等效标准库环境，不安装全局依赖。[页面验证](../../scripts/verify-live.mjs)复用现有CLI/runtime，--dom显式用DOM API创建控件，--no-screenshot只跳过截图，不跳过执行/交互/更新/清理。

```sh
python -B scripts/windows-cdp-smoke.py --host "$WIN_HOST" --user "$WIN_USER" --ssh-port "$WIN_SSH_PORT" --plan "$PLAN_JSON" --output "$EVIDENCE_DIR" --remote-root "$WIN_RUN_ROOT"
```

plan为显式外部JSON数组（name/exe/version/可选args），不在产品核心中按品牌分支。测试不修改安装目录、系统调试配置、现有同步目录或全局环境变量。当前仅是runtime首轮工具，不能将其当成Windows pre-runtime准备器。

工具复核：已用Windows Chrome验证完整HTML合同及--dom/--no-screenshot组合；交互启动、停止标记和桌面会话收尾均有独立run。早期双反斜杠路径和Session0收尾问题已修正，旧自有实例通过交互任务按PID/启动时间/路径核对请求关闭；不强制结束用户进程。源码工具的Python仅标准库，Mac先完成语法、真实对照与make check。

## Windows本机构建与后续验证

已通过Git在仓库外建立source-69ddebc独立验证checkout，HEAD与Mac提交69ddebc一致，不覆盖Syncthing目录。Windows用现有Rust1.96.1/x86_64 MSVC、两路构建完成injectra.exe，Node24.18.0执行页面验证。native-runs将Windows CLI/测试客户端与Mac SSH编排区分；原生CDP实测已有成功，不再把Mac控制端结果当作唯一证据。

后续工具新增--native-root/--native-node/--native-target-dir以在Windows直接执行页面验证；[远程诊断传输](../../scripts/windows_remote.py)只承载测试命令，复用SSH连接，参数与运行数据在仓库外。--native-python仅用于明确标记为测试探针的[Win32 Hook](../../scripts/windows-frida-hook.py)/[agent](../../tests/windows/dispatch-hook.js)，通过owner记录核对PID/创建时间/路径；Hook成功不计为页面注入通过。

[Windows Gecko测试客户端](../../scripts/windows-gecko-verify.mjs)复用原BiDi、Marionette和Thunderbird宿主合同，尚待Windows实测；不注册正式CLI后端。窗口启动参数、临时user.js和子进程环境均由显式外部plan提供，不修改用户全局配置。新测试客户端在Mac完成语法及共享合同检查后经Git部署到Windows验证。

2026-11-12用户补充授权：允许结束经PID、启动时间和路径核验的本轮残留测试进程，初始会话不动。已据初始running快照排除原进程，强制清理证据单独保存在force-owned-*.json；此前只请求正常退出的记录不改写。

## pre-runtime实验准备（尚未等同正式Windows prepare/dev）

新增[受控副本工具](../../tests/windows/prepare-copy.mjs)，按显式包路径处理目录/ASAR及版本化资源根；入口和[共享ASAR重建](../../crates/preparation/helper/archive.mjs)复用原准备器，不按软件品牌分支。[Windows PE完整性](../../tests/windows/pe-integrity.mjs)按[Electron官方格式](https://www.electronjs.org/docs/latest/tutorial/asar-integrity)更新匹配的原头摘要，不关闭fuse；变更PE资源的副本不承诺保留原Authenticode签名，原文件不修改。

[PE32/PE64格式测试](../../tests/windows/pe-integrity.test.mjs)验证只更新匹配记录、错误基线不写、拒绝越界。依赖由[独立锁文件](../../tests/windows/tools/package.json)声明，安装到仓库外；既有runtime CLI不因此要求Node。[可选远程Electron合同](../../crates/bootstrap/tests/remote_live.rs)使用真实Bootstrap/WebBackend和单一Web runtime验证更新与清理，默认ignored，需要显式私有配置及回环SSH桥。

以上工具先在Mac完成语法/编译/格式测试，共享ASAR提取后Mac三类Electron回归通过；Windows实机结果随后单列，不将测试源码或PE往返测试当作应用兼容证明。

截至跨日继续，Windows原生CLI已有21款以上页面正向证据；腾讯视频通过子进程WebView2/Qt回环环境候选也取得页面合同，不能用Mac原CEF失败覆盖Windows新成功。Frida探针的aDrive13/13、Douyin6/6及字节恢复均通过，但目标是指定启动/宿主进程，不宣称相应网页已注入。Firefox的启动器子进程校验调整后仍需复跑。

本检查点review：Parent检查回环地址、源文件保留、hash基线、PE记录匹配及协议/Hook非页面结果的隔离。共享ASAR代码提取后make check和Mac目录/ASAR/嵌套三类回归通过；PE32/PE64序列化与错基线拒绝通过。Windows副本工具仍是实验，需实机确认后才能计入兼容性。

Windows探测还发现@electron/asar 3.4.1的嵌套入口API需要原生路径分隔符；已在共享入口/归档层归一化并增加多层目录用例，避免把真实存在的.vite/build入口误报为缺失。Mac四项入口合同与make check通过，Windows复验继续。

部分Windows包带有虚拟size条目或原本已缺失的unpacked可选文件；完整流重建不能假造这些内容。测试副本工具新增[保留原数据段的增量ASAR写入](../../tests/windows/append-asar.mjs)，只更新声明条目，其他元数据及原字节保留；[独立格式测试](../../tests/windows/append-asar.test.mjs)覆盖虚拟条目、缺失外置文件和原packed数据不变。该路径尚待Windows真实App复验，不回填先前失败。
