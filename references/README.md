# 软件注入参考实现

用于源码参考、历史复核和代码统计，**不是产品后端目录**。不加入根Cargo workspace，不参与默认产品构建/测试/发布；不把归档检查当作新的应用兼容性证明。根make check仅随文档导航验证本目录Markdown链接，不执行参考代码或安装其依赖。

| 软件/路线 | 历史范围 | 当前入口 |
| --- | --- | --- |
| 抖音 Gadget/V8 | 8.7.0真实页面、点击读回及13项受控检查；7.9失败和版本漂移另记 | [案例](apps/douyin/gadget-v8/README.md) |
| Figma CDP | 126.5.6外壳注入；编辑器未验证 | [案例](apps/figma/cdp/README.md) |
| 优酷 CDP | 9.2.73容器profile下基线载荷通过 | [案例](apps/youku/cdp/README.md) |
| WorkBuddy CDP | 5.9.3指定宿主目录后基线载荷通过 | [案例](apps/workbuddy/cdp/README.md) |

共享参考：[Electron/V8桥接](shared/electron-v8/README.md) · [Web程序生成器](shared/web-program/README.md) · [现有CDP CLI调用](shared/cdp/README.md)。它们有复用潜力，目前不是跨应用/版本/平台通用实现。

## 独立检查与依赖

```sh
make -C references stats
make -C references setup
make -C references check
# 仅需要人工实机验证时显式准备二进制：
make -C references gadget
```

setup只准备专用uv Python3.13环境、Frida17.25.2与锁定ObjC bridge；check独立构建小型生成器，做语法、agent编译与离线DOM载荷检查，**不启动或附加任何目标应用**。离线DOM检查复用根项目`make setup-tests`准备的测试依赖。

依赖、生成器产物、Python bytecode、Gadget、profile、token、日志和截图全部位于仓库外：默认`${INJECTRA_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/injectra}/references`，可用`INJECTRA_REFERENCE_CACHE`覆盖。Make导出Python缓存位置；手动执行脚本也须设置仓库外`PYTHONPYCACHEPREFIX`。node_modules只是忽略的外部缓存链接。不要把旧Probe的active.json默认为本目录当前目标。

## 统计口径

`catalog.json`定位各case.json；各案例只声明应用入口和验证辅助文件，共享源码按物理文件全局计一次。`node references/scripts/stats.mjs --json`给出当前每文件行数、字节和SHA-256，不维护会过期的手写总行数。

- shared：共享参考源码，不再加到各软件专属数中；Parent已有Web runtime也不重复计入。
- application：软件入口、专属条件及命令；validation：诊断、受控面板与验证辅助。
- maintenance：归档的依赖准备/校验/统计工具，单列，不能算作某款软件的新注入能力。

按含注释/空行的物理源码行计数，排除Markdown、JSON/TOML元数据、锁文件、依赖和运行产物。此口径与原探针“501行含部分命令/配置”的描述不完全相同，不能直接相减作为功能增量。

原Probe提交/文件哈希见各origin.json及case.json；迁移只适配目录、依赖、agent和生成器位置，原独立仓库和历史run不改写。启动、目标权限、ABI及应用版本仍须在每次实机验证前核对。[产品边界](../docs/requirements.md)与[历史证据入口](../docs/validation.md)继续有效。
