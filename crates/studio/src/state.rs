use crate::{
    actor::{BackendActor, ConnectRequest, Target},
    artifacts::{ArtifactPort, ArtifactProjection, ReadonlySource},
    endpoint::EndpointRecord,
    error::{ApiError, Result},
    history::History,
    model::{Run, Startup, attribution, target_identity},
    security::BrowserAuth,
};
use injectra_bootstrap::Broker;
use injectra_workspace::{
    ExecutionCertainty, ExecutionLedger, ExecutionOutcome, SaveOutcome, Snapshot, Workspace,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};
use tokio::sync::broadcast;

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
    mutex
        .lock()
        .map_err(|_| ApiError::unknown("Studio state lock poisoned"))
}
struct Sources {
    root: PathBuf,
    workspace: Option<Workspace>,
}
struct Model {
    workspace: Value,
    files: Vec<Value>,
    revision: Option<String>,
    startup_present: bool,
    targets: Vec<Target>,
    actor: Option<BackendActor>,
    connection: u64,
    ledger: ExecutionLedger,
    runs: BTreeMap<String, Run>,
    startup: Startup,
    artifact: Option<Value>,
    readonly: BTreeMap<String, ReadonlySource>,
    artifact_capabilities: Vec<String>,
    last_error: Option<ApiError>,
}
pub struct StudioState {
    sources: Mutex<Sources>,
    model: Mutex<Model>,
    pub cache: PathBuf,
    pub endpoint: EndpointRecord,
    pub auth: Arc<BrowserAuth>,
    pub broker: Arc<Broker>,
    pub events: broadcast::Sender<Value>,
    pub history: History,
    pub artifacts: Option<Arc<dyn ArtifactPort>>,
}
impl StudioState {
    pub fn new(
        root: &Path,
        cache: PathBuf,
        endpoint: EndpointRecord,
        directory: PathBuf,
        broker: Arc<Broker>,
        artifacts: Option<Arc<dyn ArtifactPort>>,
    ) -> Result<Arc<Self>> {
        fs::create_dir_all(root)?;
        let root = root.canonicalize()?;
        let workspace = if fs::read_dir(&root)?.next().is_none() {
            None
        } else {
            Some(Workspace::open(&root)?)
        };
        let history = History::new(directory.join("runs"));
        let runs = history
            .load()?
            .into_iter()
            .map(|r| (r.request_id.clone(), r))
            .collect();
        let auth = Arc::new(BrowserAuth::new(endpoint.token.clone(), endpoint.origin())?);
        let (events, _) = broadcast::channel(128);
        let state = Arc::new(Self {
            sources: Mutex::new(Sources { root, workspace }),
            model: Mutex::new(Model {
                workspace: Value::Null,
                files: vec![],
                revision: None,
                startup_present: false,
                targets: vec![],
                actor: None,
                connection: 0,
                ledger: ExecutionLedger::default(),
                runs,
                startup: Startup::default(),
                artifact: None,
                readonly: BTreeMap::new(),
                artifact_capabilities: vec![],
                last_error: None,
            }),
            cache,
            endpoint,
            auth,
            broker,
            events,
            history,
            artifacts,
        });
        state.refresh_sources()?;
        state.refresh_artifact()?;
        Ok(state)
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        let sources = lock(&self.sources)?;
        let workspace = sources
            .workspace
            .as_ref()
            .ok_or_else(|| ApiError::conflict("Create or import this empty workspace first"))?;
        Ok(workspace.snapshot(&self.cache)?)
    }
    pub fn refresh_sources(&self) -> Result<bool> {
        let sources = lock(&self.sources)?;
        let (snapshot, files) = match &sources.workspace {
            Some(w) => {
                let s = w.snapshot(&self.cache)?;
                let f = s
                    .files
                    .iter()
                    .map(|file| {
                        let mut v = serde_json::to_value(file).expect("file serializes");
                        v["relative_path"] = json!(file.path);
                        v["path"] = json!(format!("user:{}", file.path));
                        v
                    })
                    .collect::<Vec<_>>();
                (Some(s), f)
            }
            None => (None, vec![]),
        };
        let mut model = lock(&self.model)?;
        let revision = snapshot.as_ref().map(|s| s.revision.clone());
        let changed = model.revision != revision || model.files != files;
        model.workspace = json!({"id":snapshot.as_ref().map(|s|s.workspace_id.as_str()),"root":sources.root,"revision":revision});
        model.files = files;
        model.revision = revision.clone();
        model.startup_present = snapshot.as_ref().is_some_and(|s| s.startup.is_some());
        model.startup.saved_revision = if model.startup_present {
            revision
        } else {
            None
        };
        Self::startup_state(&mut model);
        model.last_error = None;
        Ok(changed)
    }
    fn startup_state(model: &mut Model) {
        let configured = model.startup_present;
        let applied = model.startup.applied_revision.is_some();
        model.startup.pending_restart = model.startup.saved_revision
            != model.startup.applied_revision
            && (configured || applied);
        if model.startup.status != "failed" {
            model.startup.status = if model.startup.pending_restart {
                "pending_restart"
            } else if applied {
                "executed"
            } else {
                "not_configured"
            }
            .into();
        }
    }
    pub fn refresh_artifact(&self) -> Result<()> {
        let projection = match &self.artifacts {
            Some(a) => a.projection()?,
            None => ArtifactProjection::default(),
        };
        let mut model = lock(&self.model)?;
        model.artifact = projection.artifact;
        model.readonly = projection
            .files
            .into_iter()
            .map(|f| (f.path.clone(), f))
            .collect();
        model.artifact_capabilities = projection.capabilities;
        Ok(())
    }
    pub fn state(&self) -> Result<Value> {
        let model = lock(&self.model)?;
        let mut files = model.files.clone();
        files.extend(
            model
                .readonly
                .values()
                .map(|v| serde_json::to_value(v).expect("source serializes")),
        );
        let targets = model
            .targets
            .iter()
            .map(|target| {
                let mut v = serde_json::to_value(target).expect("target serializes");
                v["applied_revision"] =
                    json!(model.ledger.applied_revision(&target_identity(target)));
                v
            })
            .collect::<Vec<_>>();
        let mut runs = model.runs.values().cloned().collect::<Vec<_>>();
        runs.sort_by_key(|r| r.created_at);
        let total = runs.len();
        if total > 200 {
            runs.drain(..total - 200);
        }
        let mut capabilities = model.artifact_capabilities.clone();
        if model.revision.is_none() {
            capabilities.push("workspace.import".into());
        }
        Ok(
            json!({"workspace":model.workspace,"files":files,"targets":targets,"runs":runs,"runs_total":total,"artifact":model.artifact,"startup":model.startup,"capabilities":capabilities,"last_error":model.last_error}),
        )
    }
    pub fn publish(&self) {
        if let Ok(state) = self.state() {
            let _ = self.events.send(json!({"type":"state","state":state}));
        }
    }
    pub fn log(&self, level: &str, message: &str, origin: &str) {
        let _ = self
            .events
            .send(json!({"type":"log","level":level,"message":message,"origin":origin}));
    }
    pub fn read(&self, path: &str) -> Result<Value> {
        if path.starts_with("application:") || path.starts_with("injectra:") {
            let model = lock(&self.model)?;
            let f = model.readonly.get(path).ok_or_else(|| {
                ApiError::new(
                    404,
                    "not_found",
                    "Source is not registered for this artifact",
                    "NotExecuted",
                )
            })?;
            let content = f.content.as_ref().ok_or_else(|| {
                ApiError::unsupported("This source is metadata-only or is not readable text")
            })?;
            let mut result = serde_json::to_value(f)?;
            result["content"] = json!(content);
            return Ok(result);
        }
        let relative = path.strip_prefix("user:").unwrap_or(path);
        let sources = lock(&self.sources)?;
        let w = sources
            .workspace
            .as_ref()
            .ok_or_else(|| ApiError::conflict("Workspace not initialized"))?;
        let file = w
            .files()?
            .into_iter()
            .find(|f| f.path == relative)
            .ok_or_else(|| {
                ApiError::new(
                    404,
                    "not_found",
                    "Workspace source not found",
                    "NotExecuted",
                )
            })?;
        let content = w.read(relative)?;
        let mut result = serde_json::to_value(file)?;
        result["path"] = json!(format!("user:{relative}"));
        result["relative_path"] = json!(relative);
        result["hash"] = json!(injectra_workspace::content_hash(content.as_bytes()));
        result["content"] = json!(content);
        Ok(result)
    }
    pub fn save(&self, path: &str, content: &str, expected: Option<&str>) -> Result<Value> {
        if path.starts_with("application:") || path.starts_with("injectra:") {
            return Err(ApiError::new(
                403,
                "readonly",
                "Only user increments can be edited",
                "NotExecuted",
            ));
        }
        let relative = path.strip_prefix("user:").unwrap_or(path);
        let saved = {
            let sources = lock(&self.sources)?;
            let w = sources
                .workspace
                .as_ref()
                .ok_or_else(|| ApiError::conflict("Workspace not initialized"))?;
            match w.save_if_match(relative, content, expected)? {
                SaveOutcome::Saved { file } => file,
                SaveOutcome::Conflict { current_hash } => {
                    return Err(ApiError::conflict(format!(
                        "Source changed externally; current hash: {}",
                        current_hash.as_deref().unwrap_or("absent")
                    )));
                }
            }
        };
        self.refresh_sources().map_err(|e| {
            ApiError::new(
                500,
                "snapshot",
                format!("File saved, but revision capture failed: {e}"),
                "Executed",
            )
        })?;
        self.publish();
        Ok(json!({"revision":lock(&self.model)?.revision,"file":saved}))
    }
    pub fn import(&self, manifest: &Path) -> Result<Value> {
        {
            let mut sources = lock(&self.sources)?;
            if sources.workspace.is_some() || fs::read_dir(&sources.root)?.next().is_some() {
                return Err(ApiError::conflict("Import requires a new empty workspace"));
            }
            sources.workspace = Some(Workspace::import_manifest(&sources.root, manifest)?);
        }
        self.refresh_sources().map_err(|e| {
            ApiError::new(
                500,
                "snapshot",
                format!("Import copied files, but revision capture failed: {e}"),
                "Executed",
            )
        })?;
        self.publish();
        self.state()
    }
    pub async fn connect(self: &Arc<Self>, spec: ConnectRequest) -> Result<Value> {
        let broker = self.broker.clone();
        let actor = tokio::task::spawn_blocking(move || BackendActor::connect(spec, broker))
            .await
            .map_err(|_| ApiError::unknown("Backend initialization task failed"))??;
        let targets = actor.targets().await?;
        let epoch = {
            let mut model = lock(&self.model)?;
            for target in model.targets.clone() {
                model.ledger.invalidate(&target_identity(&target))?;
            }
            Self::sync_runs(&mut model, &self.history)?;
            model.connection += 1;
            model.actor = Some(actor.clone());
            model.targets = targets;
            model.connection
        };
        self.publish();
        let state = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                if lock(&state.model).map_or(true, |m| m.connection != epoch) {
                    break;
                }
                match actor.targets().await {
                    Ok(targets) => {
                        let mut m = match lock(&state.model) {
                            Ok(m) => m,
                            Err(_) => break,
                        };
                        if m.connection != epoch {
                            break;
                        }
                        for old in m.targets.clone() {
                            if !targets.iter().any(|t| t.id == old.id) {
                                let _ = m.ledger.invalidate(&target_identity(&old));
                            }
                        }
                        m.targets = targets;
                        let _ = Self::sync_runs(&mut m, &state.history);
                        drop(m);
                        state.publish();
                    }
                    Err(error) => {
                        let mut m = match lock(&state.model) {
                            Ok(m) => m,
                            Err(_) => break,
                        };
                        if m.connection != epoch {
                            break;
                        }
                        for target in m.targets.clone() {
                            let _ = m.ledger.invalidate(&target_identity(&target));
                        }
                        m.targets.clear();
                        m.last_error = Some(error);
                        let _ = Self::sync_runs(&mut m, &state.history);
                        drop(m);
                        state.publish();
                    }
                }
            }
        });
        self.state()
    }
    fn sync_runs(model: &mut Model, history: &History) -> Result<()> {
        for record in model.ledger.records() {
            if let Some(run) = model.runs.get_mut(&record.request_id) {
                run.update(record);
                history.save(run)?;
            }
        }
        Ok(())
    }
    pub fn start_run(
        self: &Arc<Self>,
        target: &str,
        generation: Option<String>,
        revision: &str,
        acknowledge: Option<&str>,
    ) -> Result<Value> {
        let snapshot = self.snapshot()?;
        if snapshot.revision != revision {
            return Err(ApiError::conflict(
                "Workspace revision changed; refresh before running",
            ));
        }
        snapshot.verify()?;
        let program = snapshot.mapped_program()?;
        let (actor, identity, record) = {
            let mut m = lock(&self.model)?;
            if m.runs.len() >= crate::history::MAX_RUNS {
                return Err(ApiError::conflict(
                    "Execution history capacity reached; no records were deleted",
                ));
            }
            let view = m
                .targets
                .iter()
                .find(|t| t.id == target)
                .cloned()
                .ok_or_else(ApiError::stale)?;
            if generation.as_ref().is_some_and(|g| g != &view.generation) {
                return Err(ApiError::stale());
            }
            if !view.capabilities.iter().any(|c| c == "web.page.evaluate") {
                return Err(ApiError::unsupported("Page execution is unavailable"));
            }
            let identity = target_identity(&view);
            if let Some(request) = acknowledge {
                m.ledger
                    .acknowledge_unknown(&identity, request)
                    .map_err(|e| ApiError::conflict(e.to_string()))?;
            }
            let actor = m.actor.clone().ok_or_else(ApiError::stale)?;
            let record = m
                .ledger
                .begin(identity.clone(), revision)
                .map_err(|e| ApiError::conflict(e.to_string()))?;
            let run = Run::new(&record, program.mappings.clone(), snapshot.root.clone());
            self.history.save(&run)?;
            m.runs.insert(run.request_id.clone(), run);
            (actor, identity, record)
        };
        let id = record.request_id.clone();
        let state = self.clone();
        let revision = revision.to_owned();
        let target = target.to_owned();
        tokio::spawn(async move {
            let pending = actor.submit(target, generation, revision.clone(), program.source);
            let result = match pending {
                Ok(pending) => {
                    if let Ok(mut m) = lock(&state.model) {
                        let _ = m.ledger.mark_submitted(&id, &identity);
                        let _ = Self::sync_runs(&mut m, &state.history);
                    }
                    let _ = pending.started.await;
                    if let Ok(mut m) = lock(&state.model)
                        && let Some(run) = m.runs.get_mut(&id)
                        && run.status == "queued"
                    {
                        run.status = "running".into();
                        let _ = state.history.save(run);
                        let _ = state.events.send(json!({"type":"run","run":run}));
                    }
                    pending.finished.await.unwrap_or_else(|_| {
                        Err(ApiError::unknown(
                            "Backend worker ended without execution receipt",
                        ))
                    })
                }
                Err(error) => Err(error),
            };
            state.finish_run(&id, &identity, &revision, result);
        });
        let response = json!({"request_id":record.request_id});
        self.publish();
        Ok(response)
    }
    fn finish_run(
        &self,
        id: &str,
        target: &injectra_workspace::ExecutionTarget,
        revision: &str,
        result: Result<Value>,
    ) {
        let error = result.as_ref().err().cloned();
        let outcome = match result {
            Ok(value) => ExecutionOutcome {
                certainty: ExecutionCertainty::Executed,
                result: Some(value),
                error: None,
            },
            Err(ref e) => ExecutionOutcome {
                certainty: match e.certainty.as_str() {
                    "NotExecuted" => ExecutionCertainty::NotExecuted,
                    "Executed" => ExecutionCertainty::Executed,
                    _ => ExecutionCertainty::Unknown,
                },
                result: None,
                error: Some(e.message.clone()),
            },
        };
        if let Ok(mut m) = lock(&self.model) {
            match m.ledger.complete(id, target, revision, outcome) {
                Ok(record) => {
                    if let Some(run) = m.runs.get_mut(id) {
                        run.error = error.clone();
                        run.update(&record);
                        let _ = self.history.save(run);
                        let _ = self.events.send(json!({"type":"run","run":run}));
                        if let Some(error) = error {
                            let origin = attribution(&error.message, &run.source_mappings)
                                .unwrap_or(json!({"origin":"unknown"}));
                            let _=self.events.send(json!({"type":"log","level":"error","message":error.message,"request_id":id,"target":run.target,"revision":revision,"source":origin}));
                        }
                    }
                }
                Err(_) => {
                    let _ = Self::sync_runs(&mut m, &self.history);
                }
            }
        }
        self.publish();
    }
    pub async fn reload(&self, target: String, generation: Option<String>) -> Result<Value> {
        let actor = {
            let m = lock(&self.model)?;
            let view = m
                .targets
                .iter()
                .find(|v| v.id == target)
                .ok_or_else(ApiError::stale)?;
            if !view.capabilities.iter().any(|c| c == "web.page.reload") {
                return Err(ApiError::unsupported("Reload is unavailable"));
            }
            m.actor.clone().ok_or_else(ApiError::stale)?
        };
        actor.reload(target, generation).await
    }
    pub fn observe_bootstrap(&self, after: u64) -> u64 {
        let mut sequence = after;
        for event in self.broker.events(after) {
            sequence = sequence.max(event.sequence);
            if event.kind == "startup" {
                if let Ok(mut m) = lock(&self.model) {
                    let status = event
                        .data
                        .get("status")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    m.startup.status = status.into();
                    m.startup.artifact = Some(event.artifact.clone());
                    m.startup.instance = Some(event.instance.clone());
                    m.startup.error = event
                        .data
                        .get("error")
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                    m.startup.applied_revision = if status == "executed" {
                        event
                            .data
                            .get("revision")
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                    } else {
                        None
                    };
                    Self::startup_state(&mut m);
                }
                self.publish();
            } else if event.kind == "log" {
                let _=self.events.send(json!({"type":"log","artifact":event.artifact,"instance":event.instance,"page":event.page,"generation":event.generation,"log":event.data}));
            }
        }
        sequence
    }
}
