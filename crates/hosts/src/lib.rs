//! Host-specific identification and read-only launch recipes.
pub mod electron;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct LaunchPlan {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub note: &'static str,
}
