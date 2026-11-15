# Injectra

面向桌面 Web UI 的**通用代码注入 CLI**：runtime连接已运行App执行JS/HTML/CSS；pre-runtime先准备少量dev引导，再使用现有编辑器修改外部增量，在真实App查看更新。

独立`prepare/dev`已实现；目录/ASAR/嵌套ASAR Electron夹具、Claude本地壳及QQ主页面通过当前验收，详见[范围与证据](docs/validation/cli-dev.md)。工作台、Studio和完整workspace已选择性删除；原生独立边界及历史代码/实验保留。

长期架构已建立执行域隔离：通用核心、Web／原生契约、可复用应用用例及平台／宿主职责分离。原生内存读写执行域仍仅有契约测试；已实现的Frida桥接用于Web页面执行，不等同于交付通用进程内存接口，见[架构](docs/architecture/boundaries.md)。

首个核心版本为 **Rust CLI + macOS Electron/CDP**：连接已运行且显式开放本地调试端口的目标，选定单个页面并执行注入，报告结果、失败或结果未知。Injectra负责接入与注入能力；使用者负责脚本内容、选择器、组件适配及渲染覆盖问题。管理平台和Agent不在当前范围，见[产品边界](docs/requirements.md)。

当前Mac扩展已在本工作树实现[可选Gadget后端](docs/frida-backends.md)，抖音真实页面及独立Electron36通过；CEF/NW.js独立宿主也已通过；WebKit后端、独立宿主与真实Tauri应用已通过。详细进展见[Mac交付](docs/macos-delivery.md)。

**MVP 已于 2026-10-31 经用户亲自验证通过**，交付版本为 `v0.1.0-mvp`。版本范围与证据见[交付记录](docs/releases.md)。项目采用 [Apache-2.0 许可证](LICENSE)。

## 快速入口

工具版本由 [mise.toml](mise.toml) 声明：Rust 1.93.0、Node 24.6.0。已有匹配工具可直接使用；Node用于开发验证和可选pre-runtime准备工具；编译后的runtime命令不依赖Node。Rust、测试与准备工具分别通过Cargo.lock及各package-lock.json锁定。

```sh
make build
make run ARGS="--help"
make run ARGS="targets --port 9222"
```

插件统一位于独立插件库（独立 web-ui-plugins 仓库内 README.md）；本仓库不保存插件。将所选插件manifest的路径设为`BUNDLE_MANIFEST`，再运行下列命令。`targets` 需要目标应用已开放 `127.0.0.1:9222` 的页面调试入口。确认输出中的页面身份后，把下面的 `TARGET_ID` 替换为对应 ID：

```sh
make run ARGS="apply --port 9222 --target TARGET_ID --bundle $BUNDLE_MANIFEST"
make run ARGS="remove --port 9222 --target TARGET_ID --id injectra.accent"
```

插件效果由插件库中的实现决定；再次 apply 替换同 ID 的已有内容；remove 删除拥有的节点并执行脚本登记的清理回调。CLI 成功输出 JSON，失败退出码为 1，参数错误为 2。

pre-runtime使用：先`make setup-preparation`，然后运行`injectra prepare --app "$APP" --output "$DERIVED_DIR" --data-mode isolated`；将输出artifact_file传给`injectra dev --artifact "$ARTIFACT_JSON" --bundle "$BUNDLE_MANIFEST"`。详细参数、共享数据和未知结果处理见[CLI说明](docs/dev.md)。

## 验证与源码导航

```sh
make setup-tests   # 安装锁定的测试依赖到仓库外
make check         # 格式、clippy、Rust/Web 测试、架构依赖、文档与差异检查
make test-electron # 启动并退出独立 Electron fixture，验证真实注入链路
```

- [开发与使用](docs/dev.md)：启动规划、脚本约定、缓存及验证边界。
- [需求与范围](docs/requirements.md)：已核对的分享需求和本轮 MVP 边界。
- [架构](docs/architecture/boundaries.md) · [代码地图](docs/architecture/code-map.md)。
- [应用平台与内核兼容性总表](docs/compatibility/current-matrix.md)：统一查看豆包、抖音等应用的版本、平台、内核、接入路线、结果与代码入口。
- [验收证据](docs/validation.md) · [任务](docs/tasks.md) · [文档入口](docs/README.md)。
- LaTeX论文：使用`make paper`；使用`make paper-en`，均在仓库外输出PDF。
- [软件注入参考实现](references/README.md)：按软件归档源码/启动样例，共享桥接只保留一份；可独立检查和统计，不参与产品构建或发布。

## 当前限制

页面操作仅连接显式回环端口或身份核对的会话，使用精确页面ID；apply/remove不自动启动或结束应用，不改应用包。Gadget受控启动是单独的显式命令。页面刷新或窗口重建后需由使用者重新选择目标并 apply；自动重注入、跨窗口恢复及应用控件适配不属当前交付。当前WebKit已在独立WKWebView和真实Tauri应用验证；macOS硬件范围仅Apple Silicon，Intel Mac不在支持或验证计划内；Windows/Linux尚无兼容性证明，详见[范围](docs/requirements.md)。

载荷是受信任的本地代码，具有目标页面允许的权限，并非沙箱。当前不提供任意JS撤销或一键回滚；已有remove与清理回调保留为受管理内容的辅助接口。更新是先移除旧内容再应用新内容，失败不会自动恢复旧版本。超时/断线表示执行结果不确定，不自动重试。

默认缓存根为 `${XDG_CACHE_HOME:-$HOME/.cache}/injectra`，可用 `INJECTRA_CACHE_DIR` 覆盖。构建、测试依赖、fixture profile/日志放仓库外；仓库只保存注入器源码、锁文件及最小测试数据，不保存可交付插件包。没有大型数据集、模型权重或远端部署。
