use crate::{
    artifacts::ArtifactPort,
    endpoint::{self, EndpointLease},
    error::{ApiError, Result},
    http,
    state::StudioState,
};
use injectra_bootstrap::Broker;
use serde_json::json;
use std::{path::PathBuf, sync::Arc, time::Duration};

pub struct StudioOptions {
    pub workspace: PathBuf,
    pub cache: PathBuf,
    pub assets: Option<PathBuf>,
}
pub struct Studio {
    pub state: Arc<StudioState>,
    pub router: axum::Router,
    endpoint: EndpointLease,
}
impl Studio {
    pub fn new(
        options: StudioOptions,
        artifacts: Option<Arc<dyn ArtifactPort>>,
        broker: Arc<Broker>,
    ) -> Result<Self> {
        std::fs::create_dir_all(&options.workspace)?;
        let endpoint = endpoint::bind(&options.cache, &options.workspace)?;
        let state = StudioState::new(
            &options.workspace,
            options.cache,
            endpoint.record.clone(),
            endpoint.directory.clone(),
            broker,
            artifacts,
        )?;
        let router = http::router(state.clone(), options.assets.as_deref())?;
        Ok(Self {
            state,
            router,
            endpoint,
        })
    }
    pub fn ready(&self) -> serde_json::Value {
        json!({"event":"studio-ready","origin":self.endpoint.record.origin(),"url":self.endpoint.record.launch_url(),"port":self.endpoint.record.port,"workspace":self.state.state().ok().and_then(|v|v.get("workspace").cloned()),"ready_file":self.endpoint.directory.join("ready.json")})
    }
    pub async fn serve(
        self,
        shutdown: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<()> {
        let listener = tokio::net::TcpListener::from_std(self.endpoint.listener.try_clone()?)?;
        let watcher_state = self.state.clone();
        let watcher = tokio::spawn(async move {
            let mut sequence = 0;
            let mut last_error = None;
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                sequence = watcher_state.observe_bootstrap(sequence);
                let target = watcher_state.clone();
                let result = tokio::task::spawn_blocking(move || target.refresh_sources()).await;
                match result {
                    Ok(Ok(changed)) => {
                        last_error = None;
                        if changed {
                            watcher_state.publish();
                        }
                    }
                    Ok(Err(e)) => {
                        if last_error.as_ref() != Some(&e.message) {
                            watcher_state.log("error", &e.message, "user");
                            last_error = Some(e.message);
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        let result = axum::serve(listener, self.router)
            .with_graceful_shutdown(shutdown)
            .await;
        watcher.abort();
        result.map_err(ApiError::from)
    }
}
pub fn client(
    workspace: &std::path::Path,
    cache: &std::path::Path,
    path: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value> {
    let record = endpoint::read_record(cache, workspace)?;
    let url = format!("{}/api/{path}", record.origin());
    let config = ureq::Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(300)))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let response = agent
        .post(&url)
        .header("Authorization", &format!("Bearer {}", record.token))
        .header("Origin", &record.origin())
        .send_json(body)
        .map_err(|e| {
            ApiError::unknown(format!(
                "Studio request has no receipt: {e}; do not automatically repeat"
            ))
        })?;
    let status = response.status().as_u16();
    let value: serde_json::Value = response
        .into_body()
        .read_json()
        .map_err(|e| ApiError::unknown(format!("Studio response could not be decoded: {e}")))?;
    if let Some(error) = value.get("error") {
        let mut error: ApiError = serde_json::from_value(error.clone())
            .map_err(|e| ApiError::unknown(format!("Invalid Studio error receipt: {e}")))?;
        error.status = status;
        return Err(error);
    }
    if !(200..300).contains(&status) {
        return Err(ApiError::unknown(format!(
            "Unexpected Studio status {status}"
        )));
    }
    Ok(value)
}
