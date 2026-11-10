# 版本与交付映射

## T-017 · macOS多引擎开发交付（未发布）

- 当前窗口工作分支交付，尚未合入main或推送；既有MVP tag不移动。CLI包版本暂沿用0.1.0，新增能力以本分支提交及--help为准，不把旧tag当作新后端版本。
- 1c02dca建立基线，6c1655b交付Electron Gadget，5f2d013补CEF/NW.js合同；WebKit及最终收口对应本节所在提交链。
- 默认CDP保持兼容，新增electron-gadget/webkit显式后端；可选依赖安装和使用见[Frida说明](frida-backends.md)、[WebKit说明](webkit-backend.md)。
- 验收与限制见[Mac交付](macos-delivery.md)和[当前矩阵](compatibility/current-matrix.md)；未创建安装包或GitHub Release。源码回退按独立提交处理，不自动撤销页面副作用或删除运行证据。

## v0.1.0-mvp · 2026-10-31

- 状态：用户已亲自验证并确认MVP通过；本次交付为源码与Git版本标识，无安装包或GitHub Release附件。
- 仓库：[sqmw/injectra](https://github.com/sqmw/injectra)。
- 精确源码映射：Git tag `v0.1.0-mvp`。用 `git rev-list -n 1 v0.1.0-mvp` 获取该交付提交；不因main后续演进而移动既有标识。
- 核心实现：`ba1e003`；真实豆包兼容性与验证命令：`bc4bff9`；用户验收与仓库初始化历史在交付tag中一并保留。
- 远端原初始化提交 `5ba11c2` 的Apache-2.0许可证已保留并合入，没有覆盖远端历史。
- 功能：显式本地CDP页面发现、共享CSS/HTML/JS注入、相同ID更新、受管理清理、macOS标准Electron启动规划。
- 证据：[自动验证与用户验收](validation.md)、[豆包2.33.5实机报告](compatibility/doubao.md)。
- 边界：豆包为定制Chromium147，不计作标准Electron覆盖；自动重注入、GUI及WebKit适配未实现；任意JS副作用无法保证完全回滚。
- 恢复：源码可检出上述tag；运行页面改动使用remove清理；调试启动的应用应正常退出后无参数重启。Git回退本身不撤销正在运行页面中的副作用。
