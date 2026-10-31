//! Runtime-independent ports. Implementations are supplied by the composition root.
use serde::Serialize;
use serde_json::Value;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Target {
    pub id: String,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    EvaluateJavaScript,
}

pub trait Session {
    fn capabilities(&self) -> &[Capability];
    fn evaluate(&mut self, source: &str) -> Result<Value>;
}

pub trait Engine {
    fn targets(&self) -> Result<Vec<Target>>;
    fn connect(&self, target_id: &str) -> Result<Box<dyn Session>>;
}

pub fn execute(session: &mut dyn Session, source: &str) -> Result<Value> {
    if !session
        .capabilities()
        .contains(&Capability::EvaluateJavaScript)
    {
        return Err("unsupported capability: evaluate JavaScript".into());
    }
    session.evaluate(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Unsupported;
    impl Session for Unsupported {
        fn capabilities(&self) -> &[Capability] {
            &[]
        }
        fn evaluate(&mut self, _: &str) -> Result<Value> {
            panic!("must not execute")
        }
    }

    #[test]
    fn rejects_unsupported_runtime_before_execution() {
        assert!(
            execute(&mut Unsupported, "throw 1")
                .unwrap_err()
                .to_string()
                .contains("unsupported")
        );
    }
}
