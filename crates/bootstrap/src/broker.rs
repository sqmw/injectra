use crate::{
    AgentLog, Challenge, Command, Event, Hello, Page, Receipt, Register, Registered, Report,
    TargetStatus, auth,
};
use injectra_core::{Certainty, Error, ErrorKind, Phase, Result};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Mutex, mpsc};
use std::time::{Duration, Instant};

const LEASE: Duration = Duration::from_secs(30);
struct PendingChallenge {
    hello: Hello,
    nonce: String,
    at: Instant,
}
struct Connection {
    public_id: String,
    history: VecDeque<(String, String, String, String)>,
    artifact: String,
    instance: String,
    pid: u32,
    pages: HashMap<String, Page>,
    applied: HashMap<String, String>,
    queue: VecDeque<Command>,
    last_seen: Instant,
}
struct Pending {
    session: String,
    command: Command,
    dispatched: bool,
    sender: mpsc::Sender<Result<Value>>,
}
#[derive(Default)]
struct Inner {
    keys: HashMap<String, String>,
    challenges: HashMap<String, PendingChallenge>,
    connections: HashMap<String, Connection>,
    pending: HashMap<String, Pending>,
    used_requests: HashSet<String>,
    events: VecDeque<Event>,
    sequence: u64,
}
#[derive(Default)]
pub struct Broker {
    inner: Mutex<Inner>,
}
fn invalid(message: &str) -> Error {
    Error::prepare(ErrorKind::InvalidInput, message)
}
fn lost(dispatched: bool, message: &str) -> Error {
    Error::new(
        ErrorKind::Connection,
        Phase::Submit,
        if dispatched {
            Certainty::Unknown
        } else {
            Certainty::NotExecuted
        },
        message,
    )
}
fn target_key(session: &str, page: &Page) -> String {
    format!("{session}:{}:{}", page.id, page.generation)
}
fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
}
impl Broker {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register_artifact(&self, artifact: &str, key: &str) -> Result<()> {
        if !token(artifact) || auth::decode(key).is_none() {
            return Err(invalid("Invalid artifact identity or key"));
        }
        let mut inner = self.inner.lock().unwrap();
        if let Some(old) = inner.keys.get(artifact)
            && !auth::verify(old, key)
        {
            return Err(invalid("Artifact key cannot change while registered"));
        }
        inner.keys.insert(artifact.to_string(), key.to_string());
        Ok(())
    }
    pub fn unregister_artifact(&self, artifact: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.keys.remove(artifact);
        let sessions = inner
            .connections
            .iter()
            .filter(|(_, c)| c.artifact == artifact)
            .map(|(s, _)| s.clone())
            .collect::<Vec<_>>();
        for s in sessions {
            invalidate(&mut inner, &s, "Artifact detached");
        }
    }
    pub fn challenge(&self, hello: Hello) -> Result<Challenge> {
        if !token(&hello.artifact)
            || !token(&hello.instance)
            || hello.pid == 0
            || auth::decode(&hello.client_nonce).is_none()
        {
            return Err(invalid("Malformed bridge identity"));
        }
        let mut i = self.inner.lock().unwrap();
        sweep(&mut i);
        if i.challenges.len() >= 128 {
            return Err(invalid("Too many pending bridge challenges"));
        }
        let key = i
            .keys
            .get(&hello.artifact)
            .ok_or_else(|| invalid("Unknown artifact"))?;
        let nonce = auth::random_token();
        let id = auth::random_token();
        let server_proof = auth::proof(
            key,
            "server",
            &hello.artifact,
            &hello.instance,
            hello.pid,
            &hello.client_nonce,
            &nonce,
        )
        .unwrap();
        i.challenges.insert(
            id.clone(),
            PendingChallenge {
                hello,
                nonce: nonce.clone(),
                at: Instant::now(),
            },
        );
        Ok(Challenge {
            challenge_id: id,
            server_nonce: nonce,
            server_proof,
        })
    }
    pub fn authenticate(&self, register: Register) -> Result<Registered> {
        let mut i = self.inner.lock().unwrap();
        sweep(&mut i);
        let c = i
            .challenges
            .remove(&register.challenge_id)
            .ok_or_else(|| invalid("Expired or consumed bridge challenge"))?;
        let key = i
            .keys
            .get(&c.hello.artifact)
            .ok_or_else(|| invalid("Unknown artifact"))?;
        let expected = auth::proof(
            key,
            "client",
            &c.hello.artifact,
            &c.hello.instance,
            c.hello.pid,
            &c.hello.client_nonce,
            &c.nonce,
        )
        .unwrap();
        if !auth::verify(&expected, &register.client_proof) {
            return Err(invalid("Invalid bridge proof"));
        }
        let old = i
            .connections
            .iter()
            .filter(|(_, v)| {
                v.artifact == c.hello.artifact
                    && (v.instance == c.hello.instance || v.pid == c.hello.pid)
            })
            .map(|(s, _)| s.clone())
            .collect::<Vec<_>>();
        for s in old {
            invalidate(
                &mut i,
                &s,
                "Bridge reconnected; previous session invalidated",
            );
        }
        let session = auth::random_token();
        let instance = c.hello.instance.clone();
        i.connections.insert(
            session.clone(),
            Connection {
                public_id: uuid::Uuid::new_v4().to_string(),
                history: VecDeque::new(),
                artifact: c.hello.artifact,
                instance: instance.clone(),
                pid: c.hello.pid,
                pages: HashMap::new(),
                applied: HashMap::new(),
                queue: VecDeque::new(),
                last_seen: Instant::now(),
            },
        );
        Ok(Registered { session, instance })
    }
    pub fn touch(&self, session: &str) -> Result<()> {
        let mut i = self.inner.lock().unwrap();
        sweep(&mut i);
        i.connections
            .get_mut(session)
            .ok_or_else(|| invalid("Unknown bridge session"))?
            .last_seen = Instant::now();
        Ok(())
    }
    pub fn invalidate_instance(&self, artifact: &str, instance: &str) {
        let mut i = self.inner.lock().unwrap();
        let ids = i
            .connections
            .iter()
            .filter(|(_, c)| c.artifact == artifact && c.instance == instance)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in ids {
            invalidate(&mut i, &id, "Owned process exited");
        }
    }
    pub fn poll(&self, session: &str) -> Result<Option<Command>> {
        let mut i = self.inner.lock().unwrap();
        sweep(&mut i);
        let c = i
            .connections
            .get_mut(session)
            .ok_or_else(|| invalid("Unknown bridge session"))?;
        let command = c.queue.pop_front();
        if let Some(ref command) = command {
            c.history.push_back((
                command.request_id.clone(),
                command.page.clone(),
                command.generation.clone(),
                command.revision.clone(),
            ));
            while c.history.len() > 256 {
                c.history.pop_front();
            }
        }
        if let Some(ref command) = command
            && let Some(p) = i.pending.get_mut(&command.request_id)
        {
            p.dispatched = true;
        }
        Ok(command)
    }
    pub fn report(&self, session: &str, report: Report) -> Result<()> {
        if report.pages.len() > 256 || report.receipts.len() > 64 || report.logs.len() > 100 {
            return Err(invalid("Bridge report exceeds bounds"));
        }
        let mut pages = HashMap::new();
        for p in report.pages {
            if !token(&p.id)
                || !token(&p.generation)
                || p.url.len() > 16384
                || p.title.len() > 4096
                || pages.insert(p.id.clone(), p).is_some()
            {
                return Err(invalid("Invalid or duplicate page identity"));
            }
        }
        let mut i = self.inner.lock().unwrap();
        sweep(&mut i);
        let c = i
            .connections
            .get_mut(session)
            .ok_or_else(|| invalid("Unknown bridge session"))?;
        c.last_seen = Instant::now();
        let expired = c
            .pages
            .iter()
            .filter(|(id, p)| {
                pages.get(*id).map(|n| n.generation.as_str()) != Some(p.generation.as_str())
            })
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in &expired {
            c.applied.remove(id);
        }
        c.pages = pages;
        let artifact = c.artifact.clone();
        let instance = c.instance.clone();
        // A receipt may confirm execution that completed just before navigation. Match
        // the original command before invalidating unanswered requests for old pages.
        for receipt in report.receipts {
            complete(&mut i, session, receipt)?;
        }
        let obsolete = i
            .pending
            .iter()
            .filter(|(_, p)| p.session == session && expired.contains(&p.command.page))
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in obsolete {
            if let Some(p) = i.pending.remove(&id) {
                let _ = p.sender.send(Err(lost(
                    p.dispatched,
                    "Page generation changed before a confirmed receipt",
                )));
            }
        }
        if let Some(c) = i.connections.get_mut(session) {
            c.queue.retain(|q| !expired.contains(&q.page));
        }
        for log in report.logs {
            record_log(&mut i, session, &artifact, &instance, log);
        }

        Ok(())
    }
    pub fn disconnect(&self, session: &str) {
        invalidate(
            &mut self.inner.lock().unwrap(),
            session,
            "Bridge disconnected",
        );
    }
    pub fn targets(&self) -> Vec<TargetStatus> {
        let mut i = self.inner.lock().unwrap();
        sweep(&mut i);
        i.connections
            .iter()
            .flat_map(|(_, c)| {
                c.pages.values().map(|p| TargetStatus {
                    id: target_key(&c.public_id, p),
                    artifact: c.artifact.clone(),
                    instance: c.instance.clone(),
                    pid: c.pid,
                    page: p.id.clone(),
                    generation: p.generation.clone(),
                    title: p.title.clone(),
                    url: p.url.clone(),
                    applied_revision: c.applied.get(&p.id).cloned(),
                    capabilities: vec![
                        "web.page.evaluate".into(),
                        "web.page.reload".into(),
                        "web.page.logs".into(),
                    ],
                })
            })
            .collect()
    }
    pub fn events(&self, after: u64) -> Vec<Event> {
        self.inner
            .lock()
            .unwrap()
            .events
            .iter()
            .filter(|e| e.sequence > after)
            .cloned()
            .collect()
    }
    pub fn submit(
        &self,
        target: &str,
        revision: &str,
        source: &str,
        timeout: Duration,
    ) -> Result<Value> {
        self.dispatch(target, revision, "evaluate", source, timeout, None)
    }
    pub fn submit_with_request_id(
        &self,
        target: &str,
        revision: &str,
        source: &str,
        request_id: &str,
        timeout: Duration,
    ) -> Result<Value> {
        self.dispatch(
            target,
            revision,
            "evaluate",
            source,
            timeout,
            Some(request_id),
        )
    }
    pub fn reload(&self, target: &str, timeout: Duration) -> Result<Value> {
        self.dispatch(target, "", "reload", "", timeout, None)
    }
    fn dispatch(
        &self,
        target: &str,
        revision: &str,
        operation: &str,
        source: &str,
        timeout: Duration,
        requested_id: Option<&str>,
    ) -> Result<Value> {
        if source.len() > 4 * 1024 * 1024 || revision.len() > 256 {
            return Err(invalid("Execution input exceeds bounds"));
        }
        let (tx, rx) = mpsc::channel();
        let request = requested_id
            .map(String::from)
            .unwrap_or_else(auth::random_token);
        if !token(&request) {
            return Err(invalid("Invalid request identity"));
        }
        {
            let mut i = self.inner.lock().unwrap();
            sweep(&mut i);
            if i.used_requests.contains(&request) {
                return Err(invalid("Request identity was already used; not replayed"));
            }
            if i.used_requests.len() >= 100_000 {
                return Err(invalid("Controller request history limit reached"));
            }
            i.used_requests.insert(request.clone());
            let found = i
                .connections
                .iter()
                .find_map(|(s, c)| {
                    c.pages
                        .values()
                        .find(|p| target_key(&c.public_id, p) == target)
                        .map(|p| (s.clone(), p.clone()))
                })
                .ok_or_else(|| {
                    Error::prepare(ErrorKind::TargetNotFound, "Page is stale or disconnected")
                })?;
            if i.pending
                .values()
                .any(|p| p.session == found.0 && p.command.page == found.1.id)
            {
                return Err(invalid("An execution is already pending for this target"));
            }
            let command = Command {
                request_id: request.clone(),
                page: found.1.id,
                generation: found.1.generation,
                revision: revision.into(),
                operation: operation.into(),
                source: source.into(),
            };
            i.connections
                .get_mut(&found.0)
                .unwrap()
                .queue
                .push_back(command.clone());
            i.pending.insert(
                request.clone(),
                Pending {
                    session: found.0,
                    command,
                    dispatched: false,
                    sender: tx,
                },
            );
        }
        match rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(_) => {
                let mut i = self.inner.lock().unwrap();
                if let Some(p) = i.pending.remove(&request) {
                    if let Some(c) = i.connections.get_mut(&p.session) {
                        c.queue.retain(|q| q.request_id != request);
                        if p.dispatched {
                            c.applied.remove(&p.command.page);
                        }
                    }
                    return Err(Error::new(
                        ErrorKind::Timeout,
                        Phase::Submit,
                        if p.dispatched {
                            Certainty::Unknown
                        } else {
                            Certainty::NotExecuted
                        },
                        "Bootstrap execution deadline exceeded; not replayed",
                    ));
                }
                // Receipt removal and channel delivery race at the deadline: prefer a
                // delivered confirmation, otherwise the outcome remains unknown.
                rx.try_recv().unwrap_or_else(|_| {
                    Err(Error::new(
                        ErrorKind::Timeout,
                        Phase::Submit,
                        Certainty::Unknown,
                        "Bootstrap receipt raced deadline",
                    ))
                })
            }
        }
    }
}
fn complete(i: &mut Inner, session: &str, r: Receipt) -> Result<()> {
    let Some(p) = i.pending.get(&r.request_id) else {
        return Ok(());
    }; // duplicate/late receipt, never replay
    if p.session != session
        || p.command.page != r.page
        || p.command.generation != r.generation
        || p.command.revision != r.revision
    {
        return Err(invalid("Receipt identity/revision mismatch"));
    }
    if !p.dispatched {
        return Err(invalid("Receipt arrived before dispatch"));
    }
    if !["NotExecuted", "Executed", "Unknown"].contains(&r.certainty.as_str())
        || r.ok && r.certainty != "Executed"
    {
        return Err(invalid("Invalid execution certainty"));
    }
    let p = i.pending.remove(&r.request_id).unwrap();
    let c = i.connections.get_mut(session).unwrap();
    let artifact = c.artifact.clone();
    let instance = c.instance.clone();
    if !r.ok && r.certainty != "NotExecuted" {
        c.applied.remove(&r.page);
    }
    if r.ok
        && p.command.operation == "evaluate"
        && c.pages
            .get(&r.page)
            .is_some_and(|page| page.generation == r.generation)
    {
        c.applied.insert(r.page.clone(), r.revision.clone());
    }
    event(
        i,
        &artifact,
        &instance,
        Some(r.page.clone()),
        Some(r.generation.clone()),
        "receipt",
        json!(&r),
    );
    let result = if r.ok {
        Ok(r.value)
    } else {
        Err(Error::new(
            ErrorKind::Execution,
            Phase::Execute,
            match r.certainty.as_str() {
                "NotExecuted" => Certainty::NotExecuted,
                "Unknown" => Certainty::Unknown,
                _ => Certainty::Executed,
            },
            r.error.unwrap_or_else(|| "Target execution failed".into()),
        ))
    };
    let _ = p.sender.send(result);
    Ok(())
}
fn event(
    i: &mut Inner,
    artifact: &str,
    instance: &str,
    page: Option<String>,
    generation: Option<String>,
    kind: &str,
    data: Value,
) {
    i.sequence += 1;
    i.events.push_back(Event {
        sequence: i.sequence,
        artifact: artifact.into(),
        instance: instance.into(),
        page,
        generation,
        kind: kind.into(),
        data,
    });
    while i.events.len() > 1000 {
        i.events.pop_front();
    }
}
fn record_log(i: &mut Inner, session: &str, artifact: &str, instance: &str, mut log: AgentLog) {
    let Some(c) = i.connections.get(session) else {
        return;
    };
    if !c
        .history
        .iter()
        .any(|(request, page, generation, revision)| {
            request == &log.request_id
                && page == &log.page
                && generation == &log.generation
                && revision == &log.revision
        })
    {
        return;
    }
    log.message.truncate(log.message.floor_char_boundary(8192));
    event(
        i,
        artifact,
        instance,
        Some(log.page.clone()),
        Some(log.generation.clone()),
        "console",
        json!(log),
    );
}
fn invalidate(i: &mut Inner, session: &str, why: &str) {
    i.connections.remove(session);
    let ids = i
        .pending
        .iter()
        .filter(|(_, p)| p.session == session)
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    for id in ids {
        if let Some(p) = i.pending.remove(&id) {
            let _ = p.sender.send(Err(lost(p.dispatched, why)));
        }
    }
}
fn sweep(i: &mut Inner) {
    i.challenges.retain(|_, c| c.at.elapsed() < LEASE);
    let expired = i
        .connections
        .iter()
        .filter(|(_, c)| c.last_seen.elapsed() > LEASE)
        .map(|(s, _)| s.clone())
        .collect::<Vec<_>>();
    for s in expired {
        invalidate(i, &s, "Bridge heartbeat expired");
    }
}
