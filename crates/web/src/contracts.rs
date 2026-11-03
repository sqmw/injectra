use injectra_core::{BackendId, Capability, Result, TargetRef};
use serde_json::Value;

pub const PAGE_EVALUATE: Capability = Capability("web.page.evaluate");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageContext {
    MainFrame,
}

#[derive(Debug, Clone)]
pub struct WebTarget {
    pub reference: TargetRef,
    pub title: String,
    pub url: String,
    pub context: PageContext,
}

/// Connected page context; not a generic JavaScript or native agent environment.
/// Evaluation awaits promises and returns JSON by value. Dropping is not rollback.
pub trait WebSession {
    fn target(&self) -> &TargetRef;
    fn capabilities(&self) -> &[Capability];
    fn evaluate_page(&mut self, source: &str) -> Result<Value>;
}

pub trait WebBackend {
    fn id(&self) -> &BackendId;
    fn targets(&self) -> Result<Vec<WebTarget>>;
    fn connect(&self, target: &TargetRef) -> Result<Box<dyn WebSession>>;
}
