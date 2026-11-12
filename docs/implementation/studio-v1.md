# Studio local service contract

Part of [T-040](platform-v1.md); Parent owns overall delivery and task status.

The service keeps existing WebBackend objects on dedicated OS threads, rather than adding Send or async requirements to the core/Web domain. CDP, Electron Gadget and WKWebView targets receive a per-document nonce; execution checks that nonce in the same page evaluation as the payload. Opaque Studio IDs include connection identity and document generation. Bootstrap uses its own native target generation and revision receipts. A saved source revision is never a global “applied” flag.

Browser API authorization uses a private persisted random credential delivered only in the launch URL fragment. Bearer authorization is required for API calls; writes require the exact loopback Origin. WebSocket events authenticate in their first frame under a five-second deadline and also require the exact Origin. Bootstrap routes use their separate challenge/role protocol and never accept browser credentials.

The first service address is allocated on loopback and persisted under the cache's studio directory keyed by canonical workspace root. Restart reuses the address; an occupied address or existing service lock is an explicit conflict. Credential metadata is private and symlinks are rejected. Source workspace files never contain controller credentials.

[Backend actor](../../crates/studio/src/actor.rs), [browser authentication](../../crates/studio/src/security.rs), [private endpoint metadata](../../crates/studio/src/endpoint.rs), [API errors](../../crates/studio/src/error.rs). Tests cover a non-Send backend's thread affinity, stale document refusal, Unknown propagation, bearer/origin separation, persisted address conflicts and metadata symlinks. Protocol tests and fixtures do not certify arbitrary real apps.
