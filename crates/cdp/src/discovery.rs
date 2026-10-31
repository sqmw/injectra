use injectra_core::{Result, Target};
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
    pub fn target(&self) -> Target {
        Target {
            id: self.id.clone(),
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
        .call()?;
    if response.status() != 200 {
        return Err("debugger discovery did not return HTTP 200".into());
    }
    let body = response
        .body_mut()
        .with_config()
        .limit(1024 * 1024)
        .read_to_string()?;
    let pages: Vec<Page> = serde_json::from_str(&body)?;
    Ok(pages
        .into_iter()
        .filter(|page| page.kind == "page" && page.web_socket_debugger_url.is_some())
        .collect())
}
