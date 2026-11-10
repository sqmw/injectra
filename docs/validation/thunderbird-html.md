# Thunderbird 主界面 HTML 路线验证

2026-11-11，[T-024](../TODO.md#t-024)，输入4e8bf09。用户明确选择先补齐Thunderbird实机验证，正式CLI另行推进。沿用159.0/buildID20261026161902、macOS arm64、独立空HOME/profile；不登录邮箱，不改安装包或安全设置。

## 收口结论

主界面HTML字符串经测试用的DOMParser装配路线已实测可用，完整验证注入、样式、交互、更新与清理。原共享runtime的innerHTML路线仍受宿主净化限制；默认严格合同仍是partial，未被覆盖为成功。BiDi仍不枚举Thunderbird标签页，宿主发现仍使用Marionette/tabmail。

本轮完成的是替代装配路线的验证，不是正式产品HTML能力或Gecko后端交付。原T-022结果见[历史验证](../thunderbird-validation.md)。

## 原因与路线

诊断run `thunderbird-html-probe-7Th1XY`在真实主界面比较同一个button字符串：innerHTML、template.innerHTML、createHTMLDocument及本次parserUtils尝试均未保留按钮；DOMParser的HTML/XML文档及MozXULElement入口保留了按钮。主界面及元素的XHTML命名空间正常。

选用标准DOMParser路线，不使用私有XUL解析入口，不修改净化/CSP设置：

1. 测试用装配器保留原HTML字符串；传给共享runtime的html字段使用空白字符串，仅让runtime创建并拥有容器。
2. 在runtime执行的JS回调开头，用DOMParser解析原字符串，importNode并replaceChildren到context.root。
3. 原JS交互随后执行；节点所有权、CSS、同ID更新和清理仍由仓库唯一runtime处理。

[装配器](../../tests/gecko/parsed-html.mjs)仅用于自有测试片段，拒绝script元素，行为代码单独交给JS载荷；不是任意HTML/插件Bundle的通用编译器。正式crates、Cargo依赖及web/runtime.js相对输入版本无变化。

## 两个独立验收合同

| 命令 | 合同与结果含义 |
| --- | --- |
| `make test-thunderbird` | 保留原始默认合同：原applicationUIHtml、DOM和内容页套件必须全部通过；159.0仍partial，Node退出1/Make退出2 |
| `make test-thunderbird-html` | 显式新合同parsed-html-contract-v2：验证新增applicationUIParsedHtml、原DOM和内容页套件，以及会话/页面隔离、导航/关闭拒绝和最终清理；仍执行并记录原HTML套件 |
| `make test-gecko-html-unit` | 三项jsdom测试；已加入make check，验证装配、脚本拒绝、完整更新合同与失败后的套件隔离 |

V2的passed只对应requiredSuites列出的新合同，baselinePassed/baselineOutcome继续独立报告原合同；不能只读passed就宣称原innerHTML已修复或全部Thunderbird兼容。该合同及新run在候选路线验证前明确建立；旧run和原断言未改写。

## 实测与失败保留

| run | 实际结果 |
| --- | --- |
| thunderbird-html-probe-7Th1XY | 诊断入口比较完成；非完整合同验收，自有进程退出 |
| thunderbird-gecko-Dx9jkF | 新路线首次挂载、样式和点击成功；更新用例遗漏新mountMode分支导致root为null，随后共享测试计数污染下一套件；23项检查成功，整组失败；进程退出、原日志保留 |
| thunderbird-gecko-gHjeHb | 首个V2路线通过，37项检查成功；applicationUIParsedHtml/applicationUIDom/contentPage均通过，baselineOutcome仍partial；原HTML失败完整保留；最终无注入节点、无账号、自有进程退出 |
| thunderbird-gecko-DxFYST | 最终代码的V2独立复验：同样37项检查通过、原合同仍partial；目标节点清理、空账号与自有进程退出均确认 |
| thunderbird-gecko-fjBUxN | 修正测试工具后的默认合同独立复测：27项成功、原HTML仍失败、partial及非0退出保留；自有进程退出 |

更新输入分支修正后，仍要求原有节点数、文本、事件计数与清理计数，不放宽断言。verifyRuntime新增finally清理本套件拥有的ID和测试变量；主失败保留，清理失败附带记录，避免失败用例影响后续套件。新增单元用例主动制造中途失败，再连续执行parsed-html与dom完整合同，证明隔离。

首个成功run的`application-ui-parsed-html.png`和`content-page.png`及最终DxFYST主界面截图已查看：主窗口顶层测试面板与自有内容页面板分别可见，页面身份、实际计算样式及点击回读同时有断言。不能仅凭截图判断执行上下文；该上下文已由原Window global与会话/目标世代断言核验。

以上逐run的result.json、host.log、protocol.jsonl及截图位于仓库外默认缓存根runs目录。临时诊断脚本位于外部t024目录，源码按本轮诊断用途保留，不参与产品构建；复验入口以本页Make命令为准。未删除测试profile、失败日志或用户默认配置。

## Review与验证边界

- 自审确认正式产品代码、唯一Web runtime、安装包和默认CLI后端未改变；无插件/插件副本进入仓库。
- 仅验证159.0主界面及两个自有内容页；jsdom的输入、转义文本和SVG测试不外推为Thunderbird所有HTML标签兼容性。
- 标准BiDi发现限制未解决，也不通过补丁改TabManager或伪造Firefox身份处理。正式后端与任意用户插件适配不在本轮范围。
- make check通过18 Rust、5 Web、8 WK宿主及3项新增Gecko HTML单测；Node语法、文档和任务检查通过。默认实机合同的partial与项目检查/新路线通过分别记录。
- 源码可以按本任务提交回退；安装应用、独立profile及历史run不会随Git回退删除。未再次下载或替换Thunderbird。
