# QQ、腾讯视频、Claude资源补丁顺序验证

2026-11-11，[T-038](../TODO.md#t-038)，基线b2674dd；用户指定按QQ → 腾讯视频 → Claude顺序尝试[T-035](adrive-resource-patch.md)的测试副本路线。所有实际启动串行，原安装包及原QQ/Claude实例保留；以下不是正式产品后端支持声明。

后续用户明确接受QQ共享原数据，[T-039粉紫主题预览](qq-pastel-preview.md)据此完成；独立账号隔离不再作为该后续任务的验收条件。以下旧实验的事实与条件保留。

## 实际结果

| 应用 | 结果与边界 |
| --- | --- |
| QQ6.9.86 | 副本启动清单加载器成功向实际窗口执行共享runtime；可见nonce c747c33a0e及交互通过，收到loaded/applied/clicked。但副本读取了现有账号界面，独立HOME没有隔离共享容器；不能计为完整账号隔离验收。新身份/去共享组重试在沙箱Mach端口注册时SIGTRAP，未进入页面 |
| 腾讯视频2.164.0 | 修改本地hd_lottie/demo.html并重签的副本可打开首页；观察60秒，0回执、无验证标记。没有确认该动画HTML被当前首页加载，当前候选未成功；不能据此断言所有资源补丁路线不可能 |
| Claude4.23118.1 / Electron44.9.3 | 更新ASAR及Framework内的完整性摘要后，真实本地main_window/index.html壳页面成功执行；nonce 654d6504ac、计算紫色样式及CUA点击后的交互文字/回执齐全。远端claude.ai登录视图是另外的Web内容，未注入、未登录，未验证聊天/其他业务 |

QQ点击回执的操作者未记录；CUA随后观察到同nonce和“交互验证通过”，没有将该点击写成Agent操作。QQ保留旧身份的结果只证明执行通道，不作为安全隔离方案推广。后续当前工具采用新Bundle ID和移除副本共享组权限；这会使当前QQ配方在沙箱启动阶段失败，不能因旧条件通过而隐藏此限制。

## 入口与修改

- **QQ**：实际包为Contents/Resources/app/application.asar，renderer HTML和入口JS不是可直接编辑的明文。没有尝试解密或替换这些原文件；修改副本明文package.json的main，加入启动包装器，保留原main加载。包装器在新BrowserWindow的dom-ready通过Electron WebContents API送入最小验证代码。属于启动资源补丁，未使用CDP或Frida；不是直接修改已加密HTML。
- **腾讯视频**：当前没有常见app.asar，包内发现动画HTML及Cocos脚本。只修改实际存在的hd_lottie/demo.html并加入同目录验证脚本；没有伪造首页或打开该HTML冒充原应用页面。原首页可见但没有标记，lsof只作文件句柄快照辅助，不以未列出证明永不加载。
- **Claude**：修改app.asar内.vite/renderer/main_window/index.html并添加同目录bootstrap。Electron41+还校验Framework中的ASAR元数据摘要，只更新Info.plist不够。按[官方packager实现](https://github.com/electron/packager/blob/21c5955cc3877e93d268ceaae7b1beae673aced7/src/mac.ts)重算排序后的key/algorithm/hash摘要，校验旧slot后写入新摘要，保留used=1/version=1，再对Framework及外层签名，没有关闭ASAR校验或翻转fuse。只重建app.asar，不外推ion-dist.asar相关功能。

三款均使用仓库外新副本、本地ad-hoc测试签名、独立HOME/profile及测试钥匙串参数。后续副本采用唯一Bundle ID，并从副本进程签名移除共享应用组、钥匙串组、原厂身份字段及get-task-allow，保留其他原权限并允许本地测试库加载。独立路径和参数不等于所有宿主都能隔离账号：QQ原生启动日志仍使用原容器路径，因此不能宣称本轮完整隔离或所有账号数据零访问。Agent未读取凭据、登录、发送消息或操作账号业务。

## run与保留证据

缓存根默认`${XDG_CACHE_HOME:-$HOME/.cache}/injectra`，以下均在runs/下，不进Git。

| run | 实测 |
| --- | --- |
| resource-patch-JMTqiT | QQ准备工具的plist更新代码缩进错误，未启动；修正后建立新run |
| resource-patch-lMfkFJ | QQ旧身份/原共享组条件；实际页面app://./renderer/index.html及其他窗口取得回执，标记可见；原账号界面也可见，范围收窄如上 |
| resource-patch-u8IukO | QQ新Bundle ID、去共享组；严格签名通过，但bootstrap_check_in MachPortRendezvousServer Permission denied，SIGTRAP/0回执 |
| resource-patch-oQlqiV | 腾讯视频新身份副本；真实首页可见，0回执，候选HTML未证实执行 |
| resource-patch-FnTVId | Claude仅更新Info.plist；Failed to get integrity for validatable asar archive，SIGTRAP/0回执 |
| resource-patch-rjXnDO | Claude更新Framework摘要后，loaded/applied；中文乱码。CUA点击发生于60秒监听结束后，UI交互文字变化，但原result仍为false/2回执，不追补成功 |
| resource-patch-qF7g7o | 验证脚本改为ASCII Unicode转义并使用180秒监听，新副本真实本地壳页面中文正常，loaded/applied/clicked三阶段与CUA结果通过 |

每个已完成run保留plan、proof、launch、startup.log、result及final-audit；部分有sample/cleanup/摘要记录。source/是准备阶段快照，不冒充启动时控制器哈希；期间控制器观察时限等修正未回填旧run。原result不覆盖。当前回执只能按路径/nonce区分，QQ同路径多窗口不能据此独立断言WebContents身份；本轮另有CUA可见标记核对。

## 现场与恢复

QQ旧条件副本、腾讯视频副本经路径/启动时间/profile核验后TERM结束；Claude乱码副本不响应TERM，核验后KILL自有主进程；最终Claude副本使用应用菜单退出。未结束原QQ/Claude进程。收尾时所有上述副本路径下均无进程，原QQ PID69729、Claude PID680保留原启动世代，各run记录的原关键文件哈希均匹配。该哈希集合不等于整个包逐字节核对。

随后CUA只核对原QQ窗口，未见Injectra标记残留；没有操作聊天控件或保存其账号/聊天内容到仓库。

Claude测试副本菜单曾显示Downloading Update，未由Agent触发或接受安装；短期副本均已结束，未把更新行为视为受控实验。原关键文件核验没有发现变更，不能据此推广为永久安全的日常运行副本。

## 工具、验证与恢复入口

[批次准备/启动](../../scripts/resource-patch-smoke.mjs)复用[ASAR重建](../../tests/resource-patch/archive.mjs)、[副本签名](../../tests/resource-patch/sign-copy.py)、[共享runtime测试输入](../../tests/resource-patch/page-proof.mjs)、[回执判据](../../tests/resource-patch/receipts.mjs)；新增[摘要更新器](../../tests/resource-patch/integrity-digest.py)。只保存最小测试载荷，不复制实际插件，不注册产品后端。

```sh
make setup-resource-tests
make test-resource-patch
make prepare-resource-app ARGS=claude
make test-resource-app ARGS="<prepared-run>"
```

配方可选qq/tencentvideo/claude；当前qq隔离启动失败、腾讯候选未证实加载的限制同样适用于这些复现入口。运行前应确认该副本没有已有进程；控制器不会自动退出或清理子树，结束后由控制端核验自有进程再正常退出。不关闭原会话。

make check通过；资源工具9项测试通过，包括[继承编码与默认页面边界](../../tests/resource-patch/page-proof.test.mjs)、[摘要多slot更新及异常拒绝](../../tests/resource-patch/integrity-digest.test.mjs)。这些测试不替代实机兼容证据。

外部参考保留于缓存根research/electron-packager，官方浅克隆/稀疏src，提交21c5955cc3877e93d268ceaae7b1beae673aced7；只定向读取src/mac.ts的摘要计算与存储格式，未整体引入项目或执行该仓库代码。用于上述打包元数据实现复核，源码不参与本项目构建。
