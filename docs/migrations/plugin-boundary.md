# 插件集中迁移与职责隔离

2026-11-05，归属[T-016](../TODO.md#t-016)，执行边界见[D-007](../DECISIONS.md#d-007)。

## 当前归属

全部已识别的活跃Injectra插件与可复用Bundle集中在独立web-ui-plugins库，权威目录见catalog（独立 web-ui-plugins 仓库内 catalog.json）。原有豆包快捷主题、抖音图片主题保留，新增迁入论文阅读、通用缩放、抖音验证面板、Cursor居中面板、WorkBuddy居中面板，共7项。

Injectra不再保存examples/accent或参考案例payload；注入runtime、CDP/V8桥接和验证工具仍属于注入器。Rust、Electron及离线参考检查只生成临时测试输入，不依赖插件库，不对外提供测试插件命令。架构检查拒绝源码树中的可交付Bundle JSON。

## 迁移与恢复

- 独立doubao-paper-focus项目完整迁入插件库，原目录删除；原HEAD为20e6fe0，完整Git元数据保存在仓库外插件缓存的migrations/20261105-NpxIsQ/doubao-paper-focus.git，保留所有引用/对象。
- 原examples/accent和references内抖音payload各4文件迁入相应插件bundle目录；Cursor/WorkBuddy原缓存bundle分别4/3文件迁入后删除旧bundle目录。
- 共53文件先复制逐项SHA核对，再删除旧位置；迁入收据（独立 web-ui-plugins 仓库内 docs/migrations/20261105-inventory.json）为来源与原文件hash的唯一清单。论文src与tests、四组静态Bundle共29文件在构建后再次比对一致；构建环境/导航整合修改另由提交记录。
- 插件库交付提交为67908ba；本库源文件删除、测试解耦与边界由本文件所在提交保存。需要恢复旧项目时使用原Git备份，或从插件库提交取得文件；不要恢复长期双份活跃源码。
- 既有profile、账号、日志和历史run结果未删除，历史生成的验证夹具不作为活跃插件源码；没有重启或重新注入用户应用。原始origin.json仅承担迁移前来源证据，当前case.json与统计不再把插件计入接入源码。

## 验证与review

| 检查 | 已实测结果 |
| --- | --- |
| 插件库make check | 7项构建/静态Bundle校验，22项自动测试通过；无嵌套Git，53迁入文件全部进入统一库索引 |
| 核心make check | 格式、clippy、16项Rust/5项Web、文档/依赖及插件归属检查通过 |
| 独立真实Electron | runs/electron-gyEk6c通过10项核心检查；临时CSS/HTML/JS验证样式、点击、更新与清理，不读取插件库 |
| 参考实现make check | 25份源码静态检查、3个agent编译及临时DOM输入检查通过；不启动或附加用户应用 |
| 文档与任务 | 核心/插件库本地链接及各自tracking检查通过 |

Review核对了插件库目标冲突、字节保全、旧目录消失、Git历史可恢复、业务源码无改动、核心回归无外部插件依赖，以及引用/统计与手工验证入口。首次核心doc-check因旧夹具导航失败，已更新到临时测试生成器，不放宽检查。

本轮仅证明迁移完整及自动回归；没有重新验证7个插件在真实应用中的全部效果，不把旧实测改写成新结果。插件使用者以后显式提供BUNDLE_MANIFEST；仅参考apply/verify增加该要求，产品CLI的Bundle加载接口不变。
