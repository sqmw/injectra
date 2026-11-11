# 阿里云盘pre-runtime入口核查

后续进展：[启动条件与观察器复核](adrive-startup.md)，含新的受控启动观测、线程采样和当前解锁恢复点。本页保留初次静态核查阶段的证据。

2026-11-11，[T-034](../TODO.md#t-034)，基线abbf9da。用户通过侧聊要求继续探索阿里云盘并把pre-runtime纳入候选，由主窗口执行。该阶段安装包为aDrive6.9.3；只读检查安装包元数据、有限入口封装、官方资料及可见菜单，没有启动新的注入试验。

## 当前发现

| 候选 | 已取得证据 | 尚缺什么 |
| --- | --- | --- |
| 页面初始化前preload | 包内存在mainWindow/authWindow/transferWindow三个JS封装，分别引用同名compiled.jsc及bytenode | 未找到向这些窗口注册外部preload的受支持接口；文件存在不证明可替换或已动态验证 |
| 主进程启动入口 | package.json的main为dist/main/index.js，该封装引用index.compiled.jsc | 未取得客户端提供的外部启动扩展配置；未解码或替换受保护载荷 |
| plugin.json | resource/package/aliyun-drive-sync-drive-app/darwin/plugin.json描述内置syncapp0.2.16的启动/安装/升级/停止生命周期 | 这是同步组件清单，未取得允许第三方注册任意页面代码的合同；没有运行其中的命令 |
| 官方三方应用 | 官方页面列举基于开放接口的独立客户端、播放器、挂载工具等 | 未取得个人版macOS客户端的preload注册或UI注入文档；不能以云API支持代替桌面进程接入 |
| Beta/开发者产品资料 | 检索到官方Beta宣传页及PDS开发者版资料 | Beta不等于调试构建，PDS开发者版也不等于当前个人客户端；未据此替换安装包或宣称可调试 |
| macOS外部接口声明 | 未声明文档类型扩展、AppleScript开关/脚本字典，资源目录未见sdef；已声明smartdrive URL scheme | 未取得通过这些接口注册早期代码的合同，不假设URL scheme可执行任意代码 |
| 可见菜单 | 帮助菜单为访问主页/帮助中心/在线咨询；应用菜单含常规偏好、更新和退出项 | 未见明确调试/扩展入口。尝试点击偏好设置后未观察到设置页，不能宣称全部偏好内容已排除 |

没有找到新的、经过确认的外部代码早期加载入口，因此当前原客户端的注入目标仍未实现。用户已说明暂时没有官方开发/调试构建或客户端扩展文档；后续实机验证依赖能够通过明确接口注册早期代码的目标接口或可调试构建。这是当前输入/入口缺口，不是证明不存在任何可能方案。

## 时机与接入条件

[Electron WebPreferences](https://www.electronjs.org/docs/latest/api/structures/web-preferences)的preload由宿主配置，在页面其他脚本之前加载；这不自动赋予外部进程修改该配置的能力，执行上下文也需单独核对。本轮仅确认封装文件引用，未观测它们在原客户端的运行过程。

已有Gadget尝试本来就从进程启动期加载代理；把目标称为pre-runtime，不会改变此前[加载/附加条件](adrive-frida.md)。已观察到的Node开关、ASAR完整性与库加载保护继续是实际约束；本轮没有通过更换执行时机绕开它们，也没有替换preload、组件清单或应用签名。

没有运行自有fixture作为替代“成功”。自有Electron宿主可以用于未来验证preload时序和契约，但其通过不能消除阿里客户端入口缺口。

## 来源与证据

- 本地只读证据：缓存根runs/adrive-preruntime-4mispd0u/metadata.json、entry-classification.json及external-interface-metadata.json。缓存根默认`${XDG_CACHE_HOME:-$HOME/.cache}/injectra`。包内路径均为应用资源相对路径；未读取账号凭据或访问package元数据引用的内部私有仓库。
- 官方[三方应用页面](https://www.aliyundrive.com/download/externalApplications)提供应用/开放接口介绍，本轮未找到其中的桌面preload注册合同。正文通过搜索索引可读，直接网页提取无正文，检索覆盖并非穷尽。
- 官方[Beta页面](https://www.aliyundrive.com/custom/download/webbeta)只提供测试版功能说明；不据此推定调试入口开放。
- [PDS自助生成桌面端](https://help.aliyun.com/zh/pds/drive-and-photo-service-dev/user-guide/automatically-generate-a-desktop-application/)是另一产品/域管理能力，不作为个人版aDrive可注入的证据。

界面操作仅打开并关闭菜单、尝试查看偏好；未更改设置、启动组件命令、发送咨询、修改原包或系统保护。最终观察仍为普通阿里云盘主页。产品源码和依赖无变更；文档/追踪/差异校验仅验证记录结构，不代表入口已可用。

本轮自审与记录检查：原包只读、未执行组件脚本、未访问内部仓库；时机与接入权限分开，未将未检索到文档写成接口绝对不存在。make doc-check、任务追踪及git diff --check通过；追踪引用已按Task锚点规范修正。任务保留blocked，等待可验证的目标侧入口或构建。
