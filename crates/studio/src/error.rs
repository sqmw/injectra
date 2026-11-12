use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use injectra_core::{Certainty, Error, ErrorKind};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, ApiError>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub kind: String,
    pub message: String,
    pub certainty: String,
    #[serde(skip)]
    pub status: u16,
}
impl ApiError {
    pub fn new(status: u16, kind: &str, message: impl Into<String>, certainty: &str) -> Self {
        Self {
            status,
            kind: kind.into(),
            message: message.into(),
            certainty: certainty.into(),
        }
    }
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(400, "invalid_input", message, "NotExecuted")
    }
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(409, "conflict", message, "NotExecuted")
    }
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(422, "unsupported", message, "NotExecuted")
    }
    pub fn stale() -> Self {
        Self::new(
            409,
            "target_not_found",
            "Target generation changed; select a current target",
            "NotExecuted",
        )
    }
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::new(502, "connection", message, "Unknown")
    }
}
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.message, self.certainty)
    }
}
impl std::error::Error for ApiError {}
impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        Self::new(500, "io", e.to_string(), "NotExecuted")
    }
}
impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self::invalid(e.to_string())
    }
}
impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        let kind = match e.kind {
            ErrorKind::InvalidInput => "invalid_input",
            ErrorKind::Unsupported => "unsupported",
            ErrorKind::TargetNotFound => "target_not_found",
            ErrorKind::Io => "io",
            ErrorKind::Connection => "connection",
            ErrorKind::Protocol => "protocol",
            ErrorKind::Execution => "execution",
            ErrorKind::Timeout => "timeout",
            ErrorKind::ResultEncoding => "result_encoding",
        };
        let status = match e.kind {
            ErrorKind::InvalidInput => 400,
            ErrorKind::Unsupported => 422,
            ErrorKind::TargetNotFound => 409,
            _ => 502,
        };
        let certainty = match e.certainty {
            Certainty::NotExecuted => "NotExecuted",
            Certainty::Executed => "Executed",
            Certainty::Unknown => "Unknown",
        };
        Self::new(status, kind, e.to_string(), certainty)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (
            status,
            Json(
                serde_json::json!({"error":self,"message":self.message,"certainty":self.certainty}),
            ),
        )
            .into_response()
    }
}
