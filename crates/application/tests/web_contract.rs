mod fixture {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/support/bundle.rs"
    ));
}
use injectra_application::{apply, eval, execute, list};
use injectra_core::{BackendId, Capability, Certainty, Error, ErrorKind, Phase, Result, TargetRef};
use injectra_web::{PAGE_EVALUATE, WebBackend, WebSession, WebTarget};
use serde_json::{Value, json};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

struct Backend {
    id: BackendId,
    connects: Cell<usize>,
    programs: Rc<RefCell<Vec<String>>>,
    caps: Vec<Capability>,
    wrong_target: bool,
    fail: bool,
}
impl Backend {
    fn page() -> Self {
        Self {
            id: BackendId::fresh(),
            connects: Cell::new(0),
            programs: Rc::default(),
            caps: vec![PAGE_EVALUATE],
            wrong_target: false,
            fail: false,
        }
    }
    fn target(&self) -> TargetRef {
        TargetRef::new(&self.id, "same-key")
    }
}
struct Session {
    target: TargetRef,
    programs: Rc<RefCell<Vec<String>>>,
    caps: Vec<Capability>,
    fail: bool,
}
impl WebSession for Session {
    fn target(&self) -> &TargetRef {
        &self.target
    }
    fn capabilities(&self) -> &[Capability] {
        &self.caps
    }
    fn evaluate_page(&mut self, source: &str) -> Result<Value> {
        self.programs.borrow_mut().push(source.into());
        if self.fail {
            return Err(Error::new(
                ErrorKind::Connection,
                Phase::Execute,
                Certainty::Unknown,
                "lost after submit",
            )
            .caused_by(std::io::Error::other("transport source")));
        }
        Ok(json!({"fixture":true}))
    }
}
impl WebBackend for Backend {
    fn id(&self) -> &BackendId {
        &self.id
    }
    fn targets(&self) -> Result<Vec<WebTarget>> {
        Ok(vec![])
    }
    fn connect(&self, target: &TargetRef) -> Result<Box<dyn WebSession>> {
        self.connects.set(self.connects.get() + 1);
        Ok(Box::new(Session {
            target: if self.wrong_target {
                TargetRef::new(&self.id, "other")
            } else {
                target.clone()
            },
            programs: self.programs.clone(),
            caps: self.caps.clone(),
            fail: self.fail,
        }))
    }
}
#[test]
fn the_same_bundle_program_is_reused_across_backends() {
    let a = Backend::page();
    let b = Backend::page();
    let input = fixture::BundleFixture::new();
    apply(&a, &a.target(), &input.manifest).unwrap();
    apply(&b, &b.target(), &input.manifest).unwrap();
    assert_eq!(*a.programs.borrow(), *b.programs.borrow());
    assert!(a.programs.borrow()[0].contains("injectraRuntime"));
}
#[test]
fn foreign_target_is_rejected_before_connecting() {
    let a = Backend::page();
    let b = Backend::page();
    assert_eq!(
        execute(&b, &a.target(), "1").unwrap_err().certainty,
        Certainty::NotExecuted
    );
    assert_eq!(b.connects.get(), 0);
}
#[test]
fn generic_javascript_and_mismatched_session_cannot_execute_page_payloads() {
    let mut backend = Backend::page();
    backend.caps = vec![Capability("js.evaluate")];
    assert_eq!(
        execute(&backend, &backend.target(), "1").unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert!(backend.programs.borrow().is_empty());
    backend.caps = vec![PAGE_EVALUATE];
    backend.wrong_target = true;
    assert_eq!(
        execute(&backend, &backend.target(), "1").unwrap_err().kind,
        ErrorKind::InvalidInput
    );
    assert!(backend.programs.borrow().is_empty());
}
#[test]
fn unknown_execution_and_source_survive_without_retry() {
    let mut backend = Backend::page();
    backend.fail = true;
    let error = execute(&backend, &backend.target(), "sideEffect()").unwrap_err();
    assert_eq!(error.certainty, Certainty::Unknown);
    assert_eq!(error.phase, Phase::Execute);
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(backend.programs.borrow().len(), 1);
}

#[test]
fn dev_revisions_track_content_and_keep_unknown_attempts_distinct_from_saves() {
    use injectra_application::dev::{Revision, Updates};
    let fixture = fixture::BundleFixture::new();
    let first = Revision::load(&fixture.manifest).unwrap();
    assert_eq!(first.hash, Revision::load(&fixture.manifest).unwrap().hash);
    std::fs::write(
        fixture.manifest.parent().unwrap().join("style.css"),
        "body {color: pink}",
    )
    .unwrap();
    let next = Revision::load(&fixture.manifest).unwrap();
    assert_ne!(first.hash, next.hash);
    let mut backend = Backend::page();
    backend.fail = true;
    let mut updates = Updates::default();
    assert!(updates.changed(&next));
    assert_eq!(
        updates
            .apply(&backend, &backend.target(), &next)
            .unwrap_err()
            .certainty,
        Certainty::Unknown
    );
    assert!(!updates.changed(&next));
    assert!(updates.changed(&first));
    assert_eq!(backend.programs.borrow().len(), 1);
    std::fs::write(&fixture.manifest, "{").unwrap();
    assert!(Revision::load(&fixture.manifest).is_err());
}

#[test]
fn list_and_eval_reuse_contracts_and_preserve_certainty_without_retry() {
    let a = Backend::page();
    let b = Backend::page();
    list(&a, &a.target()).unwrap();
    list(&b, &b.target()).unwrap();
    assert_eq!(*a.programs.borrow(), *b.programs.borrow());
    assert!(a.programs.borrow()[0].contains(r#""operation":"list""#));

    let mut missing_cap = Backend::page();
    missing_cap.caps = vec![Capability("js.evaluate")];
    assert_eq!(
        list(&missing_cap, &missing_cap.target()).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert!(missing_cap.programs.borrow().is_empty());

    let blank = Backend::page();
    let err = eval(&blank, &blank.target(), "   ").unwrap_err();
    assert_eq!(err.certainty, Certainty::NotExecuted);
    assert_eq!(blank.connects.get(), 0);

    let mut failing = Backend::page();
    failing.fail = true;
    let err = eval(&failing, &failing.target(), "document.title").unwrap_err();
    assert_eq!(err.certainty, Certainty::Unknown);
    assert_eq!(failing.programs.borrow().len(), 1);
}
