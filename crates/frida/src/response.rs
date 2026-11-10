use injectra_core::{Certainty, Error, ErrorKind, Phase, Result};
use serde_json::Value;
pub(crate) fn decode(value: Value) -> Result<Value> {
    if value["schema"] != 1 {
        return Err(Error::new(
            ErrorKind::Protocol,
            Phase::Decode,
            Certainty::Unknown,
            "Unknown Frida helper schema",
        ));
    }
    if value["ok"] == true {
        return value.get("value").cloned().ok_or_else(|| {
            Error::new(
                ErrorKind::Protocol,
                Phase::Decode,
                Certainty::Unknown,
                "Missing helper result",
            )
        });
    }
    let e = &value["error"];
    let certainty = match e["certainty"].as_str() {
        Some("NotExecuted") => Certainty::NotExecuted,
        Some("Executed") => Certainty::Executed,
        _ => Certainty::Unknown,
    };
    let kind = match e["kind"].as_str() {
        Some("invalid_input") => ErrorKind::InvalidInput,
        Some("unsupported") => ErrorKind::Unsupported,
        Some("target_not_found") => ErrorKind::TargetNotFound,
        Some("execution") => ErrorKind::Execution,
        Some("timeout") => ErrorKind::Timeout,
        Some("result_encoding") => ErrorKind::ResultEncoding,
        Some("connection") => ErrorKind::Connection,
        _ => ErrorKind::Protocol,
    };
    let phase = match certainty {
        Certainty::NotExecuted => Phase::Connect,
        _ => Phase::Execute,
    };
    Err(Error::new(
        kind,
        phase,
        certainty,
        format!(
            "{} [{certainty:?}]",
            e["message"]
                .as_str()
                .unwrap_or("Frida helper failed without a diagnostic")
        ),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn failures_preserve_execution_certainty() {
        for (text, expected) in [
            ("NotExecuted", Certainty::NotExecuted),
            ("Executed", Certainty::Executed),
            ("Unknown", Certainty::Unknown),
        ] {
            let e = decode(json!({"schema":1,"ok":false,"error":{"kind":"timeout","certainty":text,"message":"original"}})).unwrap_err();
            assert_eq!(e.certainty, expected);
            assert!(e.to_string().contains("original"));
        }
        assert_eq!(
            decode(json!({"schema":9,"ok":true,"value":1}))
                .unwrap_err()
                .certainty,
            Certainty::Unknown
        );
        assert!(decode(json!({"schema":1,"ok":true})).is_err());
        assert_eq!(
            decode(json!({"schema":1,"ok":true,"value":null})).unwrap(),
            Value::Null
        );
    }
}
