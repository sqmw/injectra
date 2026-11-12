//! External user increments, immutable verified revisions and execution provenance.
//! No app preparation, target process access or backend implementations live here.
mod execution;
mod filesystem;
mod model;
mod snapshot;
mod workspace;

pub use execution::*;
pub use model::*;
pub use snapshot::Snapshot;
pub use workspace::{SaveOutcome, Workspace};

use injectra_core::{Error, ErrorKind, Result};
use sha2::{Digest, Sha256};
pub const MAX_FILE_BYTES: usize = 1024 * 1024;
pub const MAX_WORKSPACE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_FILES: usize = 128;

pub fn content_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn invalid(message: impl Into<String>) -> Error {
    Error::prepare(ErrorKind::InvalidInput, message)
}
pub(crate) fn unique_id() -> Result<String> {
    let mut bytes = [0; 16];
    getrandom::fill(&mut bytes)
        .map_err(|e| invalid(format!("random identity unavailable: {e}")))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}
pub(crate) fn json<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(value).map_err(|e| invalid(e.to_string()))
}
