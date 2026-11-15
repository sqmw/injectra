# Security policy / 安全说明

## Trust model

Injectra executes trusted local JS/HTML/CSS in an explicitly selected application page. It is not a sandbox for untrusted plugins. A payload may use the authority available to that page; removing owned nodes is not a general reversal of persistent or external effects.

Use loopback endpoints and authenticated, identity-checked sessions where supported. Keep session files and Bootstrap keys outside Git. An `Unknown` result must not be automatically replayed. Access requirements and cleanup limitations depend on the [backend](docs/frida-backends.md).

## Reporting

This repository is private. Authorized collaborators may open a **redacted private-repository issue** describing the affected commit/backend, preconditions, and a minimal reproduction with owned data. Do not attach live credentials, personal profiles, user documents, or working access to someone else's application. For material that should not be visible to all repository collaborators, contact the maintainer through an already established private channel first.

Do not assume a public vulnerability-reporting portal, public advisory, or bug-bounty program exists. Coordinate disclosure with the maintainer before sharing a report outside the authorized group.

## Maintenance scope

The current development branch is the primary investigation target. Tags preserve research/source snapshots; they are not a promise of long-term security maintenance for every old release or bundled third-party runtime. There is no general kernel/native-memory backend in the product.

## 中文

仅运行可信载荷，核对目标身份并限制本地调试入口；源码仓库不保存凭据或用户 profile。发现问题时提供脱敏复现及实际影响，敏感材料先通过已有私下渠道联系维护者。私有仓库 issue 对获授权协作者可见，不等同于只向某一位维护者保密。旧 tag 是可追溯快照，不代表全部版本都获长期维护。
