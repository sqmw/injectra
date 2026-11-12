use crate::{Broker, Hello, Register, Report};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde_json::{Value, json};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
type Failure = (StatusCode, Json<Value>);
fn failure(e: impl ToString) -> Failure {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error":e.to_string()})),
    )
}
fn session(h: &HeaderMap) -> std::result::Result<&str, Failure> {
    h.get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error":"Bridge authentication required"})),
            )
        })
}
fn no_browser(h: &HeaderMap) -> std::result::Result<(), Failure> {
    if h.contains_key("origin") {
        Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error":"Bridge is not a browser API"})),
        ))
    } else {
        Ok(())
    }
}
pub fn router(broker: Arc<Broker>) -> Router {
    Router::new()
        .route("/bridge/challenge", post(challenge))
        .route("/bridge/register", post(register))
        .route("/bridge/poll", get(poll))
        .route("/bridge/report", post(report))
        .route("/bridge/disconnect", post(disconnect))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024))
        .with_state(broker)
}
async fn challenge(
    State(b): State<Arc<Broker>>,
    headers: HeaderMap,
    Json(h): Json<Hello>,
) -> std::result::Result<Json<Value>, Failure> {
    no_browser(&headers)?;
    Ok(Json(
        serde_json::to_value(b.challenge(h).map_err(failure)?).unwrap(),
    ))
}
async fn register(
    State(b): State<Arc<Broker>>,
    headers: HeaderMap,
    Json(r): Json<Register>,
) -> std::result::Result<Json<Value>, Failure> {
    no_browser(&headers)?;
    Ok(Json(
        serde_json::to_value(b.authenticate(r).map_err(failure)?).unwrap(),
    ))
}
async fn poll(
    State(b): State<Arc<Broker>>,
    headers: HeaderMap,
) -> std::result::Result<Json<Value>, Failure> {
    no_browser(&headers)?;
    let token = session(&headers)?;
    let start = Instant::now();
    loop {
        if let Some(command) = b.poll(token).map_err(failure)? {
            return Ok(Json(json!({"command":command})));
        }
        if start.elapsed() > Duration::from_secs(15) {
            return Ok(Json(json!({"command":null})));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
async fn report(
    State(b): State<Arc<Broker>>,
    headers: HeaderMap,
    Json(r): Json<Report>,
) -> std::result::Result<Json<Value>, Failure> {
    no_browser(&headers)?;
    b.report(session(&headers)?, r).map_err(failure)?;
    Ok(Json(json!({"ok":true})))
}
async fn disconnect(
    State(b): State<Arc<Broker>>,
    headers: HeaderMap,
) -> std::result::Result<Json<Value>, Failure> {
    no_browser(&headers)?;
    b.disconnect(session(&headers)?);
    Ok(Json(json!({"ok":true})))
}
