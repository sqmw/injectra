use crate::{
    actor::ConnectRequest,
    artifacts::{ExportRequest, PrepareRequest},
    error::{ApiError, Result},
    security::{ApiJson, authorize, not_found},
    state::StudioState,
};
use axum::{
    Json, Router,
    extract::{
        DefaultBodyLimit, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tower_http::services::ServeFile;

#[derive(Deserialize)]
struct FileQuery {
    path: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveRequest {
    path: String,
    content: String,
    expected_hash: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportRequest {
    manifest: PathBuf,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunRequest {
    target: String,
    revision: String,
    generation: Option<String>,
    acknowledge_unknown: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetRequest {
    target: String,
    generation: Option<String>,
}

pub fn router(state: Arc<StudioState>, assets: Option<&Path>) -> Result<Router> {
    let api = Router::new()
        .route("/state", get(get_state))
        .route("/files", get(get_file).put(save_file))
        .route("/import", post(import))
        .route("/connect", post(connect))
        .route("/run", post(run))
        .route("/prepare", post(prepare))
        .route("/dev", post(dev))
        .route("/restart", post(restart))
        .route("/reload", post(reload))
        .route("/export", post(export))
        .fallback(not_found)
        .layer(DefaultBodyLimit::max(
            injectra_workspace::MAX_FILE_BYTES * 8,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.auth.clone(),
            authorize,
        ));
    let mut app = Router::new()
        .nest("/api", api)
        .route("/api/events", get(events))
        .with_state(state.clone())
        .merge(injectra_bootstrap::router(state.broker.clone()));
    if let Some(assets) = assets {
        let root = assets.canonicalize()?;
        for (route, name) in [
            ("/", "index.html"),
            ("/index.html", "index.html"),
            ("/app.js", "app.js"),
            ("/app.css", "app.css"),
        ] {
            let file = root.join(name);
            let resolved = file.canonicalize()?;
            if !resolved.starts_with(&root) || !resolved.is_file() {
                return Err(ApiError::invalid(
                    "UI asset escapes compiled asset directory",
                ));
            }
            app = app.route_service(route, ServeFile::new(resolved));
        }
    } else {
        app=app.route("/",get(||async{(StatusCode::SERVICE_UNAVAILABLE,"Studio UI assets are unavailable; configure --assets or INJECTRA_STUDIO_ASSETS")}));
    }
    let auth = state.auth.clone();
    Ok(app.fallback(not_found).layer(middleware::from_fn(move|request:axum::extract::Request,next:axum::middleware::Next|{
  let auth=auth.clone();async move{
   if !auth.host_matches(request.headers()){return ApiError::new(403,"origin","Host does not match Studio listener","NotExecuted").into_response();}
   let mut response=next.run(request).await;
   let h=response.headers_mut();h.insert("cache-control","no-store".parse().unwrap());h.insert("referrer-policy","no-referrer".parse().unwrap());h.insert("x-content-type-options","nosniff".parse().unwrap());h.insert("x-frame-options","DENY".parse().unwrap());
   h.insert("content-security-policy","default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'; img-src 'self' data:; object-src 'none'; base-uri 'none'; frame-ancestors 'none'".parse().unwrap());
   response
  }
 })))
}
async fn blocking<T: Send + 'static>(
    action: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T> {
    tokio::task::spawn_blocking(action)
        .await
        .map_err(|_| ApiError::unknown("Service operation panicked"))?
}
async fn get_state(State(state): State<Arc<StudioState>>) -> Result<Json<Value>> {
    let state2 = state.clone();
    blocking(move || {
        state2.refresh_sources()?;
        state2.state()
    })
    .await
    .map(Json)
}
async fn get_file(
    State(state): State<Arc<StudioState>>,
    query: std::result::Result<Query<FileQuery>, axum::extract::rejection::QueryRejection>,
) -> Result<Json<Value>> {
    let query = query.map_err(|e| ApiError::invalid(e.to_string()))?.0;
    blocking(move || match query.path {
        Some(path) => state.read(&path),
        None => Ok(json!({"files":state.state()?["files"]})),
    })
    .await
    .map(Json)
}
async fn save_file(
    State(state): State<Arc<StudioState>>,
    ApiJson(body): ApiJson<SaveRequest>,
) -> Result<Json<Value>> {
    blocking(move || state.save(&body.path, &body.content, body.expected_hash.as_deref()))
        .await
        .map(Json)
}
async fn import(
    State(state): State<Arc<StudioState>>,
    ApiJson(body): ApiJson<ImportRequest>,
) -> Result<Json<Value>> {
    blocking(move || state.import(&body.manifest))
        .await
        .map(Json)
}
async fn connect(
    State(state): State<Arc<StudioState>>,
    ApiJson(body): ApiJson<ConnectRequest>,
) -> Result<Json<Value>> {
    state.connect(body).await.map(Json)
}
async fn run(
    State(state): State<Arc<StudioState>>,
    ApiJson(body): ApiJson<RunRequest>,
) -> Result<(StatusCode, Json<Value>)> {
    let result = blocking(move || {
        state.start_run(
            &body.target,
            body.generation,
            &body.revision,
            body.acknowledge_unknown.as_deref(),
        )
    })
    .await?;
    Ok((StatusCode::ACCEPTED, Json(result)))
}
async fn prepare(
    State(state): State<Arc<StudioState>>,
    ApiJson(body): ApiJson<PrepareRequest>,
) -> Result<Json<Value>> {
    let port = state
        .artifacts
        .clone()
        .ok_or_else(|| ApiError::unsupported("Preparation is unavailable in this service"))?;
    state.log("info", "Preparing application artifact", "injectra");
    blocking(move || {
        let snapshot = state.snapshot()?;
        let result = port.prepare(body, &snapshot, &state.endpoint.origin())?;
        state.refresh_artifact()?;
        state.publish();
        Ok(result)
    })
    .await
    .map(Json)
}
async fn dev(State(state): State<Arc<StudioState>>) -> Result<Json<Value>> {
    let port = state
        .artifacts
        .clone()
        .ok_or_else(|| ApiError::unsupported("Development launch is unavailable"))?;
    blocking(move || {
        let snapshot = state.snapshot()?;
        let result = port.dev(&snapshot)?;
        state.refresh_artifact()?;
        state.publish();
        Ok(result)
    })
    .await
    .map(Json)
}
async fn restart(State(state): State<Arc<StudioState>>) -> Result<Json<Value>> {
    let port = state
        .artifacts
        .clone()
        .ok_or_else(|| ApiError::unsupported("Owned restart is unavailable"))?;
    blocking(move || {
        let snapshot = state.snapshot()?;
        let result = port.restart(&snapshot)?;
        state.refresh_artifact()?;
        state.publish();
        Ok(result)
    })
    .await
    .map(Json)
}
async fn reload(
    State(state): State<Arc<StudioState>>,
    ApiJson(body): ApiJson<TargetRequest>,
) -> Result<Json<Value>> {
    state.reload(body.target, body.generation).await.map(Json)
}
async fn export(
    State(state): State<Arc<StudioState>>,
    ApiJson(body): ApiJson<ExportRequest>,
) -> Result<Json<Value>> {
    let port = state
        .artifacts
        .clone()
        .ok_or_else(|| ApiError::unsupported("Export is unavailable"))?;
    blocking(move || {
        let snapshot = state.snapshot()?;
        port.export(body, &snapshot)
    })
    .await
    .map(Json)
}
async fn events(
    State(state): State<Arc<StudioState>>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    if !state.auth.origin_matches(&headers) {
        return ApiError::new(
            403,
            "origin",
            "WebSocket requires exact Studio Origin",
            "NotExecuted",
        )
        .into_response();
    }
    upgrade
        .max_message_size(4096)
        .on_upgrade(move |socket| event_socket(socket, state))
        .into_response()
}
async fn event_socket(mut socket: WebSocket, state: Arc<StudioState>) {
    let authenticated = match tokio::time::timeout(Duration::from_secs(5), socket.recv()).await {
        Ok(Some(Ok(Message::Text(text)))) => {
            serde_json::from_str::<Value>(&text).ok().is_some_and(|v| {
                v.get("type").and_then(Value::as_str) == Some("authenticate")
                    && v.get("token")
                        .and_then(Value::as_str)
                        .is_some_and(|token| state.auth.token_matches(token))
            })
        }
        _ => false,
    };
    if !authenticated {
        let _ = socket.send(Message::Close(None)).await;
        return;
    }
    let mut events = state.events.subscribe();
    if socket
        .send(Message::Text(
            json!({"type":"authenticated"}).to_string().into(),
        ))
        .await
        .is_err()
    {
        return;
    }
    if let Ok(value) = state.state()
        && socket
            .send(Message::Text(
                json!({"type":"state","state":value}).to_string().into(),
            ))
            .await
            .is_err()
    {
        return;
    }
    loop {
        tokio::select! {
         incoming=socket.recv()=>match incoming{Some(Ok(Message::Ping(bytes)))=>{if socket.send(Message::Pong(bytes)).await.is_err(){return;}},Some(Ok(Message::Pong(_)))=>{},_=>return},
         event=events.recv()=>{
          let value=match event {Ok(v)=>v,Err(tokio::sync::broadcast::error::RecvError::Lagged(_))=>match state.state(){Ok(v)=>json!({"type":"state","state":v}),Err(_)=>return},Err(_)=>return};
          if socket.send(Message::Text(value.to_string().into())).await.is_err(){return;}
         }
        }
    }
}
