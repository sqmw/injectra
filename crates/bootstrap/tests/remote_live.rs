//! Explicit optional real remote Electron fixture. Not a preparation/product backend claim.
use injectra_bootstrap::{BootstrapBackend, Broker};
use injectra_core::{Certainty, Error, ErrorKind, Phase, Result};
use injectra_web::{Bundle, WebBackend};
use serde_json::{Value, json};
use std::{
    fs,
    net::TcpListener,
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

#[test]
#[ignore = "explicit owned remote Electron config and SSH bridge required"]
fn owned_remote_electron_page_contract() {
    let file = PathBuf::from(
        std::env::var_os("INJECTRA_REMOTE_ELECTRON_CONFIG").expect("explicit config required"),
    );
    let config: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    let output = file.parent().unwrap();
    let listener = TcpListener::bind(config["listen"].as_str().unwrap()).unwrap();
    assert!(listener.local_addr().unwrap().ip().is_loopback());
    listener.set_nonblocking(true).unwrap();
    let broker = Arc::new(Broker::new());
    broker
        .register_artifact(
            config["artifact"].as_str().unwrap(),
            config["key"].as_str().unwrap(),
        )
        .unwrap();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let task = {
        let _guard = runtime.enter();
        let listener = tokio::net::TcpListener::from_std(listener).unwrap();
        let router = injectra_bootstrap::router(broker.clone());
        runtime.spawn(async move { axum::serve(listener, router).await })
    };
    fs::write(output.join("ready.json"), b"{\"ready\":true}").unwrap();
    let backend = BootstrapBackend::new(broker.clone());
    let mut evidence = json!({"controller":"macos-rust-test","target":"windows","checks":[]});
    let attempt: Result<Value> = (|| {
        let start = Instant::now();
        let target = loop {
            let targets = backend.targets()?;
            evidence["targets"] = json!(broker.targets());
            if let Some(target) = targets.into_iter().find(|t| {
                t.url
                    .split(['?', '#'])
                    .next()
                    .unwrap_or("")
                    .ends_with(config["suffix"].as_str().unwrap_or(""))
            }) {
                break target;
            }
            if start.elapsed() > Duration::from_secs(100) {
                return Err(Error::from("Remote page connection deadline"));
            }
            thread::sleep(Duration::from_millis(150));
        };
        let nonce = uuid::Uuid::new_v4().simple().to_string();
        let id = format!("injectra.remote.{nonce}");
        let counter = format!("__injectraRemoteCleaned{nonce}");
        for (stage, label) in [(0, "first"), (1, "second")] {
            let bundle = Bundle {
                id: id.clone(),
                html: format!("<button>{label}</button>"),
                css: format!("[data-injectra=\"{id}\"] button {{ color:rgb(12,34,56); }}"),
                js: format!(
                    "const b=context.root.querySelector('button');if(b.textContent!=={label:?}||getComputedStyle(b).color!=='rgb(12, 34, 56)')throw Error('page markup/style mismatch');if((globalThis.{counter}??0)!=={stage})throw Error('cleanup order');let n=0;const click=()=>{{b.dataset.count=String(++n)}};b.addEventListener('click',click);context.onCleanup(()=>{{b.removeEventListener('click',click);globalThis.{counter}=(globalThis.{counter}??0)+1}});b.click();if(b.dataset.count!=='1')throw Error('own click callback');"
                ),
            };
            let result = backend.evaluate_revision(
                &target.reference,
                label,
                &injectra_web::apply(&bundle)?,
            )?;
            if result["status"] != if stage == 0 { "applied" } else { "updated" } {
                return Err(Error::new(
                    ErrorKind::Execution,
                    Phase::Execute,
                    Certainty::Executed,
                    "Remote apply status mismatch",
                ));
            }
            evidence["checks"]
                .as_array_mut()
                .unwrap()
                .push(json!({"stage":label,"result":result}));
        }
        let result =
            backend.evaluate_revision(&target.reference, "remove", &injectra_web::remove(&id)?)?;
        if result["status"] != "removed" {
            return Err(Error::new(
                ErrorKind::Execution,
                Phase::Execute,
                Certainty::Executed,
                "Remote remove status mismatch",
            ));
        }
        let inspect = format!(
            "(()=>{{const value={{nodes:document.querySelectorAll('[data-injectra=\"{id}\"]').length,cleaned:globalThis.{counter}}};delete globalThis.{counter};return value}})()"
        );
        let inspected = backend.evaluate_revision(&target.reference, "inspect", &inspect)?;
        if inspected != json!({"nodes":0,"cleaned":2}) {
            return Err(Error::new(
                ErrorKind::Execution,
                Phase::Execute,
                Certainty::Executed,
                "Owned cleanup assertion failed",
            ));
        }
        Ok(json!({"remove":result,"after":inspected}))
    })();
    match &attempt {
        Ok(value) => {
            evidence["passed"] = json!(true);
            evidence["final"] = value.clone();
        }
        Err(error) => {
            evidence["passed"] = json!(false);
            evidence["error"] = json!(error.to_string());
            evidence["certainty"] = json!(format!("{:?}", error.certainty));
        }
    }
    fs::write(
        output.join("result.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    broker.unregister_artifact(config["artifact"].as_str().unwrap());
    task.abort();
    runtime.shutdown_timeout(Duration::from_secs(1));
    assert!(attempt.is_ok(), "{}", evidence);
}
