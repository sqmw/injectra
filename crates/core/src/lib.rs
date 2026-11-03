//! Execution-domain independent identity, capabilities and failure semantics.
mod capability;
mod error;
mod identity;

pub use capability::{Capability, require_capability};
pub use error::{Certainty, Error, ErrorKind, Phase, Result};
pub use identity::{BackendId, TargetRef};
