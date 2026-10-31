# 核心 MVP 验证

## 已获得证据

2026-10-31，macOS arm64；Rust 1.93.0、Node 24.6.0、Electron 45.2.1、Chrome 156.0.8078.75。

首轮`make test-electron`已通过10项检查：页面发现、实际CSS计算样式、HTML挂载、JS按钮交互、重复应用不累积节点、清理、重复移除、错误目标拒绝、脚本失败清理、macOS启动规划。
证据目录相对默认缓存根：`runs/electron-0plevJ/`，result.json与electron.log保存实测环境及结果。

最终代码回归：Rust 6项、Web 5项测试全部通过；clippy（-D warnings）、格式检查与构建通过。第二次Electron 10项检查通过，证据为`runs/electron-oLBuRL/`。make run ARGS="--help"真实命令入口通过。

文档检查：13份Markdown、73条本地链接及可达性通过（归档后数量可能变化）；任务结构检查解析2个Task、2个Execution、3个Step、2个Decision通过。Cargo metadata核对core无具体适配依赖。

## 验证覆盖边界

- 真实注入发生在本项目启动的独立Electron fixture，未修改或接入用户正在使用的应用。
- 地址/能力拒绝与部分异常路径来自测试替身；不冒充所有真实应用都已验证。
- 追加真实产品证据：[豆包2.33.5兼容性](compatibility/doubao.md)，定制Chromium 147真实chat页面通过；不计作标准Electron覆盖。
- 没有WebKit、其他OS、复杂iframe/Shadow DOM、受限CSP、导航重注入或长期会话证据。
- 单元测试和文档检查不证明任意JS完全可回滚；生命周期与结果未知边界见[开发说明](dev.md)。

## 调整与review记录

- jsdom 30.1.3要求更高Node次版本，初次锁定出现engine警告；在安装运行前改用支持现有Node的26.1.0并重新生成锁文件。
- review识别同ID异步注入可能重叠，加入页面内busy门并用并发负例验证。
- 默认构建与测试缓存移出同步源码工作区；fixture仅拥有自己的进程，不结束用户进程。
- 提交前review已检查依赖图、核心契约/适配方向、路径限制、错误处理、生命周期、源码差异与文档。未发现当前合同内阻塞项；已知限制为非原子更新、任意JS不可保证撤销、超时结果未知以及只有单一Electron版本实测。详细执行证据由[任务入口](TODO.md)定位。
