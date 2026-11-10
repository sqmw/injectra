//! Controller-to-helper boundary; target devices are owned by explicit target providers.
mod controller;
mod resources;
mod response;
use injectra_core::Result;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Bridge {
    session: PathBuf,
    resources: Vec<(&'static str, &'static str)>,
}
impl Bridge {
    pub fn new(session: &Path, resources: Vec<(&'static str, &'static str)>) -> Result<Self> {
        controller::ensure_supported()?;
        Ok(Self {
            session: std::path::absolute(session)?,
            resources,
        })
    }
    pub fn request(&self, operation: &str, payload: Value) -> Result<Value> {
        controller::invoke(
            resources::materialize(&self.resources)?,
            json!({"schema":1,"session":self.session,"operation":operation,"payload":payload}),
        )
    }
}
