//! Preparation boundary. Browser projections must never contain bridge credentials.
use crate::error::Result;
use injectra_workspace::Snapshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareRequest {
    pub app: PathBuf,
    #[serde(default = "isolated")]
    pub data_mode: String,
}
fn isolated() -> String {
    "isolated".into()
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportRequest {
    pub destination: PathBuf,
    pub target: Option<String>,
    pub generation: Option<String>,
    #[serde(default)]
    pub path_suffix: Option<String>,
    #[serde(default)]
    pub origin: Option<String>,
}
#[derive(Clone, Serialize)]
pub struct ReadonlySource {
    pub path: String,
    pub relative_path: String,
    pub origin: String,
    pub stage: String,
    pub hash: String,
    pub bytes: usize,
    pub editable: bool,
    #[serde(skip)]
    pub content: Option<String>,
}
#[derive(Default)]
pub struct ArtifactProjection {
    pub artifact: Option<Value>,
    pub files: Vec<ReadonlySource>,
    pub capabilities: Vec<String>,
}
pub trait ArtifactPort: Send + Sync {
    fn projection(&self) -> Result<ArtifactProjection>;
    fn prepare(&self, request: PrepareRequest, snapshot: &Snapshot, origin: &str) -> Result<Value>;
    fn dev(&self, snapshot: &Snapshot) -> Result<Value>;
    fn restart(&self, snapshot: &Snapshot) -> Result<Value>;
    fn export(&self, request: ExportRequest, snapshot: &Snapshot) -> Result<Value>;
}
