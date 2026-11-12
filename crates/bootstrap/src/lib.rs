//! Authenticated target-side Bootstrap bridge. No application-brand or UI assumptions.
mod auth;
mod broker;
mod model;
mod routes;
pub use auth::{proof, random_token};
pub use broker::Broker;
pub use model::*;
pub use routes::router;

use injectra_core::{BackendId, Capability, Error, ErrorKind, Result, TargetRef};
use injectra_web::{PAGE_EVALUATE, PageContext, WebBackend, WebSession, WebTarget};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
pub const PAGE_RELOAD: Capability = Capability("web.page.reload");
pub const PAGE_LOGS: Capability = Capability("web.page.logs");
const CAPABILITIES: &[Capability] = &[PAGE_EVALUATE, PAGE_RELOAD, PAGE_LOGS];
pub const EVALUATION_SOURCE: &str = include_str!("../helper/evaluation.cjs");
pub const BOOTSTRAP_SOURCE: &str = include_str!("../helper/bootstrap.cjs");

pub struct BootstrapBackend {
    id: BackendId,
    broker: Arc<Broker>,
    timeout: Duration,
}
impl BootstrapBackend {
    pub fn new(broker: Arc<Broker>) -> Self {
        Self {
            id: BackendId::fresh(),
            broker,
            timeout: Duration::from_secs(15),
        }
    }
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    pub fn broker(&self) -> &Arc<Broker> {
        &self.broker
    }
    pub fn evaluate_revision(
        &self,
        target: &TargetRef,
        revision: &str,
        source: &str,
    ) -> Result<Value> {
        target.ensure_backend(&self.id)?;
        self.broker
            .submit(target.key(), revision, source, self.timeout)
    }
    pub fn reload(&self, target: &TargetRef) -> Result<Value> {
        target.ensure_backend(&self.id)?;
        self.broker.reload(target.key(), self.timeout)
    }
}
impl WebBackend for BootstrapBackend {
    fn id(&self) -> &BackendId {
        &self.id
    }
    fn targets(&self) -> Result<Vec<WebTarget>> {
        Ok(self
            .broker
            .targets()
            .into_iter()
            .map(|t| WebTarget {
                reference: TargetRef::new(&self.id, t.id),
                title: t.title,
                url: t.url,
                context: PageContext::MainFrame,
            })
            .collect())
    }
    fn connect(&self, target: &TargetRef) -> Result<Box<dyn WebSession>> {
        target.ensure_backend(&self.id)?;
        if !self.broker.targets().iter().any(|t| t.id == target.key()) {
            return Err(Error::prepare(
                ErrorKind::TargetNotFound,
                "Bootstrap target is stale or disconnected",
            ));
        }
        Ok(Box::new(Session {
            target: target.clone(),
            broker: self.broker.clone(),
            timeout: self.timeout,
        }))
    }
}
struct Session {
    target: TargetRef,
    broker: Arc<Broker>,
    timeout: Duration,
}
impl WebSession for Session {
    fn target(&self) -> &TargetRef {
        &self.target
    }
    fn capabilities(&self) -> &[Capability] {
        CAPABILITIES
    }
    fn evaluate_page(&mut self, source: &str) -> Result<Value> {
        self.broker
            .submit(self.target.key(), "adhoc", source, self.timeout)
    }
}
