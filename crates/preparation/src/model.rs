use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataMode {
    Shared,
    Isolated,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrepareRequest {
    pub source_app: PathBuf,
    pub output_root: PathBuf,
    pub data_mode: DataMode,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Probe {
    pub source_app: PathBuf,
    pub executable: PathBuf,
    pub bundle_id: String,
    pub version: String,
    pub layout: String,
    pub package_root: String,
    pub main_entry: String,
    pub sandboxed: bool,
    pub baseline: BTreeMap<String, String>,
    pub integrity_digest: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreparationPlan {
    pub probe: Probe,
    pub changes: Vec<Value>,
    pub data_mode: DataMode,
    pub signing_policy: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreparedArtifact {
    pub version: u32,
    pub id: String,
    pub source_app: PathBuf,
    pub app: PathBuf,
    pub executable: PathBuf,
    pub artifact_file: PathBuf,
    pub controller_origin: String,
    pub data_mode: DataMode,
    pub package_root: String,
    pub main_entry: String,
    pub baseline: BTreeMap<String, String>,
    pub changes: Vec<Value>,
    pub excluded_unsealed: Vec<String>,
}
