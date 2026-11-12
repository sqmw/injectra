# QQ通用CLI dev复验

2026-11-12，[T-042](../TODO.md#t-042)。承接用户对T-040 Unsupported结果的纠正；不改写当次失败或旧主题实验。

## 根因与通用修复

原QQ6.9.86-42941的package.main是application.asar/app_launcher/index.js；归档正常可读，入口80字节不是普通UTF-8源码。旧准备器按磁盘路径读取，尚未进入宿主加载器便拒绝。另将引擎路径写死为Electron Framework，未覆盖具有相同com.github.Electron.framework元数据、文件名不同的框架。

现在按资源布局识别嵌套ASAR，入口策略分两类：

| 策略 | 行为 |
| --- | --- |
| cjs-insert | 明文CommonJS保留原入口与directives，在源码入口插入引导 |
| main-delegate | 嵌套归档或不透明JS/CJS保留原字节，修改包的main为工具shim；shim先载入Bootstrap，再通过宿主Node Module._load(original,null,true)将原入口作为主模块加载 |

委托不解密/替换QQ代码，不按应用名选择。Module._load是Node内部宿主API，后续Electron/Node变动需重新验证；不宣称任意专有加载器均兼容。具备可识别ESM条件的入口仍明确拒绝。引擎按框架CFBundleIdentifier和CFBundleExecutable发现，不靠目录品牌名称。

## 实机证据

同一CLI prepare/dev、shared数据模式、新派生副本，原QQ进程和安装包保留：

- prepare预检成功；真实renderer/index.html页面完成第一版JS/HTML/CSS、第二版增量与清理，三次回执均Executed且无错误。页内断言分别核验工具节点文本与计算色值，不读取聊天正文、不发送消息。
- 当前QQ程序归档与原始application.asar逐字节摘要相同；源package.json、Mach-O、Info.plist和归档均匹配准备基线。开发更新不修改所核查的准备资源；不声称全包逐字节扫描。
- Ctrl-C结束CLI后派生App仍在；验证结束仅经可执行路径核对后退出本次PID95704。未终止原QQ实例。
- 证据位于仓库外runs/t042-qq/87518483-8b6c-4991-9017-3d7910764d5f/acceptance.json；原包、pairing/config密钥不进入Git。

## 回归与边界

入口合同测试覆盖委托后的require.main/process.mainModule/module.id/module.parent、argv、相对模块及引导先于原入口；目录、ASAR、嵌套ASAR三类真实Electron夹具覆盖相同CLI更新、多窗口隔离、异常、导航停止和免重打包。嵌套夹具断言原归档摘要不变。

这证明同一通用策略已覆盖QQ主页面；主题与业务控件仍归外部插件库，不新增QQ执行代码或平台。当前源码：[入口策略](../../crates/preparation/helper/entry.mjs)、[平台识别](../../crates/preparation/helper/platform.py)、[合同测试](../../crates/preparation/tests/entry.test.mjs)。

提交前复核：Parent检查原入口未改、委托主模块/argv/导出语义、框架识别无品牌路径、原包摘要、协议复用与旧记录隔离；修正package.json修改清单before摘要，三个入口单测及三类真实Electron夹具通过。entry_strategy为新增可选来源字段，旧准备记录仍可读取。
