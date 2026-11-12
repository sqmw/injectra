# T-040 · 通用平台v1实施

[权威Task/Execution](../TODO.md#t-040)。Parent/main，代码基线4cf736e，分派基线4cb6bea，上下文r2。按用户批准计划实现，当前未完成；不得把局部模块通过当整体交付。

## 实施与分工

顺序：来源/工作区模型 → 准备器与Bootstrap → Studio/CLI/前端集成 → 导出与实机验收。独立工作树隔离写入，Parent负责合入、任务状态和整体验收。

受控分工均绑定T-040@v1 / E-001：platform_workspace在managed worktree platform-workspace实现workspace；platform_bootstrap在platform-bootstrap实现准备器/桥；platform_studio_ui在platform-studio-ui实现前端；adrive_integrity_audit在platform-studio-server实现Rust服务/CLI。各树从4cb6bea派生，任务状态的副本只读，Parent/main唯一更新。工作树绝对位置从本chat附件发现，不写入可复用配置。

- 工作区模型：新injectra-workspace crate，用户源码只在显式外部目录；缓存/不可变快照在外部cache。manifest v1兼容；startup.cjs独立于页面载荷；来源application/injectra/user。
- 准备器与Bootstrap：新injectra-preparation和injectra-bootstrap模块/crate；按CJS入口、目录/ASAR、实际摘要格式选择，不按品牌分支。仅派生副本。保持入口身份/解析/顺序。startup同步且一次启动一次；失败停止原入口；支持独立导出。
- 前端：studio-ui，TypeScript/CodeMirror6/esbuild，依赖锁定并在缓存安装。真实App呈现效果；界面只显示文件/来源/目标/执行/日志。默认显式运行。
- Parent：Axum本地服务、CLI装配、跨后端调用、完整回归与实机；core/native不得被Web假设污染。

前端实现与产物集成见[Studio UI说明](studio-ui.md)。

## 接口对齐

前端调用 /api/state、/api/files?path=、PUT /api/files（path/content）、POST /api/run（target/revision）、POST /api/connect（backend/port/session）、POST /api/prepare（app/data_mode）、POST /api/dev、POST /api/reload、POST /api/restart、POST /api/export。/api/events提供WebSocket事件。HTTP统一返回JSON；失败含error/message/certainty，暂不支持操作返回明确错误，不能假成功。

state投影包含workspace{id,root,revision}、files[{path,origin,stage,hash}]、targets[{id,title,url,generation,capabilities}]、runs[{request_id,target,revision,status,result,error}]、artifact、startup状态。新增字段允许；前端不解析品牌名称。浏览器凭据从启动URL fragment接收，仅同源且认证访问API。

对齐补充：workspace capabilities为字符串数组workspace.import/prepare/dev/restart/export；目标能力web.page.evaluate/web.page.reload。GET files返回path/content/origin/stage/hash；PUT接受expected_hash，外部改写时冲突而不覆盖草稿。POST /api/import只导入到新空工作区。run返回202及request_id；事件为state/run/log。WS先发authenticate token，认证前不发送数据，5秒截止并核对Origin。startup单列saved_revision/applied_revision/status/pending_restart。

工作区库与Bootstrap库的Rust接口由对应实现者在首次可编译时发给Parent，Parent唯一适配装配层；不得反向引入CLI或具体后端。Workspace必须验证路径越界/符号链接、输入大小，并让快照绑定内容及来源而非仅mtime。Bootstrap按目标分别确认revision，Unknown不重放，旧世代拒绝。

服务地址首次分配后持久化，重启复用，冲突报错。Bootstrap通信使用内置Node HTTP长轮询（避免目标额外安装ws）与角色/随机挑战认证；浏览器事件使用WebSocket。认证与目标注册由bootstrap库封装供Studio合并，需报告已准备/连接/执行不同阶段。

## 验收与约束

以用户批准的七项标准为整体验收：目录/ASAR夹具及至少两真实App；外部载荷修改不改变准备产物；原入口身份与启动语义；多目标/旧回执/断线；三类来源；开发服务关闭后导出独立运行；make check、独立Electron与新协议/UI测试。

现有QQ共享数据授权保留，其他应用由工作区配置明确选择。原用户会话不得自动结束。32独立方案阈值不变；本轮静态注册。插件仍归独立插件库，仓库只保留最小夹具。原历史实测与tag不改写。

工作区模型接口与边界：[workspace v1](workspace-v1.md)。
Bootstrap API and protocol: [bootstrap-protocol](bootstrap-protocol.md).
