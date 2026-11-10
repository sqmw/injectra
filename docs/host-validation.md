# CEF与NW.js独立宿主验证

属于[T-017](TODO.md#t-017)，[阶段证据](STEPS.md#t-017)。make setup-host-tests按[下载清单](../tests/hosts/downloads.json)获取官方Mac arm64客户端；CEF约136MB、NW.js约198MB，解压与profile留在外部host-tests/runs缓存。许可随官方发行包保留，来源/版本/摘要由清单固定。删除缓存后可重新准备，不依赖已安装用户应用。

make test-hosts分别启动自有CEF和NW.js进程，用回环HTTP提供最小页面，运行共享WebBackend合同，再退出创建的进程。HOME、profile、CEF根缓存均独立；无第三方插件或用户账号参与。

2026-11-10：CEF156.0.26/Chromium156.0.8078.88与NW.js0.119.2通过。验证包含发现、JS/CSS/HTML、点击读回、Promise、异常、同ID更新、移除、错误目标、超时Unknown及后续恢复；精确结果见外部run cef-qdvrcqtq和nwjs-ypuq07xm。

首次失败保留于cef-p1s2ajmh/nwjs-_54yeoc6：循环对象被CDP报告为协议错误；原有合同规定协议错误保守Unknown，因此修正新增测试的过强分类假设，不改变产品错误处理、不把未知结果改写为已执行。Gadget有页面内编码回执时仍要求ResultEncoding/Executed。

自有宿主证据不等于所有CEF/NW.js应用可达，原真实应用未验证条目继续保留。
