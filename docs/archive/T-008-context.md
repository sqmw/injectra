# T-008 关闭上下文

上下文r4（2026-11-03）：Parent为当前窗口；main起始990967b，初始工作区干净。Task定义与终态见[TODO](../TODO.md#t-008)；执行T-008/E-001及全部步骤见[归档](T-008.md#t-008)。

长期边界已按D-004建立，现有CDP迁移后实际验收通过；原生只有契约及测试缓冲区。阶段提交34906f7、570dcbe，最终验收/收口为本文件所在提交。公共core无Web/JSON/平台实现依赖，Web和Native端口分域；当前CLI仍只装配真实CDP。

验收入口make check、make test-electron、core/native及application独立构建均通过；原始Electron run为仓库外runs/electron-xZyk1B。当前无剩余任务或未集成产物。恢复后从[现行架构](../architecture/boundaries.md)和[执行域契约](../architecture/contracts.md)读取当前合同，不再使用迁移前评估中的推迟边界建议。
