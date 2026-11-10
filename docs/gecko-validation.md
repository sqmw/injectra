# macOS Gecko 接入验证

属于[T-021](TODO.md#t-021)；从官方Firefox开始，采用独立profile和本地测试页，不登录账号。目标是验证BiDi接入与共享Web runtime；当前正式CLI仍只有CDP/Electron Gadget/WKWebView，测试脚本不作为正式Gecko后端。

成立条件：真实Firefox页面发现、JS/CSS/HTML、Promise/异常、更新移除及另页隔离可验证；失败条件：只有端口/会话成功、无法执行到页面或不能确认清理。原始结果保存在仓库外，按run保留。

参考：[Mozilla连接方法](https://developer.mozilla.org/en-US/docs/Web/WebDriver/How_to/Create_BiDi_connection)、[协议变更](https://firefox-source-docs.mozilla.org/remote/Prefs.html)。

## 重复验证

make setup-gecko-tests按[下载清单](../tests/gecko/firefox.json)获取官方Mac通用版Firefox160.0（165955382字节）；以官方SHA512SUMS核对，并通过codesign和系统spctl验签。应用及下载包放在外部缓存gecko-tests/firefox/160.0，许可证为Mozilla MPL2.0，包内第三方许可保留；清单记录来源与许可链接。缓存删除后可按同清单恢复。

make test-gecko使用独立HOME/profile和本地HTTP页面，直接连接Firefox提供的回环BiDi端点；复用仓库唯一web/runtime.js，不复制实现。测试脚本不接入原账号，不改默认浏览器，不启用Gecko系统权限。只终止脚本持有的本次子进程。运行日志、协议摘要、截图及result.json留在外部runs/firefox-bidi-*。

[验证脚本](../scripts/gecko-smoke.mjs) · [测试用BiDi客户端](../tests/gecko/bidi-client.mjs)。这些文件不装配到正式CLI，不将会话连通或测试客户端成功误报为产品后端交付。

## 实测结果与边界

2026-11-10，Mac26.5.1、控制端arm64；官方Firefox160.0、buildID20261104181552。下载包包含x86_64/arm64；本轮不据通用包外推Intel Mac验收。实际使用带窗口的Firefox（moz:headless=false），核对协议返回的PID/profile与本次启动实例一致。Firefox的UA显示Intel/10.15是协议读回值，不能用它推断真实CPU或系统版本。

最终run：firefox-bidi-Js5cnC，15项通过：精确发现与Gecko身份、共享runtime的JS/CSS/HTML及点击读回、另一页不变、实际页面截图、同ID更新不累积、移除与回调清理、重复移除、Promise、页面异常、失败注入清理、不存在上下文拒绝、关闭上下文拒绝、导航后旧realm拒绝、当前CDP产品路线明确失败、BiDi会话结束。自己的Firefox进程已退出，按下载应用可执行路径复查无残留；应用与独立profile/证据保留在缓存。

当前CLI仅CDP/Electron Gadget/WKWebView；本次Gecko成功发生于测试用BiDi连接器与共享runtime，不是新增正式CLI后端。CDP发现/json/list在这个Firefox端口返回404/NotExecuted。后续如实施产品适配，应通过现有WebBackend/WebSession装配BiDi、映射上下文/realm身份与错误确定性，并验证断线/超时/清理；该实现尚未开展，不为此次测试改公开架构。

本轮只证明Firefox160.0中的两个本地页面；Thunderbird等其他Gecko宿主、用户网站、iframe、Windows/Linux/Intel Mac未验证。Firefox的页面执行环境ID（realm）会随导航失效；正式适配不能只凭不变的标签页ID执行旧目标。没有加载Gadget、申请系统访问或接入浏览器特权界面。

## 失败保留与自审

- firefox-bidi-2VUj39：10秒session.new初始化截止，无页面载荷执行；测试实例已退出。原日志不足以定位首次超时根因。
- firefox-bidi-fWtuGI：前12项通过，旧realm已被拒绝，但测试误预期no such realm；实际响应为no such frame并明确指出旧realm不存在。按原始回执修正测试断言，仍要求旧realm消失且新页未被执行；原失败run保留。
- 新run firefox-bidi-Js5cnC：会话初始化约5.174秒，15项通过。测试会话初始化单独设45秒上限并增加阶段日志；普通测试命令仍10秒，正式产品截止未修改，超时命令不自动重放。
- 下载与验签日志：外部t021/setup-firefox.log；原始运行日志：t021/first-firefox.log、trace-firefox.log、final-firefox.log；逐run保存result.json、host.log、protocol.jsonl和applied.png。日志/二进制不入Git。
- 实现者自审：复用唯一Web runtime；独立profile/PID核验；只退出本次spawn句柄；没有产品后端注册、Cargo依赖或原功能变化。测试数据仅为临时按钮，不是可交付插件。源码按本任务提交恢复；下载应用和运行证据不因源码回退自动删除。

最终仓库检查：make check通过18 Rust/5 Web/8 WK host，Node与Python脚本语法、文档/任务检查通过；应用截图已查看。正式crates、Cargo依赖及共享runtime相对885b8b5无变更。
