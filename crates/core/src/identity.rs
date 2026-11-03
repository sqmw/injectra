use crate::{Error, ErrorKind, Result};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_INSTANCE: AtomicU64 = AtomicU64::new(1);

/// Process-local backend instance identity. Not a persistent or remote handle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackendId(u64);

impl BackendId {
    pub fn fresh() -> Self {
        Self(NEXT_INSTANCE.fetch_add(1, Ordering::Relaxed))
    }
}

/// Opaque backend-owned target key. Metadata belongs to its execution domain.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetRef {
    backend: BackendId,
    key: String,
}

impl TargetRef {
    pub fn new(backend: &BackendId, key: impl Into<String>) -> Self {
        Self {
            backend: backend.clone(),
            key: key.into(),
        }
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn backend(&self) -> &BackendId {
        &self.backend
    }
    pub fn ensure_backend(&self, backend: &BackendId) -> Result<()> {
        if &self.backend != backend {
            return Err(Error::prepare(
                ErrorKind::InvalidInput,
                "target belongs to another backend instance",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identical_local_keys_do_not_alias_between_instances() {
        let a = BackendId::fresh();
        let b = BackendId::fresh();
        let target = TargetRef::new(&a, "1");
        assert_ne!(target, TargetRef::new(&b, "1"));
        assert!(target.ensure_backend(&a).is_ok());
        assert_eq!(
            target.ensure_backend(&b).unwrap_err().certainty,
            crate::Certainty::NotExecuted
        );
    }
}
