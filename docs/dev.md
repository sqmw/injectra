# 开发与核心使用

本页主要记录已有runtime CLI用法，完整职责见[产品边界](requirements.md)。当前方向为[pre-runtime dev模式](architecture/dev-mode.md)：现有编辑器改外置增量，真实App看效果，CLI prepare/dev实现见下文；旧平台命令已移除。使用者编写载荷并处理应用控件、重渲染覆盖和再次注入；本页的remove及清理回调是既有辅助接口，不是任意JS撤销或一键回滚承诺。

## 环境与构建

版本入口：[mise.toml](../mise.toml)。已有匹配Rust/Node即可运行；使用mise管理时先按其正常信任与安装流程准备工具。`make build`构建debug CLI，`make run ARGS="--help"`显示命令。交付二进制无需Node；Node与Electron只服务开发测试。

构建脚本默认最多2个Cargo编译任务，所有Make入口通过[scripts/cargo.sh](../scripts/cargo.sh)调用。`Cargo.lock`锁定Rust依赖；不要在复现时随意更新锁文件。

架构已迁移为core/Web/native/application/CDP/平台/宿主模块；对外命令兼容。原生接口与测试边界见[执行域契约](architecture/boundaries.md)。

## 目标应用接入

1. 只读识别与参数规划：
   - `inspect-app --app <PATH> [--port <PORT>]`只读探测目标应用的宿主架构（Electron/NW.js/CEF/WKWebView/WebView2）、ASAR/目录入口形态（含ESM或字节码识别）、代码签名/符号阻塞项、可用路线及可选启动规划，不启动或修改目标；未知或原生无Web应用明确返回`Unsupported`。
   - `launch-plan --app <PATH> --port <PORT>`复用同一只读识别层输出回环CDP启动参数或环境变量规划（覆盖Electron、NW.js、CEF与WebView2；无CDP参数路由如WKWebView则明确拒绝）。
2. 如已有目标进程，是否退出与如何保存工作由用户决定。使用规划参数自行启动目标；部分应用不会开放或继承调试参数，不据此自动改包、重签名或绕过保护。
3. 使用`electron-gadget`或`webkit`路由时，可通过`session launch --app <APP> --route <ROUTE> [--gadget <DYLIB>]`、`session register --state <STATE_JSON> --output <SESSION_JSON> --route <ROUTE>`与`session verify --session <SESSION_JSON> [--route <ROUTE>]`管理受控实例与v2会话（见[Frida说明](frida-backends.md)），不自动终止已运行的同可执行文件实例。
4. 核对监听为回环地址或确认会话就绪后，执行`targets --port <PORT>`（或`--backend <BACKEND> --session <SESSION_JSON>`）；检查title/url并使用精确ID。没有页面、页面消失或不支持时停止，不默选第一个窗口。
5. `apply --port <PORT> --target <ID> --bundle <MANIFEST>`应用，`remove --port <PORT> --target <ID> --id <BUNDLE_ID>`移除。
6. `list --port <PORT> --target <ID>`查询当前页面在共享runtime注册表中的受管Bundle（`id`、`nodes`、`cleanups`）；此操作只读注册表状态。
7. `eval --port <PORT> --target <ID> (--expression <JS> | --file <PATH>)`在目标页面上下文执行JavaScript表达式并等待Promise返回JSON（顶层`undefined`归一为`null`；文件上限1 MiB）。`eval`以目标页面权限运行，并非沙箱或技术意义上的只读隔离；调用方负责所提交表达式的副作用。

## 机器输出契约（`--machine`）

默认模式保留原有输出：单次命令成功输出格式化JSON并返回退出码`0`；命令失败向`stderr`输出`injectra: <错误>`并返回`1`；CLI参数解析失败返回`2`。

传入全局参数`--machine`后，单次命令统一向`stdout`输出单行JSON信封并按执行确定性返回退出码：

- 成功：`{"ok":true,"data":...}`，退出码`0`。
- 失败：`{"ok":false,"error":{"kind":"...","phase":"...","certainty":"NotExecuted|Executed|Unknown","message":"...","causes":[...]}}`。
- 退出码映射：`2`为CLI参数无效（`NotExecuted`），`3`为执行前拒绝或目标缺失（`NotExecuted`，可在修正条件后重试），`4`为目标已执行但脚本/编码阶段失败（`Executed`），`5`为结果未知（`Unknown`，禁止自动重试）。`dev`子命令继续按JSONL流式输出事件，在启动前或异常终止错误时在`--machine`下返回同结构错误信封与退出码。

可执行文件路径中存在空格时，使用`make build`产出的二进制并对路径正确加引号；Make的ARGS只适合自己编写的简单参数，不接收未经检查的第三方命令文本。二进制位置为外部缓存根下的`cargo-target/debug/injectra`，或显式`CARGO_TARGET_DIR`下的`debug/injectra`。

Electron官方列出了[remote-debugging-port](https://www.electronjs.org/docs/latest/api/command-line-switches)；本适配仅使用[CDP Runtime协议](https://chromedevtools.github.io/devtools-protocol/)中的JS执行能力。调试入口提供页面控制能力，只为受信任的本地操作开放；客户端限制目标为127.0.0.1并不替目标应用证明其监听地址。

## 插件归属与Bundle合同

所有实际插件和可复用载荷统一在独立插件库（独立 web-ui-plugins 仓库内 README.md），Injectra不维护主题/应用插件副本。通过`BUNDLE_MANIFEST`或CLI的`--bundle`显式传入所选文件；不把插件库路径写入构建依赖。核心测试只生成仓库外临时Bundle，不读取插件库。


外部插件manifest包含`version:1`、唯一`id`和可选`css`/`html`/`js`相对文件名，至少一个非空。ID限1–80个ASCII字母、数字、点、下划线或连字符；未知字段、未知版本、绝对资源路径、解析后逃出manifest目录的路径/符号链接均拒绝。单文件上限1 MiB。

JS文件作为async函数体执行，可使用`context.root`访问本bundle插入的HTML容器；没有HTML时root为null。可选的既有接口`context.onCleanup(callback)`由载荷作者登记清理，回调按逆序执行；需要使用时先登记，再执行对应副作用，不要求所有脚本可撤销。HTML中的script标签不作为脚本加载机制，行为代码放JS文件；HTML事件属性等仍属于受信任代码，不进行净化。

CSS/HTML以JSON数据嵌入共享程序，任意引号不应改变生成程序结构；JS作为显式执行源码。本版本不是沙箱，不允许把未知来源载荷当成安全主题。CSS也可能发起资源请求或影响其他模块；不同ID的所有权分离不等于样式隔离。

## 失败与恢复

- 相同ID更新先清旧再执行新；失败后不会自动恢复旧版。可修正文件后手动再次apply。
- remove移除拥有的节点并调用登记回调；任意网络、持久化、第三方状态变化不能自动撤销。
- cleanup报错仍尝试其余回调并移除节点，但已经失败的清理不自动重试。必须按错误核对页面，不能把下一次`absent`视为所有副作用已撤销。
- 超时/断线时执行结果未知，脚本可能仍在运行；不自动重放。重新核对页面状态后再决定清理或重载。
- 页面刷新或重建后，由使用者决定并手动重注入；React/Vue更新覆盖节点等应用行为由载荷作者处理，自动DOM守护不属当前职责。

## 测试与运行数据

`make setup-tests`把tests的锁文件复制到外部测试工具目录，再执行npm ci；`make architecture-check`通过离线Cargo元数据检查执行域依赖；`make check`运行静态检查、Rust/Web测试、文档检查；`make test-electron`创建独立隐藏Electron窗口并验证真实UI状态，结束时只退出自己启动的fixture。

默认缓存根`${XDG_CACHE_HOME:-$HOME/.cache}/injectra`：`cargo-target/`构建产物，`test-tools/`锁定测试依赖，`runs/electron-*/`每次验证的profile、electron.log和result.json。`INJECTRA_CACHE_DIR`可整体覆盖；`CARGO_TARGET_DIR`可单独覆盖构建位置。覆盖路径仍应选仓库及同步目录之外。

这些数据不是源码真相；恢复时使用Cargo/npm锁文件重建，历史测试证据保留在原run，项目[验证摘要](validation.md)记录相对run身份。不要把cache、profile、日志和调试输出提交Git。没有正式发布或远端服务。

## 可选Mac后端

默认CDP命令保持兼容。新增后端、会话准备和锁定依赖见[Frida](frida-backends.md)；WKWebView执行与边界见[WebKit](webkit-backend.md)。构建产物可以离开源目录，第一方helper已嵌入Rust可执行文件；外部可选依赖与缓存仍须准备。开发工作树可通过CARGO_TARGET_DIR选择独立编译缓存，避免不同窗口覆盖同一个开发可执行文件。

## pre-runtime CLI dev

先运行make setup-preparation安装锁定工具到仓库外缓存，然后使用构建的injectra：

```sh
injectra prepare --app "$APP" --output "$DERIVED_DIR" --data-mode isolated
injectra dev --artifact "$ARTIFACT_JSON" --bundle "$MANIFEST"
# 有多个页面时交互选择，或显式指定该开发实例的网页ID：
injectra dev --artifact "$ARTIFACT_JSON" --bundle "$MANIFEST" --target "$PAGE_ID"
```

prepare输出artifact_file；将其作为ARTIFACT_JSON。APP为原应用路径，DERIVED_DIR为仓库外新副本父目录；manifest沿用v1格式，用户用现有编辑器修改声明的文件。shared需要用户明确接受该应用共享数据；isolated不是账号安全沙箱承诺，沙箱宿主无法建立条件时拒绝。不支持用户startup.cjs。准备记录entry_strategy为cjs-insert或main-delegate；旧记录缺少此字段仍可加载。后者保留原归档，通过宿主加载器执行原主模块，见[QQ复验](validation/qq-cli-dev.md)。

dev为前台命令，stdout为JSONL事件：dev-ready、目标列表/选择、execution、source-error和可关联log；stderr仍以injectra前缀报告最终失败。execution包含request_id、version、target、files、certainty、result/error与可确定的JS位置。窗口效果直接在App查看。

页面重建/关闭后重新运行dev；已运行的精确派生实例会复用，不自动退出原实例。Ctrl-C停止监听但不退出App、不撤销代码。文件改变不会重新准备应用包。端口在prepare时选择并保存，冲突明确失败，不自动修改已签名配置。

Unknown或上次异常退出遗留pending时，先核对App实际效果；明确接受新一次执行后使用：

```sh
injectra dev --artifact "$ARTIFACT_JSON" --bundle "$MANIFEST" --acknowledge-unknown "$REQUEST_ID"
```

这会开始新请求，不是重放旧请求。准备记录、配对密钥、最新回执、锁和进程日志在仓库外派生目录；不要将pairing.json或带密钥的包内config.json发布。make test-preparation运行自有目录/ASAR Electron验收，不触碰安装应用。
