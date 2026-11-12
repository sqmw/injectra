//! Foreground CLI assembly: bridge, owned app, source polling and compact receipts.
use injectra_application::dev::{Revision, Updates};
use injectra_bootstrap::{BootstrapBackend, Broker};
use injectra_core::{Certainty, Error, ErrorKind, Result};
use injectra_preparation::PreparedArtifact;
use injectra_web::WebBackend;
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, IsTerminal, Write},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
mod journal;
use journal::Journal;
fn output(value: Value) {
    println!("{value}");
}
pub fn run(
    artifact: &Path,
    bundle: &Path,
    selected: Option<&str>,
    acknowledgement: Option<&str>,
) -> Result<Value> {
    let artifact = PreparedArtifact::load(artifact)?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(artifact.root().join("dev.lock"))?;
    fs2::FileExt::try_lock_exclusive(&lock).map_err(|_| {
        Error::prepare(
            ErrorKind::Connection,
            "Another dev command owns this artifact",
        )
    })?;
    let journal = Journal::open(artifact.root(), acknowledgement)?;
    // Validate source before starting an app, but tolerate later temporary editor saves.
    Revision::load(bundle)?;
    let address = artifact
        .controller_origin
        .trim_start_matches("http://")
        .trim_end_matches('/');
    let listener = std::net::TcpListener::bind(address).map_err(|e| {
        Error::prepare(
            ErrorKind::Connection,
            format!("Dev port conflict at {address}: {e}"),
        )
    })?;
    listener.set_nonblocking(true)?;
    let broker = Arc::new(Broker::new());
    broker.register_artifact(&artifact.id, &artifact.bridge_key()?)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let stop = Arc::new(AtomicBool::new(false));
    let signal = stop.clone();
    let server = {
        let _enter = runtime.enter();
        let listener = tokio::net::TcpListener::from_std(listener)?;
        let router = injectra_bootstrap::router(broker.clone());
        runtime.spawn(async move { axum::serve(listener, router).await })
    };
    runtime.spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        signal.store(true, Ordering::Relaxed);
    });
    let result = watch(&artifact, bundle, selected, &broker, &journal, &stop);
    broker.unregister_artifact(&artifact.id);
    server.abort();
    runtime.shutdown_timeout(Duration::from_secs(1));
    result
}
fn watch(
    artifact: &PreparedArtifact,
    bundle: &Path,
    selected: Option<&str>,
    broker: &Arc<Broker>,
    journal: &Journal,
    stop: &AtomicBool,
) -> Result<Value> {
    let pid = injectra_preparation::launch(artifact)?;
    output(json!({"event":"dev-ready","pid":pid,"artifact":artifact.artifact_file}));
    let backend = BootstrapBackend::new(broker.clone());
    let start = Instant::now();
    let targets = loop {
        if stop.load(Ordering::Relaxed) {
            return Ok(json!({"event":"dev-stopped"}));
        }
        let targets = backend.targets()?;
        if !targets.is_empty() {
            break targets;
        }
        if start.elapsed() > Duration::from_secs(30) {
            return Err(Error::prepare(
                ErrorKind::Connection,
                format!(
                    "No dev page connected; inspect {}",
                    artifact.root().join("process.log").display()
                ),
            ));
        }
        thread::sleep(Duration::from_millis(150));
    };
    let target = if let Some(id) = selected {
        targets
            .into_iter()
            .find(|t| {
                t.reference.key() == id
                    || broker
                        .targets()
                        .iter()
                        .any(|p| p.id == t.reference.key() && p.page == id && p.pid == pid)
            })
            .ok_or("Specified target is not connected")?
    } else if targets.len() == 1 {
        targets.into_iter().next().unwrap()
    } else {
        output(
            json!({"event":"targets","targets":targets.iter().map(|t|json!({"id":broker.targets().iter().find(|p|p.id==t.reference.key()).map(|p|p.page.clone()),"title":t.title,"url":t.url})).collect::<Vec<_>>()}),
        );
        if !io::stdin().is_terminal() {
            return Err("Multiple pages; provide --target from the target list".into());
        }
        for (i, t) in targets.iter().enumerate() {
            eprintln!("{}: {}  {}", i + 1, t.title, t.url);
        }
        eprint!("Target number: ");
        io::stderr().flush()?;
        let input;
        let (sender, receiver) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let mut line = String::new();
            let result = io::stdin().read_line(&mut line).map(|_| line);
            let _ = sender.send(result);
        });
        loop {
            if stop.load(Ordering::Relaxed) {
                return Ok(json!({"event":"dev-stopped"}));
            }
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(result) => {
                    input = result?;
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => return Err("Target selection input closed".into()),
            }
        }
        let index = input
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|i| *i > 0 && *i <= targets.len())
            .ok_or("Invalid target number")?;
        targets.into_iter().nth(index - 1).unwrap()
    };
    output(json!({"event":"target-selected","target":target.reference.key(),"title":target.title}));
    let mut updates = Updates::default();
    let mut candidate = None;
    let mut last_error = String::new();
    let mut sequence = 0;
    loop {
        if stop.load(Ordering::Relaxed) {
            return Ok(json!({"event":"dev-stopped"}));
        }
        if !backend
            .targets()?
            .iter()
            .any(|t| t.reference == target.reference)
        {
            return Err(Error::prepare(
                ErrorKind::TargetNotFound,
                "Selected page closed or changed; run dev again to select its current page",
            ));
        }
        match Revision::load(bundle) {
            Err(error) => {
                let message = error.to_string();
                if message != last_error {
                    output(
                        json!({"event":"source-error","error":message,"certainty":"NotExecuted"}),
                    );
                    last_error = message;
                }
                candidate = None;
            }
            Ok(revision) => {
                last_error.clear();
                if updates.changed(&revision) && candidate.as_ref() == Some(&revision.hash) {
                    let request = injectra_bootstrap::random_token();
                    journal.begin(
                        &request,
                        &revision.hash,
                        target.reference.key(),
                        &revision.files,
                    )?;
                    let result = updates.apply(
                        &backend.for_submission(&request, &revision.hash),
                        &target.reference,
                        &revision,
                    );
                    let certainty = match &result {
                        Ok(_) => Certainty::Executed,
                        Err(e) => e.certainty,
                    };
                    let receipt = json!({"event":"execution","request_id":request,"version":revision.hash,"target":target.reference.key(),"files":revision.files,"location":result.as_ref().err().and_then(|e|revision.source_location(&e.to_string())),"certainty":format!("{certainty:?}"),"result":result.as_ref().ok(),"error":result.as_ref().err().map(ToString::to_string)});
                    journal.complete(&receipt, certainty)?;
                    output(receipt);
                    if certainty == Certainty::Unknown {
                        return Err("Execution result unknown; automatic updates stopped. Review effects, then explicitly use --acknowledge-unknown <request_id> for a new attempt".into());
                    }
                }
                candidate = Some(revision.hash);
            }
        }
        for event in broker.events(sequence) {
            sequence = event.sequence;
            if event.kind == "console" {
                output(json!({"event":"log","data":event.data}));
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
}
