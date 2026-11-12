# QQ粉紫主题资源补丁预览

2026-11-11，[T-039](../TODO.md#t-039)。用户明确接受共享原QQ数据，要求实现粉色、紫色主题。因此本轮不再以独立账号隔离作为验收条件；[T-038历史实验](three-app-resource-patch.md)仍保留原条件与结果。

## 已交付与实机结果

主题归独立web-ui-plugins插件库（库内说明：`plugins/qq-pastel-themes/README.md`），manifest为plugins/qq-pastel-themes/bundle/manifest.json，id为qq.pastel-themes。Injectra没有保存插件或Bundle副本，只负责生成并在运行中加载外部程序。

QQ6.9.86共享数据副本的主页面app://./renderer/index.html实测：

| 选择 | 侧栏背景 | 列表背景 | 聊天面板背景 | 结果 |
| --- | --- | --- | --- | --- |
| 粉色 | rgb(248,220,233) | rgb(255,237,245) | rgb(255,245,249) | CUA可见、实际按钮选择及计算色值一致 |
| 紫色 | rgb(230,220,248) | rgb(240,233,252) | rgb(247,243,255) | CUA可见、实际按钮选择及计算色值一致 |
| 原样 | 恢复原生样式 | 恢复原生样式 | 恢复原生样式 | 主题关闭、原样按钮选中，未替换输入节点 |

三种状态editorSame均true；夹具进一步覆盖未发送草稿、原生DOM及状态色不变。当前实机为可见空聊天面板，未发送消息或切换会话，不把气泡token映射与所有消息格式视为实测通过。最终保留粉色预览窗口，右上可切紫色或原样。

## 加载器与开发边界

[qq-theme-preview](../../scripts/qq-theme-preview.mjs)调用已有资源准备器的`qq --shared-data`：保留QQ的Bundle ID与原共享组，使用新副本及测试签名。`--shared-data`仅为QQ提供；其他配方仍使用既有隔离尝试。不修改原安装包，也不新增管理平台、正式后端或自动恢复能力。

新副本的明文启动包装器保留原入口，定位新窗口的QQ主页面，再经WebContents执行外部程序。程序由既有[共享生成器](../../references/shared/web-program/README.md)调用产品Bundle/runtime生成，插件源不进入Injectra。QQ沙箱不能任意读写父缓存目录，开发控制器通过带随机路径的127.0.0.1端点提供当前程序并接收限定的结构/色值报告，不读取聊天正文或凭据。

开发期间支持显式更新仓库外program.js；文件内容hash变化后重新apply同一Bundle，复用已有cleanup合同。验证结束写入finish-preview后停止开发轮询并关闭控制器，已运行主题和控件继续留在窗口。这是开发预览，不是重启后自动加载的安装方案。

```sh
make -C references program
# 使用共享生成器apply <外部manifest>，将输出保存为仓库外QQ_PROGRAM
node scripts/qq-theme-preview.mjs --prepare "$QQ_PROGRAM"
# 验证完成后保留当前窗口，停止预览开发轮询
touch "<returned-run>/finish-preview"
```

普通主流程只接受现有manifest，粉紫配色、控件及QQ选择器全部留在插件库。主题本身没有网络请求和应用包修改逻辑。

## 证据、修正及现场

缓存根runs/resource-patch-ucyD7f：主PID90625、WebContents2。page-2.json只保存结构类名/颜色token/尺寸/主题状态；pink-check、purple-check、original-check及theme-validation保存各状态与原关键文件核对。没有把聊天截图或正文提交仓库。截图只作会话中的可见验证。

首次预览准备读取输出字段app的假设错误，未启动；后续IqBCbS副本因沙箱外文件读写无法完成预览通信，自有PID90118经身份复核TERM结束。改用回环报告后建立ucyD7f新预览，不覆写旧准备记录。

视觉/回归修正包括：覆盖侧栏内层深色背景；灰色未读角标增加可读前景；存储不可用时通过窗口内偏好保留重载选择。原生红色状态与头像/图片不重着色。plugin重复安装、恢复/卸载、输入与对比度测试通过。

用户要求共享数据属于本轮接受条件，不据历史隔离失败阻止主题实现。Agent没有向原QQ主实例69729发送退出操作；收尾时该PID已不存在，退出者/时点未采集，不补造原因或重新打开它。新窗口90625保留供用户查看。原主程序、Info.plist、CodeResources、package.json及application.asar五项hash与准备前一致，不声称原账号运行数据不变化或整个应用包逐字节核对。

插件库make check共47项通过（新增QQ5项），Injectra make check/资源工具回归与文档检查按任务记录保存。当前范围只覆盖macOS Apple Silicon QQ6.9.86主页面，其他版本未验证。
