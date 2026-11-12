# Studio local service contract

> 状态校准（2026-11-12）：此页记录T-040@v1的Studio服务与工作流，不是当前交付要求。用户已将目标收敛到[通用注入核心与dev模式](../architecture/dev-mode.md)。 工作台与完整工作区不再推进；v1整体验收未完成；用户后续批准删减，平台模块已从当前产品删除，历史源码由Git/恢复快照保留。 本页技术事实保留，不据此继续旧范围开发。


Part of [T-040](platform-v1.md); Parent owns overall delivery and task status.

The service keeps existing WebBackend objects on dedicated OS threads, rather than adding Send or async requirements to the core/Web domain. CDP, Electron Gadget and WKWebView targets receive a per-document nonce; execution checks that nonce in the same page evaluation as the payload. Opaque Studio IDs include connection identity and document generation. Bootstrap uses its own native target generation and revision receipts. A saved source revision is never a global “applied” flag.

Browser API authorization uses a private persisted random credential delivered only in the launch URL fragment. Bearer authorization is required for API calls; writes require the exact loopback Origin. WebSocket events authenticate in their first frame under a five-second deadline and also require the exact Origin. Bootstrap routes use their separate challenge/role protocol and never accept browser credentials.

The first service address is allocated on loopback and persisted under the cache's studio directory keyed by canonical workspace root. Restart reuses the address; an occupied address or existing service lock is an explicit conflict. Credential metadata is private and symlinks are rejected. Source workspace files never contain controller credentials.

Backend actor（历史源码：`../../crates/studio/src/actor.rs`，main提交`07d55a3`；已从当前产品删除）, browser authentication（历史源码：`../../crates/studio/src/security.rs`，main提交`07d55a3`；已从当前产品删除）, private endpoint metadata（历史源码：`../../crates/studio/src/endpoint.rs`，main提交`07d55a3`；已从当前产品删除）, API errors（历史源码：`../../crates/studio/src/error.rs`，main提交`07d55a3`；已从当前产品删除）. Tests cover a non-Send backend's thread affinity, stale document refusal, Unknown propagation, bearer/origin separation, persisted address conflicts and metadata symlinks. Protocol tests and fixtures do not certify arbitrary real apps.

## Commands and browser API

`injectra create --workspace DIR --id ID` creates sources. `injectra import --workspace DIR --manifest FILE` copies a legacy manifest into an empty workspace. `injectra studio --workspace DIR [--cache DIR] [--assets DIST]` serves the workbench. Default assets are `$INJECTRA_CACHE_DIR/studio-ui/dist`; `INJECTRA_STUDIO_ASSETS` overrides them. No Node runtime is required to serve precompiled assets. Missing default assets leave the authenticated API available; a missing explicitly configured assets directory is an error.

Startup writes one compact `studio-ready` JSON line containing `origin`, `port`, `url`, `workspace`, and `ready_file`. The URL includes a private `#token=` fragment and must not be copied into public logs. CLI clients use the private ready file, bypass HTTP proxies and redirects, and never retry requests without a confirmed result. `prepare`, `dev`, and `export` are clients of the running Studio; they appear in browser capabilities only after their concrete preparation implementation is installed.

| Endpoint | Request / response |
| --- | --- |
| GET `/api/state` | Workspace, files, targets, latest 200 runs, startup and capabilities |
| GET `/api/files?path=user:page.js` | Path, content, origin, stage and current hash |
| PUT `/api/files` | `{path,content,expected_hash}`; hash mismatch is 409; null creates only absent files |
| POST `/api/import` | `{manifest}`; allowed only for an empty workspace |
| POST `/api/connect` | `{backend,port?,session?}`; static `cdp`, `electron-gadget`, `webkit`, `bootstrap` |
| POST `/api/run` | `{target,generation?,revision,acknowledge_unknown?}`; 202 `{request_id}` |
| POST `/api/reload` | `{target,generation?}`; requires target reload capability |
| POST `/api/prepare` | `{app,data_mode}` |
| POST `/api/dev`, `/api/restart` | Current owned prepared artifact and source snapshot |
| POST `/api/export` | `{destination,target?,generation?,origin?,path_suffix?}` |
| WS `/api/events` | First frame `{type:"authenticate",token}`; ack `{type:"authenticated"}` then state/run/log events |

Source paths use explicit `user:`, `application:`, or `injectra:` namespaces. Application and Injectra sources are registered readonly projections; bridge configuration and credentials are never exposed as readable files. New user files, including `startup.cjs`, use the same create-only PUT contract. External edits are detected using content hashes; failed saves never overwrite the newer source.

Run records and source mappings persist as private per-request JSON files under the workspace's Studio cache. Startup converts unfinished records to `Unknown`, invalidates runtime targets and never replays them. Current target execution is additionally gated after Unknown until an explicit matching request acknowledgement. A session admits at most 10,000 recorded requests, reports capacity exhaustion explicitly, and never removes historical evidence automatically. UI state shows the newest 200; complete records remain in the cache. Saved revision and per-target applied revision remain separate.

HTTP routes（历史源码：`../../crates/studio/src/http.rs`，main提交`07d55a3`；已从当前产品删除）, service state（历史源码：`../../crates/studio/src/state.rs`，main提交`07d55a3`；已从当前产品删除）, run persistence（历史源码：`../../crates/studio/src/history.rs`，main提交`07d55a3`；已从当前产品删除）, 历史CLI wiring（`crates/cli/src/studio.rs`，提交`07d55a3`）, service tests（历史源码：`../../crates/studio/tests/service.rs`，main提交`07d55a3`；已从当前产品删除）. Tests exercise HTTP 401/403, external-edit conflict, create-only startup sources, readonly namespaces, unavailable capabilities, WS first-frame authentication/deadline, and recovery to Unknown. Real CDP application behavior requires the separate fixture acceptance run.
