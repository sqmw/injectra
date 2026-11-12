use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub id: String,
    pub generation: String,
    pub title: String,
    pub url: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hello {
    pub artifact: String,
    pub instance: String,
    pub pid: u32,
    pub client_nonce: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Challenge {
    pub challenge_id: String,
    pub server_nonce: String,
    pub server_proof: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Register {
    pub challenge_id: String,
    pub client_proof: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Registered {
    pub session: String,
    pub instance: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub request_id: String,
    pub page: String,
    pub generation: String,
    pub revision: String,
    pub operation: String,
    pub source: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub request_id: String,
    pub page: String,
    pub generation: String,
    pub revision: String,
    pub ok: bool,
    #[serde(default)]
    pub value: Value,
    #[serde(default)]
    pub error: Option<String>,
    pub certainty: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub pages: Vec<Page>,
    #[serde(default)]
    pub receipts: Vec<Receipt>,
    #[serde(default)]
    pub logs: Vec<AgentLog>,
    #[serde(default)]
    pub startup: Option<Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentLog {
    pub request_id: String,
    pub revision: String,
    pub user_source: String,
    pub line: u32,
    pub page: String,
    pub generation: String,
    pub level: String,
    pub message: String,
    pub source: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct TargetStatus {
    pub id: String,
    pub artifact: String,
    pub instance: String,
    pub pid: u32,
    pub page: String,
    pub generation: String,
    pub title: String,
    pub url: String,
    pub applied_revision: Option<String>,
    pub capabilities: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub sequence: u64,
    pub artifact: String,
    pub instance: String,
    pub page: Option<String>,
    pub generation: Option<String>,
    pub kind: String,
    pub data: Value,
}
