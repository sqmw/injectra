# 版本与交付映射

## milestone/core-engine-cli-foundation · 2026-11-27

- 对应 [M-002 · 通用底层注入引擎与 CLI 底座夯实](TODO.md#m-002)（[规划](plans/milestone-2-core-engine.md) · [D-015](DECISIONS.md#d-015) · [收口档案](archive/T-086.md#m-002)）的注解式里程碑 tag。能力与双端验证基线提交为 `e6b11dc`；本条目所属提交仅补充发布映射说明，不移动既有 tag、不改动产品源码或重跑已通过产物。CLI 包版本仍为 `0.1.0`，本标识仅对应源码 tag，不含预编译安装包或 GitHub Release。
- 包含 `M-002` 四个已完成阶段：
  1. [T-083](archive/T-083.md#t-083)（`64e00bb`）：`--machine` 单行 JSON 错误信封与 `0/2/3/4/5` 确定性退出码、`list` 与只读 `eval` 原语。
  2. [T-084](archive/T-084.md#t-084)（`aa8e194`）：只读 `inspect-app`、通用 `launch-plan`（Electron/NW.js/CEF/WKWebView/WebView2 与未知目标 `Unsupported`）及非 CDP `session launch/register/verify` 收敛。
  3. [T-085](archive/T-085.md#t-085)（`ad76b92`）：[多架构宿主机制基线](architecture/host-mechanisms.md)、无 `unsafe` 的 `injectra-platform-windows`、跨平台 `preparation`（支持 macOS 与 Windows x64/x86、Electron ESM `esm-delegate`、包内 Junction 重建与 PE `INTEGRITY`/`ELECTRONASAR` 更新）及正式 Windows MSVC V8 `session attach-v8`（在 macOS 5 布局与 Lenovo Windows `win-formal-0XELcm` 实机通过）。
  4. [T-086](archive/T-086.md#t-086)（`e6b11dc`）：`ErrorKind::PolicyBlocked` 策略拦截分离、`manifest v1` 可选 `html_mode="dom"` 与纯 Rust 声明式 DOM 编译（Electron 45.2.1 `trusted-types 'none'` 实机通过）、纯 Rust `asar.rs` 与[外部依赖减重评估](architecture/dependency-reduction.md)。
- 明确保留既有非目标边界：不支持 `.jsc` 字节码与原生二进制段改写、`iframe` / `<webview>` 子帧注入不在本里程碑范围内、不包含内置 MCP Server 或图形工作台。

## milestone/generic-cli-windows-validation · 2026-11-13

- 已创建并推送的注解式里程碑，见[T-046](TODO.md#t-046)。tag精确指向`aaa254a3ddc0d48ff61bc3109cd2b736b45eb36e`；能力/实测基线134cf12，里程碑提交仅新增映射/任务元数据。远端tag对象`b4e82e3955c1d947cf583f903fc1881221c8033e`及解引用提交已核对一致。
- CLI路线已收敛：runtime连接既有Web页面；pre-runtime为轻量prepare/dev、现有编辑器和真实App反馈。共享Web runtime及core/web/native边界保留；没有工作台、内置编辑器或完整工作区系统。
- prepare/dev实测：目录/ASAR/嵌套ASAR夹具、macOS Apple Silicon上的QQ主页面与Claude本地壳；普通增量更新免重新准备App。见[CLI dev](validation/cli-dev.md)、[QQ修复与复验](validation/qq-cli-dev.md)。不外推Windows正式prepare/dev支持。
- Windows累计35/38安装项取得页面注入证据，含现有原生Rust CLI/CDP、Gecko测试客户端、实验Bootstrap/V8等不同通道；3款仅JS DOM路线通过且HTML字符串限制仍保留。不表示35款均有正式CLI后端或所有业务页面通过。见[逐项历史](validation/windows-apps.md)、[当前复验与网易云通过](validation/windows-retries.md)。
- 剩余3项：优酷启动依赖/Widevine404；百度网盘与企业微信当前原生欢迎/登录窗口下的Web入口仍未取得。保留旧失败与Unknown，不把接口/夹具当成兼容性证明。
- 沿用已记录的make check、独立Electron、Windows格式/受控ABI及实机证据；本次仅发布元数据，重新检查映射、链接、任务身份和diff，不重启App或重写原run。CLI包版本仍为0.1.0；本里程碑为源码tag，不含安装包或GitHub Release。

## milestone/pre-runtime-web-injection · 2026-11-11

- 用户认定的pre-runtime里程碑，本地annotated tag；精确指向`04a7013e2b3838db635850daf12edb810f064b36`。标记创建见[T-036](TODO.md#t-036)。
- [T-037](TODO.md#t-037)已推送origin并核对远端：tag对象`75a2b12a867866c2f07db648e5337127cd3fe732`及解引用提交与本地一致。
- 已验证：阿里云盘6.9.3独立测试副本的真实本地登录壳页面执行共享runtime，随机标记、计算样式和按钮交互回执通过。
- 条件：资源补丁与ASAR完整性重建、本地测试签名允许库加载、独立目录及测试钥匙串；fuses和Hardened Runtime保留。原普通窗口已恢复。
- [实机证据与复现](compatibility/adrive-resource-patch.md)；不外推原包runtime兼容、登录后业务或正式pre-runtime后端。该tag不改变CLI版本号。

下方阶段记录保留当时的合入/发布状态；其后已合入的源码由本次里程碑tag包含，当前验证范围以前述映射及对应实机记录为准。

## T-021/T-022 · Gecko验证成果（未发布）

- Firefox2c0117c、Thunderbird9531cad已由[T-023](TODO.md#t-023)快进合入本地main，尚未推送；未移动tag或创建发布。
- 本次交付为官方下载/安装、可重复验证工具与证据；正式CLI仍只有CDP/Electron Gadget/WKWebView。
- [Firefox160.0](gecko-validation.md)验证通过；[Thunderbird159.0](thunderbird-validation.md)内容页及主界面DOM路线通过，HTML按钮字符串与BiDi枚举限制保留，严格整组仍为partial。
- make check不启动这些应用；make test-thunderbird当前仍以严格合同返回非0，不能因合入而改写其原始失败。

## T-019 · 控制端与目标设备边界解耦（未发布）

- 已通过[T-020](TODO.md#t-020)快进合入本地main，尚未推送；未移动旧tag或创建发布。
- 24ca687分离控制端、目标身份与传输，01cbf7f分离WK宿主并补强回调及诊断，c6d0cf3关闭验收。
- v2会话与v1只读兼容；旧Frida页面ID需重新发现。手机实现未加入；[接口与支持范围](architecture/device-boundaries.md)、[验证与限制](validation/device-boundaries.md)。

## T-017 · macOS多引擎开发交付（未发布）

- 已通过T-018快进合入本地main，尚未推送；既有MVP tag不移动。CLI包版本暂沿用0.1.0，新增能力以本分支提交及--help为准，不把旧tag当作新后端版本。
- 1c02dca建立基线，6c1655b交付Electron Gadget，5f2d013补CEF/NW.js合同；eb42869交付WKWebView；T-017关闭提交c2f090f；T-018接收与验证记录见[任务状态簿](TODO.md#t-018)。
- 默认CDP保持兼容，新增electron-gadget/webkit显式后端；可选依赖安装和使用见[Frida说明](frida-backends.md)、[WebKit说明](webkit-backend.md)。
- 验收与限制见[Mac交付](macos-delivery.md)和[当前矩阵](compatibility/current-matrix.md)；未创建安装包或GitHub Release。源码回退按独立提交处理，不自动撤销页面副作用或删除运行证据。

## v0.1.0-mvp · 2026-10-31

- 状态：用户已亲自验证并确认MVP通过；本次交付为源码与Git版本标识，无安装包或GitHub Release附件。
- 仓库：[sqmw/injectra](https://github.com/sqmw/injectra)。
- 精确源码映射：Git tag `v0.1.0-mvp`。用 `git rev-list -n 1 v0.1.0-mvp` 获取该交付提交；不因main后续演进而移动既有标识。
- 核心实现：`ba1e003`；真实豆包兼容性与验证命令：`bc4bff9`；用户验收与仓库初始化历史在交付tag中一并保留。
- 远端原初始化提交 `5ba11c2` 的Apache-2.0许可证已保留并合入，没有覆盖远端历史。
- 功能：显式本地CDP页面发现、共享CSS/HTML/JS注入、相同ID更新、受管理清理、macOS标准Electron启动规划。
- 证据：[自动验证与用户验收](validation.md)、[豆包2.33.5实机报告](compatibility/doubao.md)。
- 边界：豆包为定制Chromium147，不计作标准Electron覆盖；自动重注入、GUI及WebKit适配未实现；任意JS副作用无法保证完全回滚。
- 恢复：源码可检出上述tag；运行页面改动使用remove清理；调试启动的应用应正常退出后无参数重启。Git回退本身不撤销正在运行页面中的副作用。
