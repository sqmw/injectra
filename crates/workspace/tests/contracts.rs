use injectra_workspace::*;
use std::fs;
use tempfile::TempDir;
fn create() -> (TempDir, Workspace) {
    let temp = TempDir::new().unwrap();
    let w = Workspace::create(&temp.path().join("user"), "test.increment").unwrap();
    (temp, w)
}
#[test]
fn revisions_capture_content_and_preserve_startup_dependencies() {
    let (temp, w) = create();
    w.save("startup.cjs", "require('./lib/helper.cjs');")
        .unwrap();
    w.save("lib/helper.cjs", "module.exports=1;").unwrap();
    w.save("page.js", "console.log('first');").unwrap();
    let a = w.snapshot(&temp.path().join("cache")).unwrap();
    let first = a.page_program().unwrap();
    assert!(first.contains("console.log('first')"));
    assert!(first.contains("//# sourceURL=injectra://"));
    assert_eq!(a.startup, Some(a.root.join("startup.cjs")));
    w.save("page.js", "console.log('second');").unwrap();
    let b = w.snapshot(&temp.path().join("cache")).unwrap();
    assert_ne!(a.revision, b.revision);
    assert_eq!(a.page_program().unwrap(), first);
    assert!(b.page_program().unwrap().contains("console.log('second')"));
    assert_eq!(
        b.revision,
        w.snapshot(&temp.path().join("cache")).unwrap().revision
    );
    let map = a.mapped_program().unwrap();
    assert_eq!(
        map.source
            .lines()
            .nth(
                map.mappings
                    .iter()
                    .find(|m| m.origin == SourceOrigin::User)
                    .unwrap()
                    .generated_start_line
                    - 1
            )
            .unwrap(),
        "console.log('first');"
    );
}
#[test]
fn imports_copy_sources_and_readonly_metadata_is_not_user_editable() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("source");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("main.js"), "original()").unwrap();
    fs::write(
        src.join("bundle.json"),
        r#"{"version":1,"id":"imported","js":"main.js"}"#,
    )
    .unwrap();
    let w =
        Workspace::import_manifest(&temp.path().join("work"), &src.join("bundle.json")).unwrap();
    w.save("page.js", "changed()").unwrap();
    assert_eq!(
        fs::read_to_string(src.join("main.js")).unwrap(),
        "original()"
    );
    assert!(
        w.files()
            .unwrap()
            .iter()
            .find(|f| f.path == "page.js")
            .unwrap()
            .imported_from
            .is_some()
    );
    assert!(w.save("manifest.json", "{}").is_err());
    assert!(w.save("injectra.workspace.json", "{}").is_err());
    assert!(Workspace::import_manifest(&src, &src.join("bundle.json")).is_err());
}
#[test]
fn editor_optimistic_conflicts_are_explicit() {
    let (_temp, w) = create();
    let old = w
        .files()
        .unwrap()
        .into_iter()
        .find(|f| f.path == "page.js")
        .unwrap();
    w.save("page.js", "external change").unwrap();
    assert!(matches!(
        w.save_if_match("page.js", "stale", Some(&old.hash))
            .unwrap(),
        SaveOutcome::Conflict { .. }
    ));
    assert_eq!(w.read("page.js").unwrap(), "external change");
    assert!(matches!(
        w.save_if_match("startup.cjs", "// init", None).unwrap(),
        SaveOutcome::Saved { .. }
    ));
    assert!(matches!(
        w.save_if_match("startup.cjs", "// stale", None).unwrap(),
        SaveOutcome::Conflict { .. }
    ));
}
#[test]
fn paths_and_size_limits_fail_before_source_mutation() {
    let (temp, w) = create();
    for bad in [
        "../escape.js",
        "/absolute.js",
        "a/../escape.js",
        "a\\escape.js",
        "a\nfile.js",
        ".private.js",
        "x:bad.js",
    ] {
        assert!(w.save(bad, "x").is_err(), "{bad}");
    }
    assert!(w.save("large.js", &"x".repeat(MAX_FILE_BYTES + 1)).is_err());
    assert!(!w.root().join("large.js").exists());
    assert!(w.snapshot(&w.root().join("cache")).is_err());
    assert!(!w.root().join("cache").exists());
    assert!(!temp.path().join("escape.js").exists());
}
#[cfg(unix)]
#[test]
fn symlinks_are_rejected_and_hardlinks_are_not_written_through() {
    use std::os::unix::fs::symlink;
    let (temp, w) = create();
    let external = temp.path().join("outside.js");
    fs::write(&external, "original").unwrap();
    symlink(&external, w.root().join("link.js")).unwrap();
    assert!(w.read("link.js").is_err());
    assert!(w.save("link.js", "bad").is_err());
    assert!(w.snapshot(&temp.path().join("cache")).is_err());
    fs::remove_file(w.root().join("link.js")).unwrap();
    fs::hard_link(&external, w.root().join("linked.js")).unwrap();
    w.save("linked.js", "owned new inode").unwrap();
    assert_eq!(fs::read_to_string(external).unwrap(), "original");
    symlink(temp.path(), w.root().join("outside")).unwrap();
    assert!(w.save("outside/escape.js", "bad").is_err());
    assert!(!temp.path().join("escape.js").exists());
}
#[cfg(unix)]
#[test]
fn readonly_snapshots_detect_owner_tampering_and_never_alias_workspace_inodes() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let (temp, w) = create();
    let snap = w.snapshot(&temp.path().join("cache")).unwrap();
    let p = snap.root.join("page.js");
    assert_ne!(
        fs::metadata(&p).unwrap().ino(),
        fs::metadata(w.root().join("page.js")).unwrap().ino()
    );
    fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&p, "tampered").unwrap();
    assert!(snap.verify().is_err());
    assert!(snap.page_program().is_err());
    assert!(w.snapshot(&temp.path().join("cache")).is_err());
}
fn target(generation: &str) -> ExecutionTarget {
    ExecutionTarget {
        backend_instance: "backend-one".into(),
        target: "page-one".into(),
        generation: generation.into(),
    }
}
#[test]
fn ledger_confirms_each_target_and_rejects_late_or_cross_generation_replies() {
    let mut ledger = ExecutionLedger::default();
    let a = target("g1");
    let mut b = a.clone();
    b.target = "page-two".into();
    let ra = ledger.begin(a.clone(), "v1").unwrap();
    let rb = ledger.begin(b.clone(), "v1").unwrap();
    assert!(ledger.begin(a.clone(), "v2").is_err());
    let ok = ExecutionOutcome {
        certainty: ExecutionCertainty::Executed,
        result: Some(serde_json::json!(1)),
        error: None,
    };
    assert!(
        ledger
            .complete(&ra.request_id, &target("g2"), "v1", ok.clone())
            .is_err()
    );
    assert!(
        ledger
            .complete(&ra.request_id, &a, "wrong", ok.clone())
            .is_err()
    );
    ledger
        .complete(&ra.request_id, &a, "v1", ok.clone())
        .unwrap();
    assert_eq!(ledger.applied_revision(&a), Some("v1"));
    assert_eq!(ledger.applied_revision(&b), None);
    ledger.invalidate(&b).unwrap();
    assert_eq!(
        ledger.records().last().unwrap().status,
        ExecutionStatus::Unknown
    );
    assert!(ledger.complete(&rb.request_id, &b, "v1", ok).is_err());
    assert!(ledger.begin(b, "v2").is_err());
    assert!(ledger.begin(target("g2"), "v2").is_ok());
}
#[test]
fn unknown_is_not_replayed_or_reported_as_applied() {
    let mut ledger = ExecutionLedger::default();
    let t = target("g");
    let r = ledger.begin(t.clone(), "v1").unwrap();
    ledger
        .complete(
            &r.request_id,
            &t,
            "v1",
            ExecutionOutcome {
                certainty: ExecutionCertainty::Unknown,
                result: None,
                error: Some("disconnected after send".into()),
            },
        )
        .unwrap();
    assert_eq!(ledger.applied_revision(&t), None);
    assert!(ledger.begin(t.clone(), "v1").is_err());
    assert_eq!(ledger.records().len(), 1);
    ledger.acknowledge_unknown(&t, &r.request_id).unwrap();
    assert_eq!(ledger.records().len(), 1);
    assert!(ledger.begin(t, "v2").is_ok());
}

#[test]
fn imported_page_named_startup_is_not_executed_as_main_increment() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("import");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("startup.cjs"), "pageOnly()").unwrap();
    fs::write(
        source.join("manifest.json"),
        r#"{"version":1,"id":"legacy","js":"./startup.cjs"}"#,
    )
    .unwrap();
    let w = Workspace::import_manifest(&temp.path().join("work"), &source.join("manifest.json"))
        .unwrap();
    let snap = w.snapshot(&temp.path().join("cache")).unwrap();
    assert_eq!(snap.bundle().unwrap().js, "pageOnly()");
    assert!(snap.startup.is_none());
    assert_eq!(
        w.files()
            .unwrap()
            .iter()
            .find(|f| f.path == "page.js")
            .unwrap()
            .stage,
        SourceStage::Page
    );
}
#[test]
fn source_mappings_distinguish_runtime_and_user_ranges() {
    let (temp, w) = create();
    w.save("page.js", "one();\ntwo();").unwrap();
    let p = w
        .snapshot(&temp.path().join("cache"))
        .unwrap()
        .mapped_program()
        .unwrap();
    let user = p
        .mappings
        .iter()
        .find(|m| m.origin == SourceOrigin::User)
        .unwrap();
    assert_eq!(user.generated_end_line - user.generated_start_line, 1);
    assert_eq!(
        p.source.lines().nth(user.generated_start_line - 1),
        Some("one();")
    );
    assert_eq!(
        p.source.lines().nth(user.generated_end_line - 1),
        Some("two();")
    );
    for pair in p.mappings.windows(2) {
        assert_eq!(pair[0].generated_end_line + 1, pair[1].generated_start_line);
    }
}
#[cfg(unix)]
#[test]
fn poisoned_cache_links_and_source_nested_cache_are_rejected() {
    use std::os::unix::fs::symlink;
    let (temp, w) = create();
    let cache = temp.path().join("cache");
    let outside = temp.path().join("outside");
    fs::create_dir(&cache).unwrap();
    fs::create_dir(&outside).unwrap();
    symlink(&outside, cache.join("workspaces")).unwrap();
    assert!(w.snapshot(&cache).is_err());
    assert!(fs::read_dir(&outside).unwrap().next().is_none());
    let link = temp.path().join("source-link");
    symlink(w.root(), &link).unwrap();
    assert!(w.snapshot(&link.join("new-cache")).is_err());
    assert!(!w.root().join("new-cache").exists());
}
#[test]
fn dispatched_attempt_is_unknown_until_confirmed() {
    let mut ledger = ExecutionLedger::default();
    let t = target("dispatch");
    let r = ledger.begin(t.clone(), "v1").unwrap();
    ledger.mark_submitted(&r.request_id, &t).unwrap();
    assert_eq!(ledger.records()[0].certainty, ExecutionCertainty::Unknown);
    assert_eq!(ledger.records()[0].status, ExecutionStatus::Running);
}
