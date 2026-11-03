use crate::{NativeTarget, TargetAddress};
use injectra_core::{
    Capability, Certainty, Error, ErrorKind, Phase, TargetRef, require_capability,
};
use std::fmt;

pub const MEMORY_READ: Capability = Capability("native.memory.read");
pub const MEMORY_WRITE: Capability = Capability("native.memory.write");
pub type MemoryResult<T> = std::result::Result<T, MemoryError>;

/// Progress is confirmed transfer only; Unknown may have additional effects.
/// Partial read bytes remain available to the caller even on failure.
#[derive(Debug)]
pub struct MemoryError {
    pub error: Error,
    pub transferred: usize,
    pub read_data: Vec<u8>,
}
impl MemoryError {
    pub fn write(error: Error, transferred: usize) -> Self {
        Self {
            error,
            transferred,
            read_data: Vec::new(),
        }
    }
    pub fn read(error: Error, data: Vec<u8>) -> Self {
        Self {
            error,
            transferred: data.len(),
            read_data: data,
        }
    }
}
impl From<Error> for MemoryError {
    fn from(error: Error) -> Self {
        Self::write(error, 0)
    }
}
impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}
impl std::error::Error for MemoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Domain-owned connection lifetime; no promise of rollback on disconnect.
pub trait NativeSession {
    fn target(&self) -> &NativeTarget;
    fn capabilities(&self) -> &[Capability];
}
/// Raw transfer port. Call read_memory for identity, capability and length checks.
pub trait MemoryRead: NativeSession {
    fn read(&mut self, address: TargetAddress, length: usize) -> MemoryResult<Vec<u8>>;
}
/// Separate port permits read-only backends without dummy write implementations.
pub trait MemoryWrite: NativeSession {
    fn write(&mut self, address: TargetAddress, bytes: &[u8]) -> MemoryResult<usize>;
}
fn validate(
    session: &(impl NativeSession + ?Sized),
    target: &TargetRef,
    cap: Capability,
    address: TargetAddress,
    length: usize,
) -> MemoryResult<()> {
    if &session.target().reference != target {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "native session belongs to another target",
        )
        .into());
    }
    require_capability(session.capabilities(), cap)?;
    session
        .target()
        .architecture
        .validate_range(address, length)?;
    Ok(())
}
pub fn read_memory(
    session: &mut (impl MemoryRead + ?Sized),
    target: &TargetRef,
    address: TargetAddress,
    length: usize,
) -> MemoryResult<Vec<u8>> {
    validate(session, target, MEMORY_READ, address, length)?;
    let bytes = session.read(address, length)?;
    if bytes.len() != length {
        let kind = if bytes.len() < length {
            ErrorKind::Execution
        } else {
            ErrorKind::Protocol
        };
        return Err(MemoryError::read(
            Error::new(
                kind,
                Phase::Execute,
                Certainty::Executed,
                "memory read did not complete the requested length",
            ),
            bytes,
        ));
    }
    Ok(bytes)
}
pub fn write_memory(
    session: &mut (impl MemoryWrite + ?Sized),
    target: &TargetRef,
    address: TargetAddress,
    bytes: &[u8],
) -> MemoryResult<usize> {
    validate(session, target, MEMORY_WRITE, address, bytes.len())?;
    let transferred = session.write(address, bytes)?;
    if transferred != bytes.len() {
        let kind = if transferred < bytes.len() {
            ErrorKind::Execution
        } else {
            ErrorKind::Protocol
        };
        return Err(MemoryError::write(
            Error::new(
                kind,
                Phase::Execute,
                Certainty::Executed,
                "memory write did not complete the requested length",
            ),
            transferred,
        ));
    }
    Ok(transferred)
}

#[cfg(test)]
mod tests;
