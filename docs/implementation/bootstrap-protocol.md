# Bootstrap bridge contract

Part of [T-040 platform implementation](platform-v1.md). This module is not an application-brand adapter and does not certify arbitrary apps.

`injectra-bootstrap` exposes `Broker`, `BootstrapBackend`, and `router(Arc<Broker>) -> axum::Router`. Register an artifact ID and its 256-bit hex key before accepting a connection. The Studio keeps these keys server-side; they must not appear in browser state or logs.

The builtin Node agent uses loopback HTTP: challenge/register with fresh client/server nonces and role-separated HMAC-SHA256, then bearer-authenticated poll/report/disconnect. Browser Origin headers are rejected on bridge routes. The HTTP service must only bind loopback; UI APIs require their own authentication and origin validation.

Process instance IDs and page generations are fresh runtime identities. A new authenticated connection invalidates any prior connection for the same artifact/process instance or PID. Reusing the artifact key does not preserve targets. Main-frame targets are reported only when a document is ready; navigation and render-process replacement invalidate old targets. No page payload is automatically applied in development mode.

Commands bind request, revision, page and generation. Each command is delivered at most once. Each page has independent applied revision, updated only after its successful matching receipt. Queued timeout is NotExecuted; delivered timeout/disconnect is Unknown. Late receipts are retained as neither new successes nor reasons to replay. Receipt retries do not reexecute code. Logs are limited to injected source URLs; this is not whole-application log collection.

The synchronous Broker API is for worker threads, not an async executor's event loop. `submit`/`reload` have explicit deadlines. Existing WebSession remains page-only; native contracts are unaffected. The agent's export mode uses fixed embedded startup/page inputs and never contacts the controller.

Tests: `cargo test -p injectra-bootstrap`; real owned Electron validation is performed with the preparation module. Contract tests alone do not prove app compatibility.
