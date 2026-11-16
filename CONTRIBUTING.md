# Contributing / 贡献指南

Injectra is a private research and development repository. Work against the current code and [evidence boundaries](docs/compatibility/current-matrix.md), and keep the [English](README.md) and [Chinese](README.zh-CN.md) introductions consistent.

## Changes and validation

1. Describe the problem, intended behavior, execution domain, and affected backend before changing code.
2. Keep the shared Web runtime in one place. Separate protocol, host, resource-format, and platform concerns; payload-specific selectors/themes belong in the external plugin project.
3. Use the versions in [mise.toml](mise.toml). Run `make setup-tests` and `make check`. Run the controlled Electron or preparation fixtures when those paths change.
4. Keep dependencies, generated PDFs, profiles, logs, credentials, and runtime caches outside the repository. Do not commit a real session file or an application plugin copy.
5. In the PR, record the validation commands, results, relevant software versions, and remaining limitations. Do not equate a fixture or open port with real-application compatibility.

## Validation evidence

New capability and compatibility claims need traceable evidence. Screenshots and summaries must preserve the recorded results and limitations. Keep historical failures and batches intact; do not invent missing aggregate results.

Real-application experiments require explicit target and process authorization. Default checks must not manipulate personal apps. Report uncertain execution without automatic retries. See [SECURITY.md](SECURITY.md) and [SUPPORT.md](SUPPORT.md).

## 中文要点

先说明问题与变更边界，再按模块职责修改；不要在通用核心加入品牌分支，也不要把主题/控件适配复制进注入器。代码变更配套验证和文档，能力与兼容性结论配套版本、批次和来源。真实应用测试单独授权；不提交会话密钥、运行数据或用户业务内容，不编造缺失结果。

Collaboration follows the [Code of Conduct](CODE_OF_CONDUCT.md). Open a focused issue or pull request; automated checks provide evidence, not approval of every application or research claim.

GitHub metadata checks use the optional pinned tools in [.github/requirements-validation.txt](.github/requirements-validation.txt). In an isolated tool environment, run `python scripts/check-github.py`; this fetches an immutable, checksum-verified official CFF schema and checks workflow/form structure. It is separate from the runtime CLI and default `make check`.
