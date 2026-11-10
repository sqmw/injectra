//! WKWebView page adapter through the native main-thread execution API.
use injectra_core::{BackendId, Error, ErrorKind, Result, TargetRef};
use injectra_frida::Bridge;
use injectra_web::{PAGE_EVALUATE, PageContext, WebBackend, WebSession, WebTarget};
use serde_json::{Value, json};
use std::path::Path;
pub struct WebkitBackend {
    id: BackendId,
    bridge: Bridge,
}
impl WebkitBackend {
    pub fn new(session: &Path) -> Result<Self> {
        Ok(Self {
            id: BackendId::fresh(),
            bridge: Bridge::new(
                session,
                vec![
                    ("route.py", include_str!("../helper/route.py")),
                    ("agent.js", include_str!("../helper/agent.js")),
                    ("webkit_agent.js", include_str!("../helper/webkit_agent.js")),
                    ("macos_appkit.js", include_str!("../helper/macos_appkit.js")),
                ],
            )?,
        })
    }
}
impl WebBackend for WebkitBackend {
    fn id(&self) -> &BackendId {
        &self.id
    }
    fn targets(&self) -> Result<Vec<WebTarget>> {
        let value = self.bridge.request("targets", json!({}))?;
        value
            .as_array()
            .ok_or("Invalid page list")?
            .iter()
            .map(|v| {
                Ok(WebTarget {
                    reference: TargetRef::new(&self.id, v["id"].as_str().ok_or("Missing page id")?),
                    title: v["title"].as_str().ok_or("Missing page title")?.into(),
                    url: v["url"].as_str().ok_or("Missing page URL")?.into(),
                    context: PageContext::MainFrame,
                })
            })
            .collect()
    }
    fn connect(&self, target: &TargetRef) -> Result<Box<dyn WebSession>> {
        target.ensure_backend(&self.id)?;
        if !self.targets()?.iter().any(|page| &page.reference == target) {
            return Err(Error::prepare(
                ErrorKind::TargetNotFound,
                "target must match exactly one current WKWebView page; run targets again",
            ));
        }
        Ok(Box::new(PageSession {
            target: target.clone(),
            bridge: self.bridge.clone(),
        }))
    }
}
struct PageSession {
    target: TargetRef,
    bridge: Bridge,
}
impl WebSession for PageSession {
    fn target(&self) -> &TargetRef {
        &self.target
    }
    fn capabilities(&self) -> &[injectra_core::Capability] {
        &[PAGE_EVALUATE]
    }
    fn evaluate_page(&mut self, source: &str) -> Result<Value> {
        self.bridge.request(
            "evaluate",
            json!({"target":self.target.key(),"source":source}),
        )
    }
}
