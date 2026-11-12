use crate::error::{ApiError, Result};
use axum::{
    Json,
    extract::{FromRequest, Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use subtle::ConstantTimeEq;

pub fn random_id() -> Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|e| ApiError::invalid(format!("Random source unavailable: {e}")))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
#[derive(Clone)]
pub struct BrowserAuth {
    token: String,
    pub origin: String,
    pub host: String,
}
impl BrowserAuth {
    pub fn new(token: String, origin: String) -> Result<Self> {
        let host = origin
            .strip_prefix("http://127.0.0.1:")
            .and_then(|p| p.parse::<u16>().ok())
            .filter(|p| *p > 0)
            .map(|p| format!("127.0.0.1:{p}"))
            .ok_or_else(|| ApiError::invalid("Studio requires an exact loopback origin"))?;
        if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(ApiError::invalid("Invalid browser credential format"));
        }
        Ok(Self {
            token,
            origin,
            host,
        })
    }
    pub fn token_matches(&self, supplied: &str) -> bool {
        bool::from(self.token.as_bytes().ct_eq(supplied.as_bytes()))
    }
    pub fn origin_matches(&self, headers: &HeaderMap) -> bool {
        headers.get("origin").and_then(|h| h.to_str().ok()) == Some(self.origin.as_str())
    }
    pub fn host_matches(&self, headers: &HeaderMap) -> bool {
        headers.get("host").and_then(|h| h.to_str().ok()) == Some(self.host.as_str())
    }
    pub fn check(&self, headers: &HeaderMap, method: &Method) -> Result<()> {
        if !self.host_matches(headers) {
            return Err(ApiError::new(
                403,
                "origin",
                "Host does not match this Studio instance",
                "NotExecuted",
            ));
        }
        if !matches!(*method, Method::GET | Method::HEAD) && !self.origin_matches(headers) {
            return Err(ApiError::new(
                403,
                "origin",
                "Writes require the exact Studio Origin",
                "NotExecuted",
            ));
        }
        let supplied = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));
        if !supplied.is_some_and(|value| self.token_matches(value)) {
            return Err(ApiError::new(
                401,
                "unauthorized",
                "Browser authentication required",
                "NotExecuted",
            ));
        }
        Ok(())
    }
}
pub async fn authorize(
    State(auth): State<Arc<BrowserAuth>>,
    request: Request,
    next: Next,
) -> Response {
    match auth.check(request.headers(), request.method()) {
        Ok(()) => next.run(request).await,
        Err(e) => e.into_response(),
    }
}
pub struct ApiJson<T>(pub T);
impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;
    async fn from_request(
        request: Request,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        Json::<T>::from_request(request, state)
            .await
            .map(|Json(value)| Self(value))
            .map_err(|e| {
                ApiError::new(
                    e.status().as_u16(),
                    "invalid_input",
                    e.body_text(),
                    "NotExecuted",
                )
            })
    }
}
pub async fn not_found() -> Response {
    ApiError::new(
        StatusCode::NOT_FOUND.as_u16(),
        "not_found",
        "Unknown API endpoint",
        "NotExecuted",
    )
    .into_response()
}
