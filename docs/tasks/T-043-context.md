# T-043恢复上下文 r1

[Task/Execution唯一状态](../TODO.md#t-043)。Parent/main唯一写入；当前chat工作树是旧platform-studio-server，尚未接收本轮Win检查点。目标仍为Windows全应用复验，不是建设平台或宣称全量支持。

## 已实测

- Windows联想主机通过全局当前事实的首选SSH连接；安装清单32新增/6已有，FlowUs/Manus明确跳过。
- main提交69ddebc建立批量交互CDP测试；61d2a59增加Windows本机CLI、Gecko/Win32 Hook测试；88f8951绑定Gecko owner。当前有未提交改进及可选Bootstrap远程fixture。
- Windows仓库外source-69ddebc为独立Git验证checkout，当前HEAD88f8951；原D:\language\rust\code\Injectra不存在，没有手动覆盖Syncthing目录。产品CLI用既有Rust1.96.1 MSVC构建69ddebc成功；Node24.18.0在Windows本地测试。
- native-runs已有多款真实页面通过；Devin/Quark/Cursor的JS DOM变体通过。截图/SSH失败与HTML Trusted Types限制独立记录，不把它们写成不支持。
- Thunderbird160通过内容页和主界面parsed-html路线；Firefox160已建BiDi会话，但Windows启动器PID与浏览器PID不同，被校验挡住，已在本地补归属验证，待提交/部署重测。
- Frida17.25.2独立UV/Python3.13测试环境部署成功；aDrive x64宿主DispatchMessageW计数13/13、Douyin ia32启动/宿主进程6/6，detach后不增长、原字节恢复。仅原生函数Hook，不是页面注入通过。
- 用户已明确允许结束核验过的本轮测试进程，初始会话保留。force-owned-90d82825.json记录27个主PID；子进程/启动器需要按测试profile随机标识、路径和创建时间继续核验，不能按软件名杀进程。

## 证据和工具

仓库外缓存windows-validation-20261112：plan.json（44项，Mac41名加Gecko/CodexSwitcher）、aux-plan.json、inventory.json、host-layouts.json、host-directories.json、runs/native-runs/gecko-runs/hook-runs各独立result.json。不要覆盖旧run或以最后一次网络失败抹掉旧成功。

Windows运行/构建数据位于D:\exe\injectra-validation-20261112；编译产物cargo-target，Frida环境frida-venv，安装来源证据另在原install-logs。代码只经Git从Mac接收；临时交互任务/launch-code.txt为生成的测试运行工件，不是远端源码编辑。

SSH指纹/账号从全局current-truth读取。批量工具使用独立ControlMaster缓存socket避免重复握手；临时Git源仅监听Mac127.0.0.1:41839，严格只导出Injectra/.git；当前SSH反向源端口51796，有15秒keepalive，checkout origin临时指向该源。结束阶段需关闭自有Git daemon/反向转发，不能动原PortBraid或用户SSH会话。

## 当前关键缺口

- 部分安装路径是启动器，不是资源根。QQ真实包在versions/9.9.33-52230/resources/app；Figma真实exe在app-126.11.8/Figma.exe（plan已改）。Douyin资源在8.8.0子目录。不能凭根目录无app.asar判无Electron。
- Folo目录仅有165MB单exe，尚未确认是完整应用还是安装/自解压入口；先查引擎/注册信息，不能记App不支持。
- 外部plan已给部分无CDP项增加WebView2/Qt回环环境探测，但尚未跑新批次；必须记录为环境候选，不预称其引擎已确认。
- 原有Mac prepare只支持macOS；Windows正式prepare/dev未交付。正在准备测试专用Windows副本→同一Bootstrap协议→共享runtime proof；可选Rust remote_live测试已编译，尚未实跑。tests/windows/prepare-directory-copy.mjs草稿仅支持默认目录，需要覆盖版本化资源根和ASAR/PE完整性，不能直接宣传为正式Windows准备器。
- Deskreen/OBS未见安装清单；Clash/EasyConnect在Mac原阶段主要是静态候选/网络副作用未启动，Windows要逐项说明，不静默跳过或启用代理/VPN业务。

## 恢复动作

先核对git status与缓存结果，收取在跑测试/构建输出；仅Parent改main。本轮权限和目标已充分，不重复问是否继续。补齐Firefox、正确入口路径及适用pre-runtime/Gadget路线后更新逐应用表，保留未完成/不适用/失败的精确阶段。Windows测试客户端不等于正式CLI后端。
