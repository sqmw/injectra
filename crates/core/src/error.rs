use std::{error::Error as StdError, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidInput,
    Unsupported,
    TargetNotFound,
    Io,
    Connection,
    Protocol,
    Execution,
    Timeout,
    ResultEncoding,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Prepare,
    Discover,
    Connect,
    Submit,
    Execute,
    Decode,
}
/// Executed means target execution is confirmed, not that effects are reversible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Certainty {
    NotExecuted,
    Executed,
    Unknown,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub phase: Phase,
    pub certainty: Certainty,
    message: String,
    source: Option<Box<dyn StdError + Send + Sync>>,
}
impl Error {
    pub fn new(
        kind: ErrorKind,
        phase: Phase,
        certainty: Certainty,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            phase,
            certainty,
            message: message.into(),
            source: None,
        }
    }
    pub fn prepare(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self::new(kind, Phase::Prepare, Certainty::NotExecuted, message)
    }
    pub fn caused_by<E: StdError + Send + Sync + 'static>(mut self, source: E) -> Self {
        self.source = Some(Box::new(source));
        self
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &dyn StdError)
    }
}
impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::prepare(ErrorKind::InvalidInput, message)
    }
}
impl From<&str> for Error {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}
impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        Self::prepare(ErrorKind::Io, source.to_string()).caused_by(source)
    }
}
