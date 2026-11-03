use crate::{Error, ErrorKind, Result};

/// Namespaced operation capability, defined by each execution domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capability(pub &'static str);

pub fn require_capability(available: &[Capability], required: Capability) -> Result<()> {
    if !available.contains(&required) {
        return Err(Error::prepare(
            ErrorKind::Unsupported,
            format!("unsupported capability: {}", required.0),
        ));
    }
    Ok(())
}
