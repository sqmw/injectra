# GitHub 仓库与协作入口

当前读者入口与论文引用见[T-081](TODO.md#t-081)。此前的稿件迁出与仓库配置分别见[T-080](TODO.md#t-080)和[T-070](TODO.md#t-070)。

## 项目入口

- [English README](../README.md) / [中文 README](../README.zh-CN.md)：用途、双路线、能力边界、命令及验证口径。
- [论文](https://arxiv.org/abs/2611.20713)由 Qin Sun 署名。[引用元数据](../CITATION.cff)将论文设为 GitHub 首选引用，软件条目沿用贡献者集体署名。复现实验时另记实际使用的提交或标签。
- [贡献指南](../CONTRIBUTING.md)、[支持](../SUPPORT.md)、[安全说明](../SECURITY.md)、[行为规范](../CODE_OF_CONDUCT.md)。
- [PR模板](../.github/pull_request_template.md)、[缺陷表单](../.github/ISSUE_TEMPLATE/bug_report.yml)、[功能/研究提案](../.github/ISSUE_TEMPLATE/feature_request.yml)、[联系入口](../.github/ISSUE_TEMPLATE/config.yml)。

## 自动检查

| 文件 | 验证范围 |
| --- | --- |
| [Repository checks](../.github/workflows/ci.yml) | Ubuntu与macOS Apple Silicon的Rust/Web/架构/文档检查；Windows独立执行域/CDP合同；GitHub YAML与CFF元数据（仅 `workflow_dispatch` 手动触发，不随 `push`/PR 自动运行） |
| [Dependabot](../.github/dependabot.yml) | 每月提出GitHub Actions/Cargo依赖更新PR，不自动合并 |

工作流使用只读contents权限和不可变Action SHA，无发布/部署密钥，不启动个人App；日常验证以本机 `make check` 与 Windows 实机脚本为主，已通过 [T-092](TODO.md#t-092) 移除 `push` 与 `pull_request` 自动触发以避免账户账单锁定产生重复失败通知。`macos-15`按[GitHub官方runner表](https://github.com/actions/runner-images)选用arm64，并在job中检查架构；Linux核心检查不构成Linux应用兼容证明。

`make doc-check`覆盖根目录所有Markdown及.github的Markdown。额外[GitHub检查器](../scripts/check-github.py)用[可选锁定依赖](../.github/requirements-validation.txt)解析工作流/表单并校验官方CFF 1.2 schema；schema来源为[固定提交与SHA256](../.github/cff-schema-source.json)，读取需要网络。本地可在专用环境运行：

```sh
uv run --python 3.13 --with-requirements .github/requirements-validation.txt scripts/check-github.py
```

这是轻量开发检查工具，不进入CLI运行依赖；uv环境和缓存均在仓库外。也可在已有隔离Python环境安装同一requirements后执行。

## 远端配置

[Linguist分类](../.gitattributes)将文档和参考归档标为documentation。About 简介与 Topics 描述软件用途，Website 链接到 [arXiv 摘要页](https://arxiv.org/abs/2611.20713)。领域标签为area:cli、area:backends、area:pre-runtime及evidence。

云端Actions曾因账户级启动限制而未创建测试job，见[T-070](TODO.md#t-070)。这不能写成CI通过，也不能视为软件测试失败。旧Actions运行与迁移审计记录保留，不等于仍跟踪旧工作流或稿件；本次未删除运行日志。

## 论文资料迁出与残留边界

论文原稿、图表、写作与投稿资料已迁往独立论文工作区；代码仓库不跟踪paper/paper-en，`.gitignore`保留对应防误提交规则。迁移见[T-076](TODO.md#t-076)。原工作树的私有恢复历史不得重新推入此库。论文阅读插件的功能验收记录属于软件证据，不作为稿件清理对象。

2026-11-16 的远端镜像复核覆盖当时的 2 个分支和 3 个标签，可达历史中未发现稿件路径或已知稿件 blob。当时从认证 API 仍可按部分旧 SHA 读取旧稿件。详见[该次核验](validation/github-paper-residuals.json)。

GitHub说明历史重写后旧提交仍可能通过SHA缓存访问，服务端缓存清理由GitHub处理，且Support不承诺清除非敏感数据：[官方说明](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository)。本次未联系Support或更换仓库。
