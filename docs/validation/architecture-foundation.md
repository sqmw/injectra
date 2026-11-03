# 长期注入架构基础重构验收

日期：2026-11-03。归属：[T-008](../TODO.md#t-008)；决策：[D-004](../DECISIONS.md#d-004)。起始源码：990967b。当前架构见[边界](../architecture/boundaries.md)及[执行域契约](../architecture/contracts.md)。

## 结果

8个crate的执行域与适配依赖已建立。core不包含URL、源码执行或JSON约束且没有依赖；native只依赖core。Web载荷从原payload迁入web，唯一页面JS及examples未修改。application接受注入的Web后端，CLI投影维持既有字段。平台元数据解析与Electron宿主规则已分离。

原生契约允许无URL的目标，按目标架构解释地址，独立提供读和写端口；测试内Vec验证普通传输、部分进度、能力拒绝和结果未知，没有真实进程访问或内存注入。

## 自动验证

| 检查 | 已实测结果 |
| --- | --- |
| make check | fmt、clippy -D warnings、16项Rust测试、5项Web测试、架构依赖与文档/差异检查通过 |
| make architecture-check | offline Cargo metadata核对8个crate角色依赖；阻止core/native耦合Web和具体后端 |
| core/native独立构建 | cargo check -p injectra-core -p injectra-native --no-default-features --locked --offline通过 |
| application独立构建 | cargo check -p injectra-application --no-default-features --locked --offline通过，不要求CDP或宿主装配 |
| native依赖树 | 只有injectra-native → injectra-core，无Web、Frida、序列化库或平台后端 |
| 原有Web源码 | 与990967b对照，web/runtime.js及examples无差异 |
| 任务检查 | project-tracking-check通过；关闭记录迁移后再次核验 |

Rust分布：应用合同4、CDP传输3、核心身份1、宿主规则1、原生合同4、Web Bundle3。测试中的替身只用于契约，不算第二真实后端支持。

## 真实Electron与兼容性

运行make test-electron，独立fixture通过原有10项核心检查：发现、CSS计算样式、HTML挂载、JS交互、重复更新、清理、重复移除、错误目标拒绝、脚本失败清理和macOS启动规划。

本轮追加断言通过：targets只含id/title/url；运行错误前缀injectra:且退出1；缺少必填参数退出2；LaunchPlan只含executable/arguments/note且调试参数不变。manifest仍用原examples/accent/manifest.json。

环境：macOS arm64，Electron45.2.1、Chrome156.0.8078.75。原始证据位于仓库外默认缓存根的runs/electron-xZyk1B，包含result.json与electron.log。fixture仅启动并退出自己的进程，没有接入或重启用户应用。

## Review与限制

- 每阶段提交前自审接口、依赖与源码差异；Web迁移和原生/平台拆分分别通过make check再提交。首次doc-check的旧payload导航和首次tracking检查缺number均已修正，未放宽检查。
- Review补齐只读trait对象调用、运行时写能力拒绝且缓冲区无副作用；传输后失联保留Unknown及source，不自动重试；CDP测试移入独立模块保持生产文件职责。
- 原生合同与测试代码完全分开，无真实原生后端、unsafe或外部Frida依赖；不宣称Windows、Linux、WebKit、真实原生脚本/模块加载或跨进程内存修改已通过。
- 当前控制进程内BackendId不是持久身份；真实原生后端仍须校验进程实例和目标权限。Executed表示目标已处理本次执行尝试，不保证载荷每条语句均执行、成功或可撤销。
- 回退应按完整阶段提交的反序进行，保持workspace及锁文件一致；只回退单个core文件会破坏新契约。源码回退不改变历史MVP证据，也不撤销页面载荷副作用。
