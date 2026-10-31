# Injectra

面向桌面 Web UI 的可插拔个性化工具。首个核心版本为 **Rust CLI + macOS Electron/CDP**：向显式开放本地调试端口的单个页面注入 CSS、HTML、JS，再移除受管理改动。当前不提供图形管理器。

**MVP 已于 2026-10-31 经用户亲自验证通过**，交付版本为 `v0.1.0-mvp`。版本范围与证据见[交付记录](docs/releases.md)。项目采用 [Apache-2.0 许可证](LICENSE)。

## 快速入口

工具版本由 [mise.toml](mise.toml) 声明：Rust 1.93.0、Node 24.6.0。已有匹配工具可直接使用；Node 只用于开发验证，编译后的 CLI 不依赖 Node。Rust 与测试依赖分别由 Cargo.lock、tests/package-lock.json 锁定。

```sh
make build
make run ARGS="--help"
make run ARGS="targets --port 9222"
```

`targets` 需要目标应用已开放 `127.0.0.1:9222` 的页面调试入口。确认输出中的页面身份后，把下面的 `TARGET_ID` 替换为对应 ID：

```sh
make run ARGS="apply --port 9222 --target TARGET_ID --bundle examples/accent/manifest.json"
make run ARGS="remove --port 9222 --target TARGET_ID --id injectra.accent"
```

示例会添加右下角缩放按钮；再次 apply 替换同 ID 的已有内容；remove 删除拥有的节点并执行脚本登记的清理回调。CLI 成功输出 JSON，失败退出码为 1，参数错误为 2。

## 验证与源码导航

```sh
make setup-tests   # 安装锁定的测试依赖到仓库外
make check         # 格式、clippy、Rust/Web 测试、文档与差异检查
make test-electron # 启动并退出独立 Electron fixture，验证真实注入链路
```

- [开发与使用](docs/dev.md)：启动规划、脚本约定、缓存及验证边界。
- [需求与范围](docs/requirements.md)：已核对的分享需求和本轮 MVP 边界。
- [架构](docs/architecture/boundaries.md) · [代码地图](docs/architecture/code-map.md)。
- [验收证据](docs/validation.md) · [任务](docs/TODO.md) · [文档入口](docs/README.md)。
- [豆包真实应用实测](docs/compatibility/doubao.md)：2.33.5定制Chromium宿主通过CLI注入与清理；现有核心无需修改。

## 当前限制

仅连接指定的回环端口与精确页面 ID；不扫描进程、不自动启动/结束用户应用、不改应用包。页面刷新或窗口重建后需重新选择目标并 apply；自动重注入、持久化配置、文件监听、iframe/Shadow DOM 定向适配、WebKit/Tauri 适配尚未实现。

载荷是受信任的本地代码，具有目标页面允许的权限，并非沙箱。任意 JS 副作用只有脚本显式登记清理时才可能撤销；清理失败会报错。更新是先移除旧内容再应用新内容，失败不会自动恢复旧版本。超时/断线表示执行结果不确定，不自动重试。

默认缓存根为 `${XDG_CACHE_HOME:-$HOME/.cache}/injectra`，可用 `INJECTRA_CACHE_DIR` 覆盖。构建、测试依赖、fixture profile/日志放仓库外；仓库只保存源码、锁文件和小样例。没有大型数据集、模型权重或远端部署。
