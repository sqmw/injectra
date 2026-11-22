# Security policy / 安全说明

## Trust model

Injectra executes trusted local JS/HTML/CSS in an explicitly selected application page. It is not a sandbox for untrusted plugins. A payload may use the authority available to that page; removing owned nodes is not a general reversal of persistent or external effects.

Use loopback endpoints and authenticated, identity-checked sessions where supported. Keep session files and Bootstrap keys outside Git. An `Unknown` result must not be automatically replayed. Access requirements and cleanup limitations depend on the [backend](docs/frida-backends.md).

## Reporting

Report suspected vulnerabilities to the maintainer through an established private channel. If you need contact details, open a public issue asking for a private reporting channel and keep the technical details for the private exchange. Include the affected commit, backend and observed impact in the private report. Remove credentials, personal profiles and user documents from attachments.

## Maintenance scope

The current development branch is the primary investigation target. Tags preserve research/source snapshots; they are not a promise of long-term security maintenance for every old release or bundled third-party runtime. There is no general kernel/native-memory backend in the product.

## 中文

仅运行可信载荷，核对目标身份并限制本地调试入口。发现疑似漏洞时，请通过已有私下渠道联系维护者，在私下报告中提供受影响的提交、后端和实际影响。如果需要联系方式，可以在公开 Issue 中只询问私下报告渠道。发送附件前请移除凭据、个人配置及用户文档。
