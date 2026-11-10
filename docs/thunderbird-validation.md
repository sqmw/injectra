# macOS Thunderbird 安装与验证

2026-11-11后续：[T-024主界面HTML装配路线](validation/thunderbird-html.md)已实测通过；原innerHTML严格合同仍partial，标准BiDi发现和正式CLI支持边界未改变。下文保留T-022历史结果。

[T-022](TODO.md#t-022)，输入2c0117c。官方Thunderbird159.0，独立空HOME/profile，不登录邮箱、导入账号或发送邮件，不改默认客户端。验证实际宿主窗口和自有内容页；正式CLI支持单独计。

成立标准：真实实例、明确目标、逐项执行与效果/清理回读；只有端口或会话成功不算页面通过。失败/未测如实保留，不放宽系统保护。安装前确认用户无既有Thunderbird，计划保留于用户Applications；运行数据在仓库外。

## 安装与重复验证

make setup-thunderbird-tests复用[官方Gecko分发校验](../scripts/setup-gecko-tests.py)，按[锁定清单](../tests/gecko/thunderbird.json)下载Thunderbird159.0（161907266字节），核对官方SHA512并通过codesign/spctl。安装于当前Mac用户的~/Applications/Thunderbird.app；已有路径只校验，不覆盖不同版本。下载和验签回执存外部gecko-tests/thunderbird/159.0。包内原许可证保留，主许可MPL2.0；清单含来源及许可入口。

make test-thunderbird使用新HOME/profile启动带窗口的实例，核验返回PID/profile/版本。Marionette负责主窗口与Thunderbird宿主枚举；内容页通过该页面的Gecko执行actor访问，BiDi单独探测并保留其枚举限制；只有该空配置测试进程设置MOZ_REMOTE_ALLOW_SYSTEM_ACCESS=1，以访问Thunderbird宿主DOM。它不是系统权限设置，不修改SIP、签名或已安装包；原邮箱账号没有接入。

[测试脚本](../scripts/thunderbird-smoke.mjs) · [Marionette测试客户端](../tests/gecko/marionette-client.mjs) · [双目标共享runtime检查](../tests/gecko/runtime-checks.mjs)。运行日志、截图和结果在外部runs/thunderbird-gecko-*；--inspect仅识别宿主与协议，不计完整验收。Firefox安装准备的默认命令仍是make setup-gecko-tests。

参考：[Thunderbird官方测试说明](https://github.com/thunderbird/developer-docs/blob/master/thunderbird-development/testing/running-tests.md)、[Mozilla Marionette](https://firefox-source-docs.mozilla.org/remote/marionette/)、[帧协议](https://firefox-source-docs.mozilla.org/remote/marionette/Protocol.html)。测试用协议客户端没有注册到正式CLI。

初始探查run thunderbird-gecko-05QvmK确认真实主界面chrome://messenger/content/messenger.xhtml、mail:3pane、空账号、Marionette3和BiDi可用。完整测试run thunderbird-gecko-meNaIf在等待mail-idle-startup-tasks-finished时45秒初始化截止；未执行页面载荷，已退出本次实例，原日志保留。随后采用Mozilla自动化常用的Mac --foreground启动选项，新run单独记录，不重写旧结果，也不加长原操作截止。

## 已确认的宿主差异

159.0安装包中的remote/shared/TabManager.sys.mjs仅实现Firefox/Android的getTabBrowser，其他宿主返回null。对应run thunderbird-gecko-FPQord已实际创建并加载两个Thunderbird contentTab，但BiDi getTree仍为空，完整验证在发现阶段失败；不能宣称Thunderbird已经具有Firefox相同的BiDi标签页支持。

本轮[Thunderbird测试宿主适配](../tests/gecko/thunderbird-host.mjs)通过原有tabmail枚举内容browser；记录会话、browsingContext、innerWindowId与URL。每次执行先核对身份，再调用该WindowGlobal的MarionetteCommands actor；没有补丁修改TabManager或伪造Firefox应用身份。此为版本相关内部接口验证，不是已发布产品后端。

run thunderbird-gecko-Mu2nMW确认目标发现成功，但globalThis===window断言失败，说明默认Marionette沙箱不是页面原有执行环境；测试在载荷前停止。后续通过页面自身执行入口检验同一合同，不通过删去身份断言或更改共享runtime使结果变绿。原始run保留。

下载阶段archive/ftp/CDN的默认TLS协商失败；官方HTTPS入口重定向与SHA512SUMS均现场核对，Thunderbird下载采用HTTP/1.1和TLS1.2成功；没有关闭证书验证或更改系统网络配置。Firefox默认下载参数保持原样。

## 最终实测结论

最终run为thunderbird-gecko-Ta3rSb，Mac26.5.1、arm64主机，Thunderbird159.0/buildID20261026161902，窗口模式。27项检查成功，但整组结果为partial、退出码1，不能写成全量通过：

| 场景 | 结果 |
|---|---|
| 内容页：共享runtime的JS/CSS/HTML、点击读回、更新/移除、Promise/异常、清理失败可见 | 通过 |
| 主界面：共享runtime的JS/CSS，载荷通过createElement/append创建控件并注册清理 | 通过 |
| 主界面：runtime.html直接插入button字符串 | 未通过；片段被清空，按钮不存在 |
| 不同页面隔离、错误会话/目标拒绝、导航世代失效、关闭目标拒绝 | 通过 |
| 会话删除、自有进程退出、无邮箱账号 | 通过 |

主界面实际为application/xhtml+xml，根与新建div均在XHTML命名空间；因此此前的命名空间猜测不成立。已测button片段经innerHTML后为空，与[Gecko特权文档的HTML净化设计](https://blog.mozilla.org/attack-and-defense/2019/12/02/help-test-firefoxs-built-in-html-sanitizer-to-protect-against-uxss-bugs/)相符，判为宿主设计限制，不外推为全部HTML都不可用。正向DOM路线单独列出，没有替换或删除原HTML失败用例；make test-thunderbird仍以严格的整组结果返回非0。

页面原环境执行由[调试入口](../tests/gecko/page-world.mjs)使用159.0实际自带的moz-src:///devtools/platform/jsdebugger.sys.mjs，在独立系统沙箱创建临时Debugger并执行到原Window global；沿用Gecko调试接口的bypassCSP选项，finally移除debuggee。没有修改应用CSP设置、关闭HTML净化、修改安装包、签名或系统保护。此能力依赖显式启动的空配置测试进程及其Gecko系统访问许可，不是普通网页的权限。

application-ui-dom.png与content-page.png已查看：前者的测试控件位于Thunderbird主界面上，后者位于自有内容页；计算样式、事件读回与清理均有断言。10次检查/尝试的自有PID均已退出，按安装应用路径复查无残留。安装保留于用户Applications；运行证据及空测试profile保留在外部缓存。用户Library/Thunderbird下也出现默认profile和崩溃/配置元数据，未删除或改写这些目录；实际验收会话的moz:profile均逐次核对为run内独立路径。

## 其他失败记录与验证边界

| run后缀（均为thunderbird-gecko-前缀） | 实际结论 |
|---|---|
| se3bOR | 主界面普通window.eval被CSP拒绝，未放宽策略 |
| 3yWXMx | 旧文档模块URI在159.0中不存在；以安装包实际模块位置修正测试工具 |
| e6i3LJ | 在现有沙箱再次创建调试沙箱的调用不适用；改为在既有独立系统沙箱初始化Debugger |
| EjpnND | 原Window执行通过，主界面HTML按钮被清空，严格HTML用例失败 |
| SGF1Nb | 独立内容页套件通过，主界面HTML套件失败；17项通过，整组partial |
| Ta3rSb | 增加独立的主界面DOM路线并通过；27项通过，HTML失败保留，整组partial |

初始化、BiDi发现和默认沙箱失败的前置run见上文；这些结果不回填为成功。日志在外部t022目录，各run保留result.json、host.log、protocol.jsonl与实际截图。没有邮箱登录、邮件收发或真实邮件正文验证；其他版本/平台也未验。首次启动等待通知超时的根因未最终定位，--foreground后的成功不等于已证明所有启动场景稳定。

自审：正式crates、Cargo依赖和web/runtime.js保持输入2c0117c版本；新增内容只是安装/验证工具和文档。测试客户端不是正式Gecko后端，Thunderbird的宿主发现与特权文档行为均不能由Firefox结果外推。源码可按本次独立提交恢复，安装、测试profile和失败证据不会随源码回退自动删除。

最终项目检查：make check通过18 Rust/5 Web/8 WK host；共享安装器默认Firefox路径重新验签/准备通过；Node/Python语法与Make入口通过。严格Thunderbird合同的partial仍保留，与项目检查通过分开计。
