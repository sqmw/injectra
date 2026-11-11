# 阿里云盘测试副本资源补丁验证

2026-11-11，[T-035](../TODO.md#t-035)；基线168deca，aDrive6.9.3/6.9.3.1，macOS Apple Silicon。用户明确授权在测试副本修改资源并验证真实页面执行。[原包CDP/Frida失败](adrive-startup.md)保留，此次是独立的新路线。

## 已验证结果

**测试副本的真实本地登录壳页面执行成功。** CUA确认URL属于该副本的app.asar/dist/index.html、路由为#/login，页面上出现随机标识bfde8ea5e8及“Injectra 页面执行成功”。点击测试按钮后显示“交互验证通过”，控制端取得同一标识的三阶段回执。

| 阶段 | 启动后的时间 | 实际回执 |
| --- | --- | --- |
| loaded | 约3.692秒 | bootstrap在原HTML解析期执行 |
| applied | 约3.866秒 | 共享runtime返回applied；节点已连接；计算背景色rgb(53,32,107) |
| clicked | 约21.567秒 | CUA点击原生窗口中的测试按钮，页面更新文字，回环回执connected=true |

回执只携带本次nonce、无查询参数的页面路径及测试状态，不采集账号凭据。#root在静态HTML中本就存在，不能单凭appRootPresent证明业务初始化；本次登录页状态由独立AX/截图核对。外部登录iframe、登录后文件页、同步及主题插件均未验证，未进行登录操作。

## 实现与构建条件

- 原包页面HTML与dist/index.js可读。只在首个既有script前加入同步本地injectra-proof.js；它登记DOMContentLoaded回调，之后复用单一来源web/runtime.js挂载临时测试标记及按钮，不覆盖原#root。该时机早于HTML业务脚本，晚于Electron preload，不是主进程/任意原生Hook。
- 使用官方@electron/asar3.4.1逐项重建归档，保留原有unpacked、link、executable状态。原索引24361项，新索引24362项；显式资源替换是dist/index.html，新增dist/injectra-proof.js。相对原包，另外38个unpacked文件因原生重签或签名资源清单而变化，审计已逐项保存；不能把“两个入口资源”写成整个副本只变两个文件。
- 先对副本嵌套原生代码签名，再生成ASAR文件/块完整性与头哈希，更新副本Info.plist的ElectronAsarIntegrity，最后签外层包。保留ASAR完整性校验、fuses及Hardened Runtime。
- 本机无可用开发者签名身份，采用ad-hoc测试签名，不具有原厂Team身份。成功副本的进程签名设置disable-library-validation=true，允许加载本地测试签名库；这是明确改变的副本条件，不能称与原包保护条件相同。
- 独立HOME/CFFIXED_USER_HOME、Chromium profile及缓存；副本复制时排除包内sync_drive_app/data及syncapp.pid，不复制已发现的同步数据库/sidecar。成功启动另带--use-mock-keychain，避免Chromium测试依赖真实Safe Storage。该参数用于测试的背景见[Chromium官方构建说明](https://chromium.googlesource.com/chromium/src/+/main/docs/mac_build_instructions.md)。没有创建/重置用户钥匙串、读取密码或操作系统授权窗口；不能据此证明应用所有原生代码都不调用钥匙串。

独立目录属于部分隔离，系统服务仍可能共享。登录页仍显示桌面登录提示，本次未点击头像或继续登录，不宣称完整账户隔离或全部应用功能不受影响。

## 保留的失败与修正

| run（缓存根runs下） | 结果 |
| --- | --- |
| adrive-resource-MyzvZn | 准备阶段发现ASAR返回写入流尚未flush，末尾文件哈希不一致；校验拦住，未启动应用。修正为等待finished并加入3MB尾文件测试；原失败副本保留 |
| adrive-resource-T0PSdi | 副本严格签名校验通过，但保留原库验证条件的ad-hoc构建被dyld以Team ID不匹配拒绝，SIGABRT，0回执，未进入页面 |
| adrive-resource-Y9k4VL | 副本允许本地测试库加载，进入Electron；独立HOME下采样显示SecKeychainAddGenericPassword/AuthorizationCopyRights等待，90秒0回执。只核验采样当时；自有主进程不响应SIGTERM后经身份复核SIGKILL结束 |
| adrive-resource-ZEAQtJ | 新副本及新nonce，增加测试钥匙串参数；真实登录壳页面loaded/applied/clicked通过，原始result保留 |

Review补强了阶段顺序、同页面/nonce及实际颜色判据，另以review-verification.json复核原始成功回执，通过；没有覆盖旧result或重跑同一run。工具后续禁止同一已启动run重试，须重新prepare。回执最终统一落盘，避免并发覆盖；签名写入保护使用显式检查，Python -O不能移除。

## 复现入口与边界

以下是可选实机验证工具，不属于CLI公开后端。它会准备新的副本，并需要已授权正常退出普通实例才能运行；脚本拒绝已有同路径主进程，不自动管理用户实例。

```sh
make setup-resource-tests
make test-resource-patch
make prepare-adrive-resource
# 使用prepare返回的run目录；测试前正常退出普通阿里云盘
make test-adrive-resource ARGS="<prepared-run>"
```

控制器只监听127.0.0.1，最多观察90秒；在测试窗口点击“验证页面交互”取得回执。不会自动结束子树。准备副本、临时HOME、原始日志、完整性与签名记录均位于仓库外缓存根；默认`${XDG_CACHE_HOME:-$HOME/.cache}/injectra`。实际应用和测试依赖不入Git，验证载荷是临时夹具，不是插件。

源码导航：[启动与准备](../../scripts/adrive-resource-smoke.mjs)、[ASAR重建](../../tests/resource-patch/archive.mjs)、[副本签名](../../tests/resource-patch/sign-copy.py)、[页面标记](../../tests/resource-patch/page-proof.mjs)、[回执判据](../../tests/resource-patch/receipts.mjs)。5项可选测试覆盖[归档内容/权限状态与越界拒绝](../../tests/resource-patch/archive.test.mjs)、[回执误报](../../tests/resource-patch/receipts.test.mjs)及[优化模式下原包保护](../../tests/resource-patch/sign-copy.test.mjs)。

## 证据与恢复

成功run保存plan.json、launch.json、receipts.json、result.json、archive-difference-audit.json、signature-comparison.json、fuse-comparison.json、restoration-check.json与review-verification.json。原/副本fuse均为v1、7项、0100110，逐字节相同。运行后、review修改前补存source-before-review快照，明确不是启动时采集的源码哈希；生成的proof.js及runtime哈希与plan一致。后续工具在prepare/launch阶段自动保存各自源码快照，不回填旧run字段。

测试主PID79974通过应用菜单正常退出，原普通实例80119恢复首页/最近看过。UI退出核对期间短暂观察到另一个副本PID80085，其启动环境未记录，不并入成功证据；身份检查前它已退出，没有向该PID发送信号。最终成功副本进程无残留。原主程序、app.asar、Info.plist、CodeResources四项哈希一致；旧字段originalUnchanged仅指这四项，不代表整个包逐字节核对。原包既有同步运行文件保留。

当前结论：资源补丁路线在上述测试构建条件下成立。后续产品化、原厂签名身份、跨版本补丁、更新恢复及登录后能力都不由此次通过推出；当前正式产品交付仍为runtime注入。
