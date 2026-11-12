use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use futures_util::{SinkExt, StreamExt};
use injectra_bootstrap::Broker;
use injectra_studio::{Studio, StudioOptions};
use injectra_workspace::Workspace;
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tower::ServiceExt;

fn service(root: &std::path::Path) -> Studio {
    let workspace = root.join("workspace");
    Workspace::create(&workspace, "studio-test").unwrap();
    Studio::new(
        StudioOptions {
            workspace,
            cache: root.join("cache"),
            assets: None,
        },
        None,
        Arc::new(Broker::new()),
    )
    .unwrap()
}
async fn request(
    service: &Studio,
    method: &str,
    path: &str,
    body: Value,
    token: bool,
    origin: &str,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("host", &service.state.auth.host)
        .header("content-type", "application/json");
    if token {
        builder = builder.header(
            "authorization",
            format!("Bearer {}", service.state.endpoint.token),
        );
    }
    if !origin.is_empty() {
        builder = builder.header("origin", origin);
    }
    let response = service
        .router
        .clone()
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 16 * 1024 * 1024)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}
#[tokio::test]
async fn auth_conflict_creation_and_capability_boundary() {
    let temp = tempfile::tempdir().unwrap();
    let service = service(temp.path());
    let origin = service.state.endpoint.origin();
    let (status, _) = request(&service, "GET", "/api/state", Value::Null, false, "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = request(
        &service,
        "PUT",
        "/api/files",
        json!({"path":"user:page.js","content":"evil","expected_hash":null}),
        true,
        "http://foreign.invalid",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, state) = request(&service, "GET", "/api/state", Value::Null, true, "").await;
    assert_eq!(status, StatusCode::OK);
    assert!(state["capabilities"].as_array().unwrap().is_empty());
    let file = state["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "user:page.js")
        .unwrap();
    std::fs::write(temp.path().join("workspace/page.js"), "external edit").unwrap();
    let (status, error) = request(
        &service,
        "PUT",
        "/api/files",
        json!({"path":"user:page.js","content":"overwrite","expected_hash":file["hash"]}),
        true,
        &origin,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["certainty"], "NotExecuted");
    assert_eq!(
        std::fs::read_to_string(temp.path().join("workspace/page.js")).unwrap(),
        "external edit"
    );
    let (status, _) = request(
        &service,
        "PUT",
        "/api/files",
        json!({"path":"user:startup.cjs","content":"module.exports = {};","expected_hash":null}),
        true,
        &origin,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = request(
        &service,
        "PUT",
        "/api/files",
        json!({"path":"user:startup.cjs","content":"overwrite","expected_hash":null}),
        true,
        &origin,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = request(
        &service,
        "PUT",
        "/api/files",
        json!({"path":"application:main.js","content":"overwrite","expected_hash":null}),
        true,
        &origin,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = request(
        &service,
        "GET",
        "/api/files?path=injectra:config.json",
        Value::Null,
        true,
        "",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = request(
        &service,
        "POST",
        "/api/prepare",
        json!({"app":"fixture.app"}),
        true,
        &origin,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = request(
        &service,
        "POST",
        "/api/import",
        json!({"manifest":"anything"}),
        true,
        &origin,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}
#[tokio::test]
async fn websocket_authentication_is_first_frame_and_origin_bound() {
    use tokio_tungstenite::{
        connect_async,
        tungstenite::{Message, client::IntoClientRequest},
    };
    let temp = tempfile::tempdir().unwrap();
    let service = service(temp.path());
    let url = format!("ws://{}/api/events", service.state.auth.host);
    let origin = service.state.endpoint.origin();
    let token = service.state.endpoint.token.clone();
    let (stop, shutdown) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(service.serve(async {
        let _ = shutdown.await;
    }));
    let req = url.clone().into_client_request().unwrap();
    assert!(connect_async(req).await.is_err());
    let mut req = url.clone().into_client_request().unwrap();
    req.headers_mut().insert("origin", origin.parse().unwrap());
    let (mut ws, _) = connect_async(req.clone()).await.unwrap();
    ws.send(Message::Text(
        json!({"type":"authenticate","token":"wrong"})
            .to_string()
            .into(),
    ))
    .await
    .unwrap();
    assert!(matches!(ws.next().await, Some(Ok(Message::Close(_)))));
    let (mut ws, _) = connect_async(req.clone()).await.unwrap();
    ws.send(Message::Text(
        json!({"type":"authenticate","token":token})
            .to_string()
            .into(),
    ))
    .await
    .unwrap();
    let ack = ws.next().await.unwrap().unwrap().into_text().unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&ack).unwrap()["type"],
        "authenticated"
    );
    let state = ws.next().await.unwrap().unwrap().into_text().unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&state).unwrap()["type"],
        "state"
    );
    ws.close(None).await.unwrap();
    drop(ws);
    let (mut ws, _) = connect_async(req).await.unwrap();
    let closed = tokio::time::timeout(Duration::from_secs(7), ws.next())
        .await
        .unwrap();
    assert!(matches!(closed, Some(Ok(Message::Close(_)))));
    drop(ws);
    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}
#[test]
fn unfinished_execution_is_persisted_as_unknown_without_replay() {
    use injectra_studio::{history::History, model::Run};
    use injectra_workspace::{ExecutionLedger, ExecutionTarget};
    let temp = tempfile::tempdir().unwrap();
    let history = History::new(temp.path().join("runs"));
    assert!(history.load().unwrap().is_empty());
    let target = ExecutionTarget {
        backend_instance: "old-backend".into(),
        target: "old-target".into(),
        generation: "old-document".into(),
    };
    let record = ExecutionLedger::default()
        .begin(target, "revision")
        .unwrap();
    let run = Run::new(&record, vec![], temp.path().join("snapshot"));
    history.save(&run).unwrap();
    let loaded = history.load().unwrap();
    assert_eq!(loaded[0].status, "unknown");
    assert_eq!(loaded[0].certainty, "Unknown");
    assert_eq!(history.load().unwrap()[0].status, "unknown");
}
