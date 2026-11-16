# GitHub 仓库与协作入口

当前展示与复核归属[T-080](TODO.md#t-080)；早期配置记录见[T-070](TODO.md#t-070)。可见性由用户自行控制。

## 项目入口

- [English README](../README.md) / [中文 README](../README.zh-CN.md)：用途、双路线、能力边界、命令及验证口径。
- [软件引用](../CITATION.cff)：软件贡献者集体署名；复现时记录准确commit/tag。
- [贡献指南](../CONTRIBUTING.md)、[支持](../SUPPORT.md)、[安全说明](../SECURITY.md)、[行为规范](../CODE_OF_CONDUCT.md)。
- [PR模板](../.github/pull_request_template.md)、[缺陷表单](../.github/ISSUE_TEMPLATE/bug_report.yml)、[功能/研究提案](../.github/ISSUE_TEMPLATE/feature_request.yml)、[联系入口](../.github/ISSUE_TEMPLATE/config.yml)。

## 自动检查

| 文件 | 验证范围 |
| --- | --- |
| [Repository checks](../.github/workflows/ci.yml) | Ubuntu与macOS Apple Silicon的Rust/Web/架构/文档检查；Windows独立执行域/CDP合同；GitHub YAML与CFF元数据 |
| [Dependabot](../.github/dependabot.yml) | 每月提出GitHub Actions/Cargo依赖更新PR，不自动合并 |

工作流使用只读contents权限和不可变Action SHA，无发布/部署密钥，不启动个人App。`macos-15`按[GitHub官方runner表](https://github.com/actions/runner-images)选用arm64，并在job中检查架构；Linux核心检查不构成Linux应用兼容证明。

`make doc-check`覆盖根目录所有Markdown及.github的Markdown。额外[GitHub检查器](../scripts/check-github.py)用[可选锁定依赖](../.github/requirements-validation.txt)解析工作流/表单并校验官方CFF 1.2 schema；schema来源为[固定提交与SHA256](../.github/cff-schema-source.json)，读取需要网络。本地可在专用环境运行：

```sh
uv run --python 3.13 --with-requirements .github/requirements-validation.txt scripts/check-github.py
```

这是轻量开发检查工具，不进入CLI运行依赖；uv环境和缓存均在仓库外。也可在已有隔离Python环境安装同一requirements后执行。

## 远端配置

[Linguist分类](../.gitattributes)将文档和参考归档标为documentation，不改变文件字节或证据哈希。About说明和Topics描述代码项目；Website留空，不再指向已迁出的目录。领域标签为area:cli、area:backends、area:pre-runtime及evidence，未关联Issue的area:paper标签已移除。未修改private可见性、许可、tag、协作权限、计费或强制审批。

云端Actions曾因账户级启动限制而未创建测试job，见[T-070](TODO.md#t-070)。这不能写成CI通过，也不能视为软件测试失败。旧Actions运行与迁移审计记录保留，不等于仍跟踪旧工作流或稿件；本次未删除运行日志。

## 论文资料迁出与残留边界

论文原稿、图表、写作与投稿资料已迁往独立论文工作区；代码仓库不跟踪paper/paper-en，`.gitignore`保留对应防误提交规则。迁移见[T-076](TODO.md#t-076)。原工作树的私有恢复历史不得重新推入此库。论文阅读插件的功能验收记录属于软件证据，不作为稿件清理对象。

2026-11-16远端全新镜像复核覆盖2个分支和3个标签：可达历史中无稿件路径，全部对象中无已知稿件blob匹配。但已知旧SHA仍可在认证API中读取旧稿件，不能称GitHub服务器已彻底删除；同一仓库继续保持private，公开由用户决定。详见[本次核验](validation/github-paper-residuals.json)。

GitHub说明历史重写后旧提交仍可能通过SHA缓存访问，服务端缓存清理由GitHub处理，且Support不承诺清除非敏感数据：[官方说明](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository)。本次未联系Support或更换仓库。
