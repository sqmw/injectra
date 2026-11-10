# macOS十款应用顺序验证

2026-11-11，[T-032](../TODO.md#t-032)，基线8c041ce；本机Apple Silicon。用户明确要求按表中顺序尝试。以下是本批已取得的证据，整体推进状态以Task/Execution为准。

## 应用结果

| 顺序 | 应用/本次版本 | 当前证据 | 范围/剩余缺口 |
| --- | --- | --- | --- |
| 1 | 阿里云盘6.8.0 | 直接和LaunchServices启动均出现CDP监听日志，但取得页面前主进程退出；未执行载荷 | 无调试参数对照可运行12秒，提示与调试启动相关但未唯一定位；后续Node入口试验在启动前遇到非测试参数的新实例，等待用户退出授权 |
| 2 | 百度网盘8.2.7 | **CDP页面合同通过** | 先在启动data页面通过，随后另起实例在core.asar/index.html完整复验；以后者作为应用页面证据 |
| 3 | 企业微信5.0.9 | 已启动CEF renderer，指定端口及实际监听端口均未取得CDP页面 | 未执行载荷；不等于CEF机制或所有其他接入路线不支持 |
| 4 | 千问5.1.7.296 | **CDP页面合同通过** | chrome://newtab页面；启动生成包内日志导致严格签名失败，保存新日志到外部缓存后签名复核恢复；旧失败回执保留 |
| 5 | Anytype0.55.4 | **CDP页面合同通过** | dist/tabs.html宿主标签页面；同次发现index.html，不将其列为已执行目标 |
| 6 | Claude4.23118.1 | 明确拒绝带调试/网络覆盖参数启动，退出1 | 原用户实例保留；未取得页面，未绕过启动检查或开发者授权机制 |
| 7 | Xmind24.01.13311 | **CDP页面合同通过** | 独立welcome.html；原用户PID674与原文档实例保留，不将欢迎页证明外推编辑器画布 |
| 8 | 腾讯视频2.164.0 | 已有实际Web/未登录视图日志，指定端口及其他监听端口未取得CDP页面 | 未执行载荷；未用网络错误冒充CDP协议错误 |
| 9 | 网易云音乐3.1.5 | **CDP页面合同通过** | orpheus://orpheus/app.html实际页面；不外推账号/播放业务 |
| 10 | QQ6.9.86 | 原实例端口非CDP，Frida attach PermissionDenied；获准正常退出后，独立目录启动的CDP和Node入口均未取得目标 | 已重新打开普通QQ窗口；未提交载荷，未登录操作或发送消息 |

五项通过均复用正式CDP后端及[scripts/verify-live.mjs](../../scripts/verify-live.mjs)，验证自有CSS/HTML/JS面板、计算样式、按钮0→1、同ID更新不累积、remove及无节点残留。截图只裁剪自有测试面板。通过指列出的页面与操作，不代表所有窗口、登录后业务页面或任意载荷兼容。

## 运行证据

原始实验根为缓存目录runs/mac-ten-apps-fzwjzf8z；缓存根默认`${XDG_CACHE_HOME:-$HOME/.cache}/injectra`。安装包、会话、日志、测试输入和源码定向片段不进入仓库。既有原普查记录不重写。

| 应用 | 独立run（相对本次实验根） | 共享页面合同证据（相对缓存根runs） |
| --- | --- | --- |
| 阿里云盘 | adrive-hhdaz6y5、adrive-ls-2hlr6yg2、adrive-no-debug-jumvlj3h | 未进入页面；Node试验被运行实例预检拒绝，未产生新启动run |
| 百度网盘 | baidu-taufvr_h、baidu-page-hsaszip3 | live-lLMTj3为启动页；live-kgi4Iz为应用页 |
| 企业微信 | wecom-s_xhx2gr、wecom-ports-mqls8nfg | 未进入载荷；后者保存all-owned-listeners.txt及观察端口 |
| 千问 | qianwen-or0rz76r | live-vcTKtm；post-cleanup.json为后续签名恢复证据 |
| Anytype | anytype-6vg_d15u | live-emY5Bs |
| Claude | claude-n0ney_g8 | startup.log明确拒绝调试参数 |
| Xmind | xmind-n4miorqh | live-Yxmns7 |
| 腾讯视频 | tencentvideo-q4qo46_5、tencentvideo-ports-rzp5vumj | 未进入载荷；后者保存实际监听检查 |
| 网易云音乐 | netease-a3gaqwn6 | live-5Kv77m |
| QQ | qq-r_suqmov、qq-node-8qd6z4o9 | qq-existing-endpoints.json及qq-existing-attach.json记录原实例只读接入尝试 |

共16次新进程实验，其中阿里无调试启动只作对照，不计注入成功；百度两次属于不同页面范围，不能重复计为两款通过。Node桥接验证器未取得可用Node入口，不能把其准备完成或语法正确当作路线支持。

## 隔离、现场与清理

- 启动参数指定新的HOME/CFFIXED_USER_HOME、XDG目录和Chromium profile；沙箱应用使用自身容器Caches下的新目录。参数和目录存在不等于完整宿主隔离：企业微信/腾讯视频/网易云的指定profile未创建，不能声称所有宿主偏好、共享Group Containers或钥匙串均隔离。未进行账号登录、同步/上传、聊天或播放操作。
- 所有实际启动串行。现有Claude/Xmind实例未关闭；Xmind先定向核对用户目录派生及单实例锁逻辑，再另起新profile测试，且只在新PID拥有的回环CDP端口提交载荷。
- QQ原PID653的退出由用户明确批准，CUA用正常退出操作确认App quit；两次测试实例结束后以正常方式重新打开QQ。原账号数据未清空，重新打开时显示原账号自动登录/联网界面；不宣称已重新验证聊天业务。
- 阿里后续出现普通实例PID69081，缺少测试参数，不当作自有进程结束；相关授权请求及恢复点见STEPS。
- 跟踪到的本次测试PID在复核时均不再存在；主可执行文件SHA前后一致。LaunchServices不是直接子进程时，按确切测试profile、PID和启动世代核验；其退出状态不可取，不把未知退出状态写成退出码0。
- 阿里首次未保存启动前严格签名检查；第二次前后同为扩展属性错误，不能回填首次基线。Xmind、QQ严格签名在测试前后均为既有失败，未尝试修包。页面验证与签名状态分别报告。
- 千问唯一新增签名差异是框架Resources/quantum_apps/agent_runtime_debug.log。确认出生时间晚于run、无剩余进程和打开句柄后，将它移动到该run保留；未删除日志、未重签名，post-cleanup记录签名恢复。原result.json的passed=false仍保留，应用页面通过与恢复结果分别取证。

## 验证工具与审阅边界

本轮没有改产品源码或增加按应用区分的注入后端。缓存内cdp-attempt.py、cdp-portscan-attempt.py、node-attempt.py和node-bridge-test.mjs是一次性诊断工具：分别尝试启动/发现、核对实际监听、检查Node入口；使用实际app路径与本机缓存绝对路径仅限本次实验，不作为长期配置或公开插件。脚本在实验期间有增量修改，各run保留其真实参数/阶段结果，不宣称全部run使用同一未变实现。

自审核对了：通过项只有已执行的实际目标；启动data页不替代百度应用页；Anytype标签页/Xmind欢迎页范围不扩大；Node未连通不算桥接成功；原用户进程和未知新进程不归入清理；千问日志有可恢复备份；未知与未执行结果不自动重放。正式验证源码未修改，因此本轮验证以真实应用合同、脚本语法及文档/追踪校验为主，不无条件重复其他引擎的已有套件。
