//! A connection owns its backend on one OS thread; WebBackend is deliberately not Send.
use crate::{
    error::{ApiError, Result},
    security::random_id,
};
use injectra_bootstrap::{BootstrapBackend, Broker};
use injectra_core::{Result as CoreResult, TargetRef};
use injectra_web::WebBackend;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, mpsc},
    thread,
};
use tokio::sync::oneshot;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectRequest {
    pub backend: String,
    pub port: Option<u16>,
    pub session: Option<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Target {
    pub id: String,
    pub title: String,
    pub url: String,
    pub generation: String,
    pub backend: String,
    pub backend_instance: String,
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}
struct BoundTarget {
    reference: TargetRef,
    generation: String,
    native: bool,
    view: Target,
}

pub(crate) enum Backend {
    Web(Box<dyn WebBackend>),
    Bootstrap(BootstrapBackend),
}
impl Backend {
    fn web(&self) -> &dyn WebBackend {
        match self {
            Self::Web(b) => b.as_ref(),
            Self::Bootstrap(b) => b,
        }
    }
    fn execute(&self, target: &TargetRef, revision: &str, source: &str) -> CoreResult<Value> {
        match self {
            Self::Web(b) => injectra_application::execute(b.as_ref(), target, source),
            Self::Bootstrap(b) => b.evaluate_revision(target, revision, source),
        }
    }
}
fn create(spec: &ConnectRequest, broker: Arc<Broker>) -> Result<Backend> {
    match spec.backend.as_str() {
        "cdp" if spec.session.is_none() => {
            Ok(Backend::Web(Box::new(injectra_cdp::CdpBackend::new(
                spec.port
                    .filter(|p| *p > 0)
                    .ok_or_else(|| ApiError::invalid("CDP requires port"))?,
            )?)))
        }
        "electron-gadget" if spec.port.is_none() => Ok(Backend::Web(Box::new(
            injectra_electron_gadget::ElectronGadgetBackend::new(
                spec.session
                    .as_deref()
                    .ok_or_else(|| ApiError::invalid("Gadget requires session"))?,
            )?,
        ))),
        "webkit" if spec.port.is_none() => {
            Ok(Backend::Web(Box::new(injectra_webkit::WebkitBackend::new(
                spec.session
                    .as_deref()
                    .ok_or_else(|| ApiError::invalid("WebKit requires session"))?,
            )?)))
        }
        "bootstrap" if spec.port.is_none() && spec.session.is_none() => {
            Ok(Backend::Bootstrap(BootstrapBackend::new(broker)))
        }
        _ => Err(ApiError::unsupported(
            "Unsupported backend or endpoint combination",
        )),
    }
}

enum Command {
    Discover(oneshot::Sender<Result<Vec<Target>>>),
    Execute {
        target: String,
        generation: Option<String>,
        revision: String,
        source: String,
        started: oneshot::Sender<()>,
        reply: oneshot::Sender<Result<Value>>,
    },
    Reload {
        target: String,
        generation: Option<String>,
        reply: oneshot::Sender<Result<Value>>,
    },
}
#[derive(Clone)]
pub struct BackendActor {
    sender: mpsc::SyncSender<Command>,
    pub instance: String,
    pub backend: String,
}
pub struct PendingRun {
    pub started: oneshot::Receiver<()>,
    pub finished: oneshot::Receiver<Result<Value>>,
}
impl BackendActor {
    pub fn connect(spec: ConnectRequest, broker: Arc<Broker>) -> Result<Self> {
        let label = spec.backend.clone();
        Self::start(label, move || create(&spec, broker))
    }
    fn start(
        factory_name: String,
        factory: impl FnOnce() -> Result<Backend> + Send + 'static,
    ) -> Result<Self> {
        let instance = random_id()?;
        let worker_instance = instance.clone();
        let worker_name = factory_name.clone();
        let (tx, rx) = mpsc::sync_channel(32);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("injectra-backend".into())
            .spawn(move || {
                let backend = match factory() {
                    Ok(backend) => backend,
                    Err(error) => {
                        let _ = ready_tx.send(Err(error));
                        return;
                    }
                };
                let mut worker = Worker {
                    backend,
                    instance: worker_instance,
                    name: worker_name,
                    targets: HashMap::new(),
                };
                let _ = ready_tx.send(Ok(()));
                while let Ok(command) = rx.recv() {
                    match command {
                        Command::Discover(reply) => {
                            let _ = reply.send(worker.discover());
                        }
                        Command::Execute {
                            target,
                            generation,
                            revision,
                            source,
                            started,
                            reply,
                        } => {
                            let _ = started.send(());
                            let outcome =
                                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                    worker.execute(
                                        &target,
                                        generation.as_deref(),
                                        &revision,
                                        &source,
                                    )
                                }))
                                .unwrap_or_else(|_| {
                                    Err(ApiError::unknown(
                                        "Backend execution panicked; outcome unknown",
                                    ))
                                });
                            let _ = reply.send(outcome);
                        }
                        Command::Reload {
                            target,
                            generation,
                            reply,
                        } => {
                            let _ = reply.send(worker.reload(&target, generation.as_deref()));
                        }
                    }
                }
            })?;
        ready_rx
            .recv()
            .map_err(|_| ApiError::unknown("Backend worker stopped during initialization"))??;
        Ok(Self {
            sender: tx,
            instance,
            backend: factory_name,
        })
    }
    fn send(&self, command: Command) -> Result<()> {
        self.sender.try_send(command).map_err(|e| match e {
            mpsc::TrySendError::Full(_) => {
                ApiError::new(503, "busy", "Backend request queue is full", "NotExecuted")
            }
            mpsc::TrySendError::Disconnected(_) => ApiError::new(
                503,
                "connection",
                "Backend worker disconnected",
                "NotExecuted",
            ),
        })
    }
    pub async fn targets(&self) -> Result<Vec<Target>> {
        let (tx, rx) = oneshot::channel();
        self.send(Command::Discover(tx))?;
        rx.await
            .map_err(|_| ApiError::unknown("Backend discovery worker disconnected"))?
    }
    pub fn submit(
        &self,
        target: String,
        generation: Option<String>,
        revision: String,
        source: String,
    ) -> Result<PendingRun> {
        let (started, started_rx) = oneshot::channel();
        let (reply, finished) = oneshot::channel();
        self.send(Command::Execute {
            target,
            generation,
            revision,
            source,
            started,
            reply,
        })?;
        Ok(PendingRun {
            started: started_rx,
            finished,
        })
    }
    pub async fn reload(&self, target: String, generation: Option<String>) -> Result<Value> {
        let (reply, rx) = oneshot::channel();
        self.send(Command::Reload {
            target,
            generation,
            reply,
        })?;
        rx.await
            .map_err(|_| ApiError::unknown("Backend reload worker disconnected"))?
    }
}
struct Worker {
    backend: Backend,
    instance: String,
    name: String,
    targets: HashMap<String, BoundTarget>,
}
impl Worker {
    fn marker(&self) -> String {
        format!("injectra.studio.document.{}", self.instance)
    }
    fn discover(&mut self) -> Result<Vec<Target>> {
        let native = match &self.backend {
            Backend::Bootstrap(b) => Some(b.broker().targets()),
            _ => None,
        };
        let mut next = HashMap::new();
        let mut views = Vec::new();
        for target in self.backend.web().targets()? {
            let mut error = None;
            let (generation, capabilities) = if let Some(metadata) = &native {
                let Some(found) = metadata.iter().find(|m| m.id == target.reference.key()) else {
                    continue;
                };
                (found.generation.clone(), found.capabilities.clone())
            } else {
                let candidate = random_id()?;
                let source = format!(
                    "(()=>{{const k=Symbol.for({});if(!Object.prototype.hasOwnProperty.call(globalThis,k))Object.defineProperty(globalThis,k,{{value:{},configurable:false}});return globalThis[k];}})()",
                    json!(self.marker()),
                    json!(candidate)
                );
                match self
                    .backend
                    .execute(&target.reference, "identity-probe", &source)
                {
                    Ok(Value::String(value)) if !value.is_empty() => {
                        (value, vec!["web.page.evaluate".into()])
                    }
                    Ok(_) => {
                        error = Some(ApiError::invalid(
                            "Document identity probe returned an invalid value",
                        ));
                        (String::new(), vec![])
                    }
                    Err(e) => {
                        error = Some(e.into());
                        (String::new(), vec![])
                    }
                }
            };
            let key = injectra_workspace::content_hash(
                format!("{}\0{}", target.reference.key(), generation).as_bytes(),
            );
            let id = format!("{}:{key}", self.instance);
            let view = Target {
                id: id.clone(),
                title: target.title,
                url: target.url,
                generation: generation.clone(),
                backend: self.name.clone(),
                backend_instance: self.instance.clone(),
                capabilities,
                error,
            };
            if view.error.is_none() {
                next.insert(
                    id,
                    BoundTarget {
                        reference: target.reference,
                        generation,
                        native: native.is_some(),
                        view: view.clone(),
                    },
                );
            }
            views.push(view);
        }
        self.targets = next;
        Ok(views)
    }
    fn target(&self, id: &str, generation: Option<&str>) -> Result<&BoundTarget> {
        let target = self.targets.get(id).ok_or_else(ApiError::stale)?;
        if generation.is_some_and(|g| g != target.generation) {
            return Err(ApiError::stale());
        }
        Ok(target)
    }
    fn execute(
        &self,
        id: &str,
        generation: Option<&str>,
        revision: &str,
        source: &str,
    ) -> Result<Value> {
        let target = self.target(id, generation)?;
        if !target
            .view
            .capabilities
            .iter()
            .any(|c| c == "web.page.evaluate")
        {
            return Err(ApiError::unsupported("Target cannot evaluate page code"));
        }
        if target.native {
            return self
                .backend
                .execute(&target.reference, revision, source)
                .map_err(Into::into);
        }
        let envelope = random_id()?;
        let guarded = format!(
            "(async()=>{{if(globalThis[Symbol.for({})]!=={})return {{guard:{},stale:true}};return {{guard:{},stale:false,value:await ({})}};}})()",
            json!(self.marker()),
            json!(target.generation),
            json!(envelope),
            json!(envelope),
            source
        );
        let result = self
            .backend
            .execute(&target.reference, revision, &guarded)?;
        if result.get("guard") != Some(&Value::String(envelope)) {
            return Err(ApiError::unknown("Invalid guarded execution receipt"));
        }
        if result.get("stale") == Some(&Value::Bool(true)) {
            return Err(ApiError::stale());
        }
        if result.get("stale") != Some(&Value::Bool(false)) {
            return Err(ApiError::unknown("Incomplete guarded execution receipt"));
        }
        Ok(result.get("value").cloned().unwrap_or(Value::Null))
    }
    fn reload(&self, id: &str, generation: Option<&str>) -> Result<Value> {
        let target = self.target(id, generation)?;
        if !target
            .view
            .capabilities
            .iter()
            .any(|c| c == "web.page.reload")
        {
            return Err(ApiError::unsupported(
                "Reload is unavailable for this target",
            ));
        }
        match &self.backend {
            Backend::Bootstrap(b) => b.reload(&target.reference).map_err(Into::into),
            _ => Err(ApiError::unsupported(
                "Reload is unavailable for this backend",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use injectra_core::{BackendId, Capability, Certainty, Error, ErrorKind, Phase};
    use injectra_web::{PAGE_EVALUATE, PageContext, WebSession, WebTarget};
    use std::{
        cell::RefCell,
        rc::Rc,
        sync::{
            Mutex,
            atomic::{AtomicUsize, Ordering},
        },
    };
    struct Fake {
        id: BackendId,
        local: Rc<RefCell<usize>>,
        generation: Arc<AtomicUsize>,
        executed: Arc<AtomicUsize>,
        threads: Arc<Mutex<Vec<thread::ThreadId>>>,
    }
    struct Session {
        target: TargetRef,
        local: Rc<RefCell<usize>>,
        generation: Arc<AtomicUsize>,
        executed: Arc<AtomicUsize>,
        threads: Arc<Mutex<Vec<thread::ThreadId>>>,
    }
    impl WebBackend for Fake {
        fn id(&self) -> &BackendId {
            &self.id
        }
        fn targets(&self) -> CoreResult<Vec<WebTarget>> {
            Ok(vec![WebTarget {
                reference: TargetRef::new(&self.id, "same-page"),
                title: "fixture".into(),
                url: "https://fixture.invalid/same".into(),
                context: PageContext::MainFrame,
            }])
        }
        fn connect(&self, t: &TargetRef) -> CoreResult<Box<dyn WebSession>> {
            Ok(Box::new(Session {
                target: t.clone(),
                local: self.local.clone(),
                generation: self.generation.clone(),
                executed: self.executed.clone(),
                threads: self.threads.clone(),
            }))
        }
    }
    impl WebSession for Session {
        fn target(&self) -> &TargetRef {
            &self.target
        }
        fn capabilities(&self) -> &[Capability] {
            &[PAGE_EVALUATE]
        }
        fn evaluate_page(&mut self, source: &str) -> CoreResult<Value> {
            self.threads.lock().unwrap().push(thread::current().id());
            *self.local.borrow_mut() += 1;
            if source.contains("Object.defineProperty") {
                return Ok(json!(format!(
                    "document-{}",
                    self.generation.load(Ordering::SeqCst)
                )));
            }
            if source.contains("simulate_unknown") {
                self.executed.fetch_add(1, Ordering::SeqCst);
                return Err(Error::new(
                    ErrorKind::Timeout,
                    Phase::Execute,
                    Certainty::Unknown,
                    "lost reply",
                ));
            }
            let tail = source.split("guard:").nth(1).unwrap();
            let mut de = serde_json::Deserializer::from_str(tail);
            let guard = String::deserialize(&mut de).unwrap();
            let stale = !source.contains(&format!(
                "\"document-{}\"",
                self.generation.load(Ordering::SeqCst)
            ));
            if !stale {
                self.executed.fetch_add(1, Ordering::SeqCst);
            }
            Ok(json!({"guard":guard,"stale":stale,"value":42}))
        }
    }
    #[tokio::test]
    async fn non_send_backend_stays_on_actor_and_stale_documents_do_not_execute() {
        let generation = Arc::new(AtomicUsize::new(1));
        let executed = Arc::new(AtomicUsize::new(0));
        let threads = Arc::new(Mutex::new(Vec::new()));
        let (g, e, t) = (generation.clone(), executed.clone(), threads.clone());
        let actor = BackendActor::start("fake".into(), move || {
            Ok(Backend::Web(Box::new(Fake {
                id: BackendId::fresh(),
                local: Rc::new(RefCell::new(0)),
                generation: g,
                executed: e,
                threads: t,
            })))
        })
        .unwrap();
        let first = actor.targets().await.unwrap().remove(0);
        generation.store(2, Ordering::SeqCst);
        let pending = actor
            .submit(
                first.id.clone(),
                Some(first.generation.clone()),
                "rev".into(),
                "42".into(),
            )
            .unwrap();
        assert_eq!(
            pending.finished.await.unwrap().unwrap_err().certainty,
            "NotExecuted"
        );
        assert_eq!(executed.load(Ordering::SeqCst), 0);
        let second = actor.targets().await.unwrap().remove(0);
        assert_ne!(first.id, second.id);
        let pending = actor
            .submit(first.id, None, "rev".into(), "42".into())
            .unwrap();
        assert_eq!(
            pending.finished.await.unwrap().unwrap_err().kind,
            "target_not_found"
        );
        let pending = actor
            .submit(
                second.id,
                Some(second.generation),
                "rev".into(),
                "simulate_unknown".into(),
            )
            .unwrap();
        assert_eq!(
            pending.finished.await.unwrap().unwrap_err().certainty,
            "Unknown"
        );
        assert_eq!(executed.load(Ordering::SeqCst), 1);
        let values = threads.lock().unwrap();
        assert!(values.iter().all(|id| *id == values[0]));
        assert_ne!(values[0], thread::current().id());
    }
}
