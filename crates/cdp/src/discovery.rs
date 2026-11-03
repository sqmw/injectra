use injectra_core::{BackendId, Certainty, Error, ErrorKind, Phase, Result, TargetRef};
use injectra_web::{PageContext, WebTarget};
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Page {
    pub id: String,
    pub title: String,
    pub url: String,
    #[serde(rename = "type")]
    kind: String,
    pub web_socket_debugger_url: Option<String>,
}

impl Page {
    pub fn target(&self, backend: &BackendId) -> WebTarget {
        WebTarget {
            reference: TargetRef::new(backend, self.id.clone()),
            context: PageContext::MainFrame,
            title: self.title.clone(),
            url: self.url.clone(),
        }
    }
}

pub(crate) fn discover(port: u16) -> Result<Vec<Page>> {
    let config = ureq::Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(3)))
        .build();
    let agent: ureq::Agent = config.into();
    let mut response = agent
        .get(format!("http://127.0.0.1:{port}/json/list"))
        .call()
        .map_err(|e| failure(ErrorKind::Connection, e))?;
    if response.status() != 200 {
        return Err(Error::new(
            ErrorKind::Protocol,
            Phase::Discover,
            Certainty::NotExecuted,
            "debugger discovery did not return HTTP 200",
        ));
    }
    let body = response
        .body_mut()
        .with_config()
        .limit(1024 * 1024)
        .read_to_string()
        .map_err(|e| failure(ErrorKind::Protocol, e))?;
    let pages: Vec<Page> =
        serde_json::from_str(&body).map_err(|e| failure(ErrorKind::Protocol, e))?;
    Ok(pages
        .into_iter()
        .filter(|page| page.kind == "page" && page.web_socket_debugger_url.is_some())
        .collect())
}

fn failure<E: std::error::Error + Send + Sync + 'static>(kind: ErrorKind, e: E) -> Error {
    Error::new(kind, Phase::Discover, Certainty::NotExecuted, e.to_string()).caused_by(e)
}
