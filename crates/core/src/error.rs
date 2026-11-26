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
    PolicyBlocked,
    Execution,
    Timeout,
    ResultEncoding,
}
impl ErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "InvalidInput",
            Self::Unsupported => "Unsupported",
            Self::TargetNotFound => "TargetNotFound",
            Self::Io => "Io",
            Self::Connection => "Connection",
            Self::Protocol => "Protocol",
            Self::PolicyBlocked => "PolicyBlocked",
            Self::Execution => "Execution",
            Self::Timeout => "Timeout",
            Self::ResultEncoding => "ResultEncoding",
        }
    }
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
impl Phase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prepare => "Prepare",
            Self::Discover => "Discover",
            Self::Connect => "Connect",
            Self::Submit => "Submit",
            Self::Execute => "Execute",
            Self::Decode => "Decode",
        }
    }
}
/// Executed means target execution is confirmed, not that effects are reversible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Certainty {
    NotExecuted,
    Executed,
    Unknown,
}
impl Certainty {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotExecuted => "NotExecuted",
            Self::Executed => "Executed",
            Self::Unknown => "Unknown",
        }
    }
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
    pub fn causes(&self) -> Vec<String> {
        let mut chain = Vec::new();
        let mut current: Option<&(dyn StdError + 'static)> = self.source();
        while let Some(err) = current {
            chain.push(err.to_string());
            current = err.source();
        }
        chain
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    #[derive(Debug)]
    struct NestedError {
        label: &'static str,
        inner: Option<Box<NestedError>>,
    }
    impl fmt::Display for NestedError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.label)
        }
    }
    impl StdError for NestedError {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            self.inner
                .as_deref()
                .map(|e| e as &(dyn StdError + 'static))
        }
    }

    #[test]
    fn error_variants_expose_stable_identifiers_and_cause_chain() {
        assert_eq!(ErrorKind::InvalidInput.as_str(), "InvalidInput");
        assert_eq!(ErrorKind::Unsupported.as_str(), "Unsupported");
        assert_eq!(ErrorKind::TargetNotFound.as_str(), "TargetNotFound");
        assert_eq!(ErrorKind::Io.as_str(), "Io");
        assert_eq!(ErrorKind::Connection.as_str(), "Connection");
        assert_eq!(ErrorKind::Protocol.as_str(), "Protocol");
        assert_eq!(ErrorKind::PolicyBlocked.as_str(), "PolicyBlocked");
        assert_eq!(ErrorKind::Execution.as_str(), "Execution");
        assert_eq!(ErrorKind::Timeout.as_str(), "Timeout");
        assert_eq!(ErrorKind::ResultEncoding.as_str(), "ResultEncoding");

        assert_eq!(Phase::Prepare.as_str(), "Prepare");
        assert_eq!(Phase::Discover.as_str(), "Discover");
        assert_eq!(Phase::Connect.as_str(), "Connect");
        assert_eq!(Phase::Submit.as_str(), "Submit");
        assert_eq!(Phase::Execute.as_str(), "Execute");
        assert_eq!(Phase::Decode.as_str(), "Decode");

        assert_eq!(Certainty::NotExecuted.as_str(), "NotExecuted");
        assert_eq!(Certainty::Executed.as_str(), "Executed");
        assert_eq!(Certainty::Unknown.as_str(), "Unknown");

        let root = NestedError {
            label: "socket reset",
            inner: Some(Box::new(NestedError {
                label: "broken pipe",
                inner: None,
            })),
        };
        let err = Error::new(
            ErrorKind::Connection,
            Phase::Execute,
            Certainty::Unknown,
            "lost transport",
        )
        .caused_by(root);
        assert_eq!(err.causes(), vec!["socket reset", "broken pipe"]);
    }
}
