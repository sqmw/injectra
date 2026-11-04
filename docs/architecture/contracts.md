# 执行域契约与扩展入口

权威架构见[边界](boundaries.md)，取舍与术语映射见[D-004](../DECISIONS.md#d-004)。本页描述产品workspace的合同；native crate只有契约。[references](../../references/README.md)中的真实应用参考脚本不构成产品后端。

## 公共基础

`core`不定义万能Session，也不决定源代码语言、传输协议、输出格式或操作寿命。BackendId由调用方创建后端实例时分配，在控制进程内唯一；克隆身份表示同一个实例。TargetRef中的key仅由其后端解释。跨实例传入同名key会被拒绝，跨控制进程持久句柄尚未定义。

Capability是执行域命名的操作标识，使用精确匹配，不提供隐式fallback。Error的Display保留CLI可读错误，source保存底层原因；代码应检查kind/phase/certainty，不解析Display字符串。

| 确定性 | 含义 | 调用者边界 |
| --- | --- | --- |
| NotExecuted | 本次目标操作未发送/被执行前检查拒绝 | 不代表历史操作没有副作用 |
| Executed | 目标已处理本次执行尝试，或已有传输进度证据 | 可以是解析/运行异常、部分写入或返回值编码失败；不保证载荷所有语句执行或可撤销 |
| Unknown | 请求可能已执行，结果无法确定 | 不自动重放；保留原始原因与已确认进度 |

准备阶段的文件错误/输入错误不会升级成目标执行；后端I/O必须结合实际发送边界构造错误，不能从超时字面推定未执行。core的字符串/文件错误便捷转换只用于准备阶段。

## Web执行域

- WebTarget具有TargetRef、title/url和PageContext。当前上下文是MainFrame；仅作页面目标，不包含worker或原生插桩环境。
- WebBackend提供id/targets/connect；connect只能接受其本实例TargetRef，并返回绑定该目标的WebSession。
- WebSession提供target/capabilities/evaluate_page；页面求值等待Promise、以JSON值返回，脚本异常必须保留。某个JS引擎可执行脚本并不足以声明`web.page.evaluate`。
- application公开targets/apply/remove/execute；后端由调用者注入，不导入具体适配。apply保持manifest v1的目录/大小校验和原有共享程序生成；execute核对后端身份、会话目标与页面能力。
- CLI在单次命令边界将显式端口和原target ID转换为内部TargetRef；输出投影回原id/title/url，不暴露实例ID。默认命令仍只装配CDP。

## 原生执行域

- NativeTarget持有TargetRef、pid及TargetArchitecture；PID是元数据，未来真实后端必须验证进程实例身份。架构包含名称、地址位宽和字节序，不从控制机推断。
- TargetAddress明确表示目标地址（u64），不是宿主指针；位宽合同目前为1..64位，区间检查使用更宽算术避免地址溢出。未来更宽地址需版本化扩展此域，不改变Web或core。
- NativeSession只表达目标和声明能力，由调用方拥有资源寿命。MemoryRead、MemoryWrite为独立端口，可以用trait对象传入受检用例，读后端无需提供假写入方法。
- `read_memory`/`write_memory`为受检入口：核对目标引用、能力及目标地址范围后，调用域端口一次；请求长度与返回长度不等时报告错误，不伪报完整成功。
- MemoryError携带core Error、已确认transferred和读取失败时的read_data。Unknown的transferred只证明该部分已确认，不能用它推断其余字节没有变化。
- 端口实现是受信任的适配代码；直接调用底层read/write不会经过用例校验。未来宿主权限和进程保护的核验仍属于真实后端，不由接口声明替代。

只有`#[cfg(test)]`下的BufferSession使用本地Vec缓冲区。当前native crate及默认CLI内不存在真实进程访问、地址解引用、Frida依赖、unsafe或可对外返回模拟成功的CLI命令。原生脚本、模块加载及持续事件接口以后在native域扩展，不强制转换为页面JS或JSON。

## 平台与宿主

platform-macos::inspect_app返回通用的macOS应用包元数据：规范化可执行文件、Framework目录名称。hosts::electron使用这些元数据识别标准Electron并生成既有LaunchPlan。非macOS仍明确拒绝当前规划；计划不启动/重启/结束用户应用。

## 验收句柄

- application合同：同一Bundle送到不同测试后端时生成程序相同；错误目标/缺页面能力不执行；Unknown与source不丢失且只调用一次。
- 原生合同：无URL目标、只读端口、32/64位地址边界、读写/部分进度及Unknown无重放；只证明架构。
- CDP传输合同：地址限制、响应关联、脚本失败/编码失败、协议错误与发送后失联的确定性。
- 真实Electron fixture：从CLI验证页面发现、实际CSS/HTML/JS效果、更新/清理和宿主规划；不是任意WebKit/原生支持证据。
