# 通用注入核心与 pre-runtime dev 模式

当前定义：[T-040@v3](../TODO.md#t-040)。用户批准CLI收敛与平台代码删减；决策见[D-013](../DECISIONS.md#d-013)。

## 两条路线

| 路线 | 行为 |
| --- | --- |
| runtime | 原有targets/apply/remove连接CDP、Electron Gadget或WKWebView，不依赖dev服务或准备器工具 |
| pre-runtime | prepare在派生副本准备少量dev引导；dev监听外部manifest v1并在真实App网页更新 |

一次准备入口 → 现有编辑器保存JS/HTML/CSS → CLI发送最新增量 → 真实App显示效果。普通修改不重新打包或签名；仅页面增量，无用户startup.cjs、导出、自动重启、模拟预览或工作台。

## 模块边界

- application/dev：读取一致增量、内容版本、来源与通过Web契约执行。只读用户文件，不创建或维护完整工作区。
- preparation：按Electron CommonJS、目录/ASAR/嵌套归档、完整性格式识别入口；通过平台helper只修改派生副本。原入口身份、同步顺序和模块解析保持；不支持则明确拒绝。
- Bootstrap：App内小型Node引导和独立Web后端，连接CLI回环服务；随机挑战/角色认证，拒绝浏览器Origin。没有浏览器文件API或界面。
- CLI：prepare/dev装配、启动或连接精确派生实例、监听与JSONL回执。异步仅用于连接I/O，不反向改变core/Web/native契约。

准备器限macOS Apple Silicon、Electron CommonJS宿主、目录/ASAR及目录内嵌套ASAR。明文入口采用cjs-insert；嵌套/不透明JS或CJS通过main-delegate保留原字节，使用原宿主加载器建立原主模块身份。框架按元数据识别。纯ESM、模糊入口、无法保持语义/签名/资源条件的目标Unsupported。Node及锁定ASAR/Acorn工具只用于准备；runtime命令不要求这些工具已安装。

## 目标与更新

仅一个网页时直接使用；多个时终端选择一次或传--target。参数支持该开发实例的网页ID，内部使用后端/连接/文档身份；不凭PID代替页面。非交互歧义返回列表和错误，不猜测。页面销毁、重建或失联后停止本次自动更新，不自动转移或重注入。

监听manifest及声明的JS/CSS/HTML内容，连续两次稳定捕获后应用；内容相同不重复。串行执行，执行期间的新修改在下次捕获时合并为最新内容。manifest或文件临时无效时报告NotExecuted并等待修正，不运行上次旧版本。

每次提交携带请求ID、版本与目标；每次执行前保存一个pending记录，明确结果后保存最新receipt并清除pending。Unknown停止监听，进程异常退出遗留pending也阻止重放；用户核对后可用--acknowledge-unknown精确确认，再发起新执行。不是历史数据库或自动恢复系统。

文件、内容摘要、目标、执行确定性和匹配的用户JS行号随回执返回。无法归因的堆栈保留原始信息；不承诺自动追踪任意副作用。重复JS、DOM选择器、业务及清理逻辑由增量作者负责。共享Web runtime与既有remove/context.onCleanup保留。

Ctrl-C只停止CLI与连接，保留App和页面效果。不会自动终止已有用户实例。数据模式显式选择；QQ已有共享数据授权有效，其他应用不自动继承。

## 恢复与历史

Studio、studio-ui和完整workspace已从当前产品删除；有效通道和最小增量能力提取复用。历史代码保留在main提交07d55a3及旧工作树；删除前源码/补丁快照位于仓库外缓存recovery/t040-cli-20261112。恢复须新建恢复分支，不覆盖当前工作区。

[v1方案](../archive/T-040-platform-v1.md)、[v1执行](../archive/T-040.md#t-040)、[v2定义](../archive/T-040-v2.md)及原应用实测不改写。core/Web/native分域、原生独立内存契约及第33个正式适配方案前插件装配约束保留。

使用见[CLI开发说明](../dev.md)，验收见[CLI dev验证](../validation/cli-dev.md)。

[T-042 QQ复验](../validation/qq-cli-dev.md)补齐嵌套入口覆盖，不将T-040当次Unsupported推断为QQ不可注入。
