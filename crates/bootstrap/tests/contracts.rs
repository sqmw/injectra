use injectra_bootstrap::*;
use injectra_core::Certainty;
use std::{sync::Arc, thread, time::Duration};
fn connected() -> (Arc<Broker>, String, Page) {
    let b = Arc::new(Broker::new());
    let key = random_token();
    b.register_artifact("fixture", &key).unwrap();
    let hello = Hello {
        artifact: "fixture".into(),
        instance: random_token(),
        pid: 42,
        client_nonce: random_token(),
    };
    let c = b.challenge(hello.clone()).unwrap();
    assert_eq!(
        c.server_proof,
        proof(
            &key,
            "server",
            &hello.artifact,
            &hello.instance,
            hello.pid,
            &hello.client_nonce,
            &c.server_nonce
        )
        .unwrap()
    );
    let r = b
        .authenticate(Register {
            challenge_id: c.challenge_id,
            client_proof: proof(
                &key,
                "client",
                &hello.artifact,
                &hello.instance,
                hello.pid,
                &hello.client_nonce,
                &c.server_nonce,
            )
            .unwrap(),
        })
        .unwrap();
    let p = Page {
        id: "1".into(),
        generation: "document1".into(),
        title: "fixture".into(),
        url: "file:///fixture.html".into(),
    };
    b.report(
        &r.session,
        Report {
            pages: vec![p.clone()],
            receipts: vec![],
            logs: vec![],
            startup: None,
        },
    )
    .unwrap();
    (b, r.session, p)
}
fn receive(b: &Broker, s: &str) -> Command {
    for _ in 0..100 {
        if let Some(c) = b.poll(s).unwrap() {
            return c;
        }
        thread::sleep(Duration::from_millis(2));
    }
    panic!("No command");
}
fn ack(c: &Command) -> Receipt {
    Receipt {
        request_id: c.request_id.clone(),
        page: c.page.clone(),
        generation: c.generation.clone(),
        revision: c.revision.clone(),
        ok: true,
        value: serde_json::json!(7),
        error: None,
        certainty: "Executed".into(),
    }
}
#[test]
fn role_and_replay_are_rejected() {
    let b = Broker::new();
    let key = random_token();
    b.register_artifact("fixture", &key).unwrap();
    let h = Hello {
        artifact: "fixture".into(),
        instance: random_token(),
        pid: 1,
        client_nonce: random_token(),
    };
    let c = b.challenge(h).unwrap();
    let r = Register {
        challenge_id: c.challenge_id,
        client_proof: c.server_proof,
    };
    assert!(b.authenticate(r.clone()).is_err());
    assert!(b.authenticate(r).is_err());
}
#[test]
fn targets_ack_independently_and_old_generations_cannot_execute() {
    let (b, s, p) = connected();
    let p2 = Page {
        id: "2".into(),
        ..p.clone()
    };
    b.report(
        &s,
        Report {
            pages: vec![p.clone(), p2.clone()],
            receipts: vec![],
            logs: vec![],
            startup: None,
        },
    )
    .unwrap();
    let t = b.targets().into_iter().find(|t| t.page == "1").unwrap();
    let worker = {
        let b = b.clone();
        let id = t.id.clone();
        thread::spawn(move || b.submit(&id, "v1", "1+6", Duration::from_secs(1)))
    };
    let c = receive(&b, &s);
    b.report(
        &s,
        Report {
            pages: vec![p.clone(), p2.clone()],
            receipts: vec![ack(&c)],
            logs: vec![],
            startup: None,
        },
    )
    .unwrap();
    assert_eq!(worker.join().unwrap().unwrap(), 7);
    let targets = b.targets();
    assert_eq!(
        targets
            .iter()
            .find(|t| t.page == "1")
            .unwrap()
            .applied_revision
            .as_deref(),
        Some("v1")
    );
    assert_eq!(
        targets
            .iter()
            .find(|t| t.page == "2")
            .unwrap()
            .applied_revision,
        None
    );
    let new = Page {
        generation: "document2".into(),
        ..p
    };
    b.report(
        &s,
        Report {
            pages: vec![new, p2],
            receipts: vec![],
            logs: vec![],
            startup: None,
        },
    )
    .unwrap();
    assert_eq!(
        b.submit(&t.id, "v2", "8", Duration::from_millis(1))
            .unwrap_err()
            .certainty,
        Certainty::NotExecuted
    );
}
#[test]
fn timeout_does_not_redeliver_and_late_receipt_cannot_claim_applied() {
    let (b, s, p) = connected();
    let t = b.targets().remove(0);
    let e = b
        .submit(&t.id, "never-polled", "1", Duration::from_millis(2))
        .unwrap_err();
    assert_eq!(e.certainty, Certainty::NotExecuted);
    assert!(b.poll(&s).unwrap().is_none());
    let worker = {
        let b = b.clone();
        thread::spawn(move || b.submit(&t.id, "uncertain", "2", Duration::from_millis(40)))
    };
    let c = receive(&b, &s);
    assert_eq!(
        worker.join().unwrap().unwrap_err().certainty,
        Certainty::Unknown
    );
    assert!(b.poll(&s).unwrap().is_none());
    b.report(
        &s,
        Report {
            pages: vec![p],
            receipts: vec![ack(&c)],
            logs: vec![],
            startup: None,
        },
    )
    .unwrap();
    assert_eq!(b.targets()[0].applied_revision, None);
}
#[test]
fn wrong_revision_is_rejected_without_completing_request() {
    let (b, s, p) = connected();
    let t = b.targets().remove(0);
    let worker = {
        let b = b.clone();
        thread::spawn(move || b.submit(&t.id, "v1", "2", Duration::from_secs(1)))
    };
    let c = receive(&b, &s);
    let mut wrong = ack(&c);
    wrong.revision = "v2".into();
    assert!(
        b.report(
            &s,
            Report {
                pages: vec![p.clone()],
                receipts: vec![wrong],
                logs: vec![],
                startup: None
            }
        )
        .is_err()
    );
    b.report(
        &s,
        Report {
            pages: vec![p],
            receipts: vec![ack(&c)],
            logs: vec![],
            startup: None,
        },
    )
    .unwrap();
    assert_eq!(worker.join().unwrap().unwrap(), 7);
}
#[test]
fn disconnect_preserves_delivery_uncertainty() {
    let (b, s, _) = connected();
    let t = b.targets().remove(0);
    let worker = {
        let b = b.clone();
        thread::spawn(move || b.submit(&t.id, "v1", "2", Duration::from_secs(1)))
    };
    receive(&b, &s);
    b.disconnect(&s);
    assert_eq!(
        worker.join().unwrap().unwrap_err().certainty,
        Certainty::Unknown
    );
    assert!(b.targets().is_empty());
}
#[test]
fn node_and_rust_use_the_same_proof_bytes() {
    let key = "11".repeat(32);
    let expected = proof(&key, "server", "fixture", "process", 42, "client", "server").unwrap();
    let script = "const c=require('crypto');process.stdout.write(c.createHmac('sha256',Buffer.from(process.argv[1],'hex')).update(JSON.stringify(['injectra-bootstrap-v1','server','fixture','process',42,'client','server'])).digest('hex'))";
    let out = std::process::Command::new("node")
        .args(["-e", script, &key])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap(), expected);
}
