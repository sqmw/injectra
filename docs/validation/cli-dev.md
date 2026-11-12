# CLI dev验收

归属[T-040@v3](../TODO.md#t-040)，[范围](../architecture/dev-mode.md)。

## 自动验证

make check、make test-electron；make test-preparation验证两类自有Electron应用：目录/ASAR、原入口身份/相对解析/strict语义、JS/CSS/HTML连续更新、源码位置、异常后新版本执行、页面重建停止、准备产物摘要不变。夹具与临时增量在仓库外，无插件副本。

Bootstrap协议测试覆盖认证、跨实例身份、世代改变、执行前超时NotExecuted、发送后Unknown、旧回执、日志归属及不重放；CLI journal测试覆盖未决请求跨重启与精确确认。原runtime CLI独立回归，core/native独立构建。

## 实机

QQ与Claude使用同一准备器；数据模式分别遵循已有QQ shared授权及Claude isolated尝试。2026-11-12实际结果：

| 对象 | 结果 | 证据边界 |
| --- | --- | --- |
| Electron目录/ASAR夹具 | 通过 | 原入口身份、strict、相对解析、预检不执行原入口、多窗口隔离、两次增量、错误行定位、导航停止、关键资源摘要不变；ESM在创建副本前拒绝 |
| Claude4.23118.1 | 通过 | 同一prepare/dev，isolated派生副本，app.asar的.vite/build/index.pre.js；真实本地壳页面两版HTML/CSS/JS断言及清理回执Executed；Ctrl-C不退出App；未登录/验证远端聊天 |
| QQ6.9.86 | Unsupported | 主入口application.asar/app_launcher/index.js不是当前可直接读取的明文CJS文件；通用准备器在副本修改前拒绝，未新增品牌绕路；旧QQ主题实验不被本结果覆盖 |

Claude控制台事件未收到，页面内DOM/CSS断言和执行回执通过；不把可选日志缺失写成日志验收成功。首次Claude尝试等待console事件而超时（PID88127），随后新请求/PID88561改用页内断言与执行回执通过；未把前次日志等待失败改成成功。原用户实例没有被终止；仅测试新副本的精确PID在验证后经路径核对退出。

仓库外证据：runs/preparation-fixtures-iNzd88/下两类acceptance.json；runs/t040-cli-real/claude/e791b34e-911b-48a5-a345-6796f0f80f5b/acceptance.json。早期资源封印读取、摘要空slot及日志事件名调试失败由对应派生目录/日志保留，不回填为成功。原包未修改；关键资源摘要校验不等于整包逐字节核对。

## 恢复

平台删除提交a403982可独立审查/反向撤销；旧main基线85ce983、代码07d55a3及各platform工作树保留。仓库外recovery/t040-cli-20261112保存五树的HEAD、patch和源码tar；不能为恢复而覆盖已有工作区或删除运行证据。

## 提交前复核

Parent复核依赖图、与4cf736e的runtime源码差异、请求/文档身份、源捕获、未决落盘、复制/签名及退出边界。runtime执行域和原后端源码无差异；不存在Studio/完整workspace/CodeMirror产品依赖。复核修正了document/global nonce装配不一致、console事件名、嵌套辅助可执行文件识别、可选封印资源、未初始化摘要slot、入口相对require、未决JSON缺字段及交互选择Ctrl-C等问题；对应fixture/协议/持久化/文档检查已通过。不是独立审核或其他平台兼容证明。
