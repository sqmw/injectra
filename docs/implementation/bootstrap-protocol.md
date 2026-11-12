# Bootstrap bridge contract

> 状态校准（2026-11-12）：此页记录T-040@v1的Bootstrap通信实现，不是当前交付要求。用户已将目标收敛到[通用注入核心与dev模式](../architecture/dev-mode.md)。 其中通道/身份/逐目标确认可作为复用候选；当前与Studio的装配事实不等于v2已完成解耦或通用实机验收。 本页技术事实保留，不据此继续旧范围开发。


Part of [T-040 platform implementation](platform-v1.md). This module is not an application-brand adapter and does not certify arbitrary apps.

`injectra-bootstrap` exposes `Broker`, `BootstrapBackend`, and `router(Arc<Broker>) -> axum::Router`. Register an artifact ID and its 256-bit hex key before accepting a connection. The Studio keeps these keys server-side; they must not appear in browser state or logs.

The builtin Node agent uses loopback HTTP: challenge/register with fresh client/server nonces and role-separated HMAC-SHA256, then bearer-authenticated poll/report/disconnect. Browser Origin headers are rejected on bridge routes. The HTTP service must only bind loopback; UI APIs require their own authentication and origin validation.

Process instance IDs and page generations are fresh runtime identities. A new authenticated connection invalidates any prior connection for the same artifact/process instance or PID. Reusing the artifact key does not preserve targets. Main-frame targets are reported only when a document is ready; navigation and render-process replacement invalidate old targets. No page payload is automatically applied in development mode.

Commands bind request, revision, page and generation. Each command is delivered at most once. Each page has independent applied revision, updated only after its successful matching receipt. Queued timeout is NotExecuted; delivered timeout/disconnect is Unknown. Late receipts are retained as neither new successes nor reasons to replay. Receipt retries do not reexecute code. Logs are limited to injected source URLs; this is not whole-application log collection.

The synchronous Broker API is for worker threads, not an async executor's event loop. `submit`/`reload` have explicit deadlines. Existing WebSession remains page-only; native contracts are unaffected. The agent's export mode uses fixed embedded startup/page inputs and never contacts the controller.

Tests: `cargo test -p injectra-bootstrap`; real owned Electron validation is performed with the preparation module. Contract tests alone do not prove app compatibility.

Protocol review: public target IDs use independent public connection IDs, never session bearer tokens. The agent installs a per-document nonce and checks it atomically in the same evaluation as the generated expression; workspace-generated expression programs retain their original line numbers (the guard adds no preceding newline). Direct statement source must first be compiled by the workspace/program layer, as with the existing Gadget expression convention. Own logs carry request/revision/original-document metadata and a separate user source URL; the broker validates these against dispatched request history, retaining old-document attribution for late logs. Dispatched Unknown or Executed failure clears prior applied revision; NotExecuted preserves it.

Run the document-guard/source-line regression with `node --test crates/bootstrap/tests/evaluation.test.cjs` in addition to Rust tests.
