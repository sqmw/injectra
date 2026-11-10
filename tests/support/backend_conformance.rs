use injectra_core::{Certainty, ErrorKind, TargetRef};
use injectra_web::{Bundle, WebBackend};
use serde_json::{Value, json};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn verify(backend: &dyn WebBackend, selected: &str) {
    let target = backend
        .targets()
        .unwrap()
        .into_iter()
        .find(|p| p.reference.key() == selected)
        .expect("explicit page");
    let reference = target.reference;
    let id = format!(
        "injectra.test.{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("manifest.json");
    fs::write(
        &manifest,
        json!({"version":1,"id":id,"css":"style.css","html":"content.html","js":"run.js"})
            .to_string(),
    )
    .unwrap();
    fs::write(
        dir.path().join("style.css"),
        format!("[data-injectra=\"{id}\"] button {{color:rgb(1,2,3)}}"),
    )
    .unwrap();
    fs::write(
        dir.path().join("content.html"),
        "<button data-count=\"0\">Injectra test</button>",
    )
    .unwrap();
    fs::write(dir.path().join("run.js"),"const b=context.root.querySelector('button');const click=()=>b.dataset.count=String(Number(b.dataset.count)+1);b.addEventListener('click',click);context.onCleanup(()=>b.removeEventListener('click',click));await Promise.resolve();").unwrap();
    let evaluate = |source: &str| injectra_application::execute(backend, &reference, source);
    let selector = json!(format!("[data-injectra=\"{id}\"]")).to_string();
    let button = json!(format!("[data-injectra=\"{id}\"] button")).to_string();
    let count = format!("document.querySelectorAll({selector}).length");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert_eq!(
            injectra_application::apply(backend, &reference, &manifest).unwrap()["status"],
            "applied"
        );
        assert_eq!(evaluate(&count).unwrap(), 2);
        assert_eq!(
            evaluate(&format!(
                "getComputedStyle(document.querySelector({button})).color"
            ))
            .unwrap(),
            "rgb(1, 2, 3)"
        );
        assert_eq!(evaluate(&format!("(()=>{{const b=document.querySelector({button});b.click();return b.dataset.count;}})()")).unwrap(),"1");
        assert_eq!(
            evaluate("Promise.resolve({answer:42})").unwrap(),
            json!({"answer":42})
        );
        for source in [
            "(()=>{throw Error('injectra-intentional');})()",
            "Promise.reject(Error('injectra-intentional'))",
        ] {
            let error = evaluate(source).unwrap_err();
            assert_eq!(error.certainty, Certainty::Executed);
            assert!(error.to_string().contains("injectra-intentional"));
        }
        let cyclic = evaluate("(()=>{const x={};x.x=x;return x;})()").unwrap_err();
        // CDP may report serialization failure as a protocol error, without
        // execution-stage proof. Preserve its existing conservative contract.
        assert!(
            matches!(
                (cyclic.kind, cyclic.certainty),
                (ErrorKind::ResultEncoding, Certainty::Executed)
                    | (ErrorKind::Protocol, Certainty::Unknown)
            ),
            "unexpected encoding failure: {cyclic:?}"
        );
        for other in backend
            .targets()
            .unwrap()
            .into_iter()
            .filter(|p| p.reference != reference && p.url.starts_with("http"))
        {
            assert_eq!(
                injectra_application::execute(backend, &other.reference, &count).unwrap(),
                0
            );
        }
        assert_eq!(
            injectra_application::apply(backend, &reference, &manifest).unwrap()["status"],
            "updated"
        );
        assert_eq!(evaluate(&count).unwrap(), 2);
        assert_eq!(
            injectra_application::remove(backend, &reference, &id).unwrap()["status"],
            "removed"
        );
        assert_eq!(evaluate(&count).unwrap(), 0);
        assert_eq!(
            injectra_application::remove(backend, &reference, &id).unwrap()["status"],
            "absent"
        );
        let missing = TargetRef::new(backend.id(), "does-not-exist");
        assert_eq!(
            injectra_application::execute(backend, &missing, "1")
                .unwrap_err()
                .certainty,
            Certainty::NotExecuted
        );
        let error = evaluate("new Promise(()=>{})").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Timeout);
        assert_eq!(error.certainty, Certainty::Unknown);
        assert_eq!(evaluate("40+2").unwrap(), Value::from(42));
        // Bundle compilation remains shared, independently of backend choice.
        assert!(
            injectra_web::apply(&Bundle::load(&manifest).unwrap())
                .unwrap()
                .contains("injectraRuntime")
        );
    }));
    let cleanup = injectra_application::remove(backend, &reference, &id);
    assert!(cleanup.is_ok(), "test-owned cleanup failed: {cleanup:?}");
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}
