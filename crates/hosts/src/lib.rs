//! Host-specific identification and read-only launch recipes.
mod asar_probe;
pub mod electron;
pub mod inspect;

pub use asar_probe::PackageSummary;
use injectra_core::{Error, ErrorKind, Result};
pub use inspect::{HostFamily, InspectionReport, RouteAssessment, inspect_app};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LaunchPlan {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub environment: BTreeMap<String, String>,
    pub note: &'static str,
}

pub fn launch_plan(app: &Path, port: u16) -> Result<LaunchPlan> {
    let report = inspect_app(app, Some(port))?;
    report.launch_plan.ok_or_else(|| {
        Error::prepare(
            ErrorKind::Unsupported,
            "host architecture does not support a loopback CDP launch plan; inspect routes with inspect-app",
        )
    })
}
