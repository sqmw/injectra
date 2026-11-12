# External increment workspace contract

> 状态校准（2026-11-12）：此页记录T-040@v1的完整工作区/快照模型，不是当前交付要求。用户已将目标收敛到[通用注入核心与dev模式](../architecture/dev-mode.md)。 工作台与完整工作区不再推进；现有实现保留，尚未完成v1整体验收。 本页技术事实保留，不据此继续旧范围开发。


Part of [T-040](../TODO.md#t-040); Parent owns Task/Execution status. This library supplies source/snapshot and execution-state contracts, not target preparation or a live backend.

## Public API

`injectra-workspace` depends only on the existing core/Web domains plus filesystem/hash/serialization utilities.

- `Workspace::create(root, bundle_id)` creates a new or empty external directory with manifest v1 and editable page.js/page.css/page.html; `import_manifest(root, manifest)` copies bounded user assets into canonical page.js/page.css/page.html in a new/empty root and records their original source references (including the input manifest). Neither mutates imported source files.
- `Workspace::open(root)`, `id()`, `root()`, `files()`, `read(relative_path)`, `save(relative_path, content)` operate only on that explicit root. Editor writes to manifest.json and injectra.workspace.json are rejected; optional startup.cjs and relative JS/CJS/MJS/CSS/HTML/JSON helper files can be added.
- `save_if_match(path, content, expected_hash)` returns `SaveOutcome::Saved { file }` or `Conflict { current_hash }`; None means create-only. Writes within the Workspace object are serialized. This is an optimistic editor check, not an atomic compare-and-swap against arbitrary external editors that ignore the workspace lock.
- `snapshot(cache_root)` produces `Snapshot { workspace_id, revision, root, manifest, startup, files }`. The startup path, when present, is absolute inside root and is separate from the page manifest. `Snapshot::load(revision_directory)`, `verify()`, `bundle()`, `startup_source()`, `page_program()` and `mapped_program()` are public.

`SourceOrigin` is application/injectra/user; `SourceStage` is page/startup/shared/metadata. Workspace files are user increments; application baselines and tool-generated changes are references supplied by the preparation layer, not editable files imported into this root. `SourceReference` preserves provenance without granting path access. Types serialize with snake_case enum values.

## Snapshot and path guarantees

Source files are UTF-8 text, each at most 1 MiB, total at most 8 MiB, with at most 128 entries and bounded nesting. Hidden directories (including .git) are not imported as payload. Relative paths reject traversal, control characters, backslashes and colon prefixes; generated source URLs contain only the workspace/revision identity; user filenames remain separate mapping data.

Unix directory-handle operations open each path component without following symlinks. Symlinks/special files are rejected, and atomic replacement never writes through an existing hardlink. The filesystem module explicitly reports unsupported on non-Unix systems; this is not Windows certification.

Snapshots copy bytes into separate inodes, hash source contents and provenance, publish a complete revision without replacing an existing one, then mark payload/metadata read-only. Existing revisions are verified rather than overwritten. Cache ancestors are resolved before creation; symlinked internal cache directories or revision directories are rejected. The cache is a service-owned directory, not a path selected by an untrusted browser client. Bundle generation reads and validates owned bytes before constructing the existing Web Bundle and calling `injectra_web::apply`; there is no copied runtime implementation.

Read-only flags are accidental-write protection, not protection from the OS account owner. Every use validates hashes and file membership, so owner tampering fails rather than silently changing a revision. Source capture rechecks files and membership; arbitrary uncooperative writers can still require the caller to retry capture explicitly. No automatic retry of target execution follows.

`mapped_program()` supplies an injectra:// workspace/revision/generated-unit URL with explicit 1-based generated line ranges: wrapper/runtime ranges are Injectra-owned and only the user JS range maps to the user file. `generated_start_line`/`generated_end_line` bound every mapping; `source_start_line` is the corresponding logical source line. CSS/HTML and startup sources remain in the snapshot source list. This is a mapping for this generator's wrapper, not automatic recovery of an original application's source or third-party transpiler maps.

## Execution ledger

`ExecutionTarget` contains backend_instance, target and generation. `ExecutionLedger::begin` creates a request record and refuses another in-flight operation on that target or an unacknowledged Unknown. Call `mark_submitted(request_id, target)` immediately before backend dispatch to make in-flight certainty Unknown. `complete(request_id, target, revision, outcome)` only accepts the corresponding pending reply. Successful completion records that target's revision; executed errors/Unknown clear its applied state.

`invalidate(target)` retires that generation and marks pending work Unknown. A new generation has a separate identity. `acknowledge_unknown` only opens a gate after an explicit caller decision: it never queues or replays source. Records preserve prior attempts. Studio is responsible for persisting records if history must survive service restart and for bounding/paging long sessions.

## Tests and code map

Filesystem（历史源码：`../../crates/workspace/src/filesystem.rs`，main提交`07d55a3`；已从当前产品删除）, workspace/import（历史源码：`../../crates/workspace/src/workspace.rs`，main提交`07d55a3`；已从当前产品删除）, snapshots/mapping（历史源码：`../../crates/workspace/src/snapshot.rs`，main提交`07d55a3`；已从当前产品删除）, source types（历史源码：`../../crates/workspace/src/model.rs`，main提交`07d55a3`；已从当前产品删除）, execution ledger（历史源码：`../../crates/workspace/src/execution.rs`，main提交`07d55a3`；已从当前产品删除）, contract tests（历史源码：`../../crates/workspace/tests/contracts.rs`，main提交`07d55a3`；已从当前产品删除）.

`cargo test -p injectra-workspace` covers source preservation, content revisions, startup dependencies, conflicts, traversal/size/symlink rejection, hardlink replacement, snapshot tampering and per-target/Unknown behavior. Twelve focused contract tests and clippy with warnings denied pass. These are library guarantees, not real application compatibility evidence. Full doc-check in the isolated worktree is blocked by an existing cross-repository plugin link whose relative checkout layout is absent here; Parent runs it from the main checkout.
