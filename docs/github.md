# GitHub 仓库与协作入口

归属[T-070](TODO.md#t-070)。仓库保持private；以下文件管理项目展示和协作，不改变运行时注入能力或论文科研主张。

## 项目与研究入口

- [English README](../README.md) / [中文 README](../README.zh-CN.md)：用途、双路线、能力边界、命令、验证口径及研究导航。
- [软件引用](../CITATION.cff)：使用项目贡献者集体署名；无论文DOI或preferred-citation，论文仍准备中。复现另记录准确commit/tag。
- [贡献指南](../CONTRIBUTING.md)、[支持](../SUPPORT.md)、[安全说明](../SECURITY.md)、[行为规范](../CODE_OF_CONDUCT.md)。
- [PR模板](../.github/pull_request_template.md)、[缺陷表单](../.github/ISSUE_TEMPLATE/bug_report.yml)、[功能/研究提案](../.github/ISSUE_TEMPLATE/feature_request.yml)、[联系入口](../.github/ISSUE_TEMPLATE/config.yml)。

## 自动检查

| 文件 | 验证范围 |
| --- | --- |
| [Repository checks](../.github/workflows/ci.yml) | Ubuntu与macOS Apple Silicon的Rust/Web/架构/文档检查；Windows独立执行域/CDP合同；GitHub YAML与CFF元数据 |
| [Dependabot](../.github/dependabot.yml) | 每月提出GitHub Actions/Cargo依赖更新PR，不自动合并，不改变现有锁文件快照的历史 |

工作流使用只读contents权限和不可变Action SHA，无发布/部署密钥，不启动个人App。`macos-15`按[GitHub官方runner表](https://github.com/actions/runner-images)选用arm64，并在job中检查架构；Linux核心检查不构成Linux应用兼容证明。

`make doc-check`覆盖根目录所有Markdown及.github的Markdown。额外[GitHub检查器](../scripts/check-github.py)用[可选锁定依赖](../.github/requirements-validation.txt)解析工作流/表单并校验官方CFF 1.2 schema；schema来源为[固定提交与SHA256](../.github/cff-schema-source.json)，读取需要网络。本地可在专用环境运行：

```sh
uv run --python 3.13 --with-requirements .github/requirements-validation.txt scripts/check-github.py
```

这是轻量开发检查工具，不进入CLI运行依赖；uv环境和缓存均在仓库外。也可在已有隔离Python环境安装同一requirements后执行。

## 远端配置与边界

[Linguist分类](../.gitattributes)将论文、研究图稿、文档和参考归档标为documentation，不改变文件字节或证据哈希。About说明和Topics与项目定位一致，研究入口指向仓库内论文目录。不建立虚构网站、论文作者/DOI、公开Release或Pages；不改变private可见性、现有许可/tag或协作者权限，也不新增强制人工审批门禁。

论文翻译会话的未提交内容不属于本任务；本工作树按已提交稿源写研究导航，不将翻译中间态视为完成稿或纳入投稿就绪声明。论文统计口径差异仍由承接。
