# T-040 · Studio前端

[总体接口与归属](platform-v1.md)。本子任务只实现本地Web工作台，实际效果在目标App中展示。后端/CLI与整体验收由Parent集成，前端测试不能替代真实App兼容性或完整浏览器视觉验收。

## 入口与构建

源码位于studio-ui/src：types描述服务投影；model处理目标世代、未知结果和草稿确认；api处理认证；editor包装CodeMirror6；studio装配交互。只对user来源提供编辑/保存，application与injectra来源只读展示。后端仍须独立实施写入权限与路径校验。

```sh
node studio-ui/scripts/setup.mjs
node studio-ui/scripts/build.mjs
node studio-ui/scripts/check.mjs
```

建议Parent添加setup-studio-ui/build-studio-ui/check-studio-ui三个Make入口，分别调用上述命令。依赖通过package-lock锁定并安装到缓存根studio-ui/tools，生成产物默认在缓存根studio-ui/dist。缓存根继承INJECTRA_CACHE_DIR/XDG_CACHE_HOME约定；INJECTRA_STUDIO_ASSETS可指定其他构建输出目录。源码目录不生成node_modules或dist。

产物固定为index.html、app.js、app.css。Rust服务可从该目录读取或嵌入，URL必须分别映射为/、/app.js、/app.css；生产运行不需要Node。当前编译产物不带source map，不在Git中保存。

## API与交互

- HTTP使用Authorization Bearer；启动URL的#token=读取后立即从URL清除，凭据仅保留本页面内存。工作台内“刷新连接”复用凭据；完整浏览器刷新需重新打开CLI输出链接。凭据不写storage或日志，不放API/WS URL中。
- WebSocket连接/api/events后先发送authenticate帧，收到authenticated后才接收state/run/log。断线不自动重连或执行；用户点击刷新连接恢复。
- API文件path作为不透明标识原样回传；relative_path仅用于展示。GET文件返回content/origin/stage/hash。PUT携带expected_hash，409保留草稿且阻止覆盖；重新载入是显式操作，有未保存内容时确认放弃。保存期间的新编辑不会被响应覆盖。
- 保存与运行分开；默认显式运行已保存revision。可选保存时应用只对页面增量生效，startup保存始终显示等待重启。POST run/reload同时传目标generation；目标世代变化立即清空选择并暂停自动应用。
- Unknown暂停保存时应用，不发自动重试。先手动运行取得明确结果后，用户才能重新开启自动应用。多目标按当前选择与服务状态运行，不以全局“保存成功”冒充应用成功。
- 全局capabilities使用prepare/dev/restart/export/workspace.import；目标使用web.page.evaluate/web.page.reload。缺少能力的按钮禁用，不以UI空实现伪装支持。prepare必须明确选择data_mode，未选择时不提交。
- 原App基线、工具引导/修改清单、用户增量分别列出。错误文件/行号在已知文件中提供定位；原始结果和错误正文只通过textContent显示，不使用innerHTML或执行返回内容。文件内容仅交给CodeMirror作为文本。

## 验证边界

check命令运行严格TypeScript检查、真实CodeMirror构建及10项测试：认证凭据范围、WS首帧/认证顺序、目标世代失效、Unknown不重放、保存时编辑竞争、只读来源、草稿冲突、版本/目标提交及启动增量待重启。

DOM交互测试使用JSDOM和受控编辑器替身，验证工作台流程；不伪称真实CodeMirror排版或目标App执行已验收。Parent需在集成服务后补真实浏览器视觉和端到端检查。当前无原App进程操作、没有新增插件或业务载荷。
