# macOS Electron/V8桥接参考

来源：抖音独立探针`622fe2e94e10164a083c36e34ff1ae3fdfedbc99`，[原文件哈希及迁移说明](origin.json)。参考过[Frida官方Node/V8示例](https://frida.re/docs/examples/javascript/)，Frida/Gadget使用官方现成组件；没有vendor第三方运行库。

- `agent/bootstrap.js`：进程/home回执与DYLD继承清理。
- `agent/v8.js`：macOS主队列、目标框架libuv/V8函数及native回调资源。
- `bridge.py`：Gadget通信、Node回执关联、页面内结果/异常封装。

两个agent与来源字节一致；bridge仅将agent路径改为相对自身模块，保留原调用签名以便比较。历史实测只覆盖记录中的抖音/Electron/V8/macOS arm64组合；不是其他应用或ABI已验证。

与应用名/页面选择分离，实际入口在[抖音案例](../../apps/douyin/gadget-v8/README.md)。源码是参考，不提供完整的断线、永不结束操作或所有初始化失败下的生产级资源保证。
