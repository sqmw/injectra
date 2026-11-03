use injectra_core::{Capability, Certainty, Error, ErrorKind, Phase, Result, TargetRef};
use injectra_web::{PAGE_EVALUATE, WebSession};
use serde_json::{Value, json};
use std::{
    net::{Ipv4Addr, SocketAddr, TcpStream},
    time::{Duration, Instant},
};
use tungstenite::{Message, WebSocket};
use url::Url;

const TIMEOUT: Duration = Duration::from_secs(5);
fn failure<E: std::error::Error + Send + Sync + 'static>(
    kind: ErrorKind,
    phase: Phase,
    certainty: Certainty,
    e: E,
) -> Error {
    Error::new(kind, phase, certainty, e.to_string()).caused_by(e)
}
fn uncertain(e: tungstenite::Error, phase: Phase) -> Error {
    let kind = match &e {
        tungstenite::Error::Io(io)
            if matches!(
                io.kind(),
                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
            ) =>
        {
            ErrorKind::Timeout
        }
        _ => ErrorKind::Connection,
    };
    failure(kind, phase, Certainty::Unknown, e)
}
pub(crate) fn validate_socket(address: &str, port: u16) -> Result<Url> {
    let url = Url::parse(address).map_err(|e| {
        failure(
            ErrorKind::InvalidInput,
            Phase::Connect,
            Certainty::NotExecuted,
            e,
        )
    })?;
    if url.scheme() != "ws"
        || url.host_str() != Some("127.0.0.1")
        || url.port_or_known_default() != Some(port)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !url.path().starts_with("/devtools/page/")
    {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            Phase::Connect,
            Certainty::NotExecuted,
            "debugger WebSocket must be a page on the specified 127.0.0.1 port",
        ));
    }
    Ok(url)
}
pub(crate) struct CdpSession {
    socket: WebSocket<TcpStream>,
    next_id: u64,
    target: TargetRef,
}
impl CdpSession {
    pub fn connect(address: &str, port: u16, target: TargetRef) -> Result<Self> {
        let url = validate_socket(address, port)?;
        let configure = |e| {
            failure(
                ErrorKind::Connection,
                Phase::Connect,
                Certainty::NotExecuted,
                e,
            )
        };
        let stream =
            TcpStream::connect_timeout(&SocketAddr::from((Ipv4Addr::LOCALHOST, port)), TIMEOUT)
                .map_err(configure)?;
        stream.set_read_timeout(Some(TIMEOUT)).map_err(configure)?;
        stream.set_write_timeout(Some(TIMEOUT)).map_err(configure)?;
        let config = tungstenite::protocol::WebSocketConfig::default()
            .max_message_size(Some(4 * 1024 * 1024))
            .max_frame_size(Some(4 * 1024 * 1024));
        let (socket, _) =
            tungstenite::client::client_with_config(url.as_str(), stream, Some(config)).map_err(
                |e| {
                    failure(
                        ErrorKind::Connection,
                        Phase::Connect,
                        Certainty::NotExecuted,
                        e,
                    )
                },
            )?;
        Ok(Self {
            socket,
            next_id: 1,
            target,
        })
    }
    fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        self.socket
            .send(Message::Text(
                json!({"id":id,"method":method,"params":params})
                    .to_string()
                    .into(),
            ))
            .map_err(|e| uncertain(e, Phase::Submit))?;
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or_else(|| {
                    Error::new(
                        ErrorKind::Timeout,
                        Phase::Execute,
                        Certainty::Unknown,
                        "CDP command timed out",
                    )
                })?;
            self.socket
                .get_mut()
                .set_read_timeout(Some(remaining))
                .map_err(|e| {
                    failure(ErrorKind::Connection, Phase::Execute, Certainty::Unknown, e)
                })?;
            match self
                .socket
                .read()
                .map_err(|e| uncertain(e, Phase::Execute))?
            {
                Message::Text(text) => {
                    let response: Value = serde_json::from_str(&text).map_err(|e| {
                        failure(ErrorKind::Protocol, Phase::Decode, Certainty::Unknown, e)
                    })?;
                    if response["id"].as_u64() != Some(id) {
                        continue;
                    }
                    if let Some(error) = response.get("error") {
                        // A generic protocol error does not prove the expression never ran.
                        return Err(Error::new(
                            ErrorKind::Protocol,
                            Phase::Execute,
                            Certainty::Unknown,
                            format!("CDP error: {error}"),
                        ));
                    }
                    return response.get("result").cloned().ok_or_else(|| {
                        Error::new(
                            ErrorKind::Protocol,
                            Phase::Decode,
                            Certainty::Unknown,
                            "CDP reply missing result",
                        )
                    });
                }
                Message::Close(_) => {
                    return Err(Error::new(
                        ErrorKind::Connection,
                        Phase::Execute,
                        Certainty::Unknown,
                        "debugger connection closed",
                    ));
                }
                Message::Ping(_) => self
                    .socket
                    .flush()
                    .map_err(|e| uncertain(e, Phase::Execute))?,
                _ => {}
            }
        }
    }
}
impl WebSession for CdpSession {
    fn target(&self) -> &TargetRef {
        &self.target
    }
    fn capabilities(&self) -> &[Capability] {
        &[PAGE_EVALUATE]
    }
    fn evaluate_page(&mut self, source: &str) -> Result<Value> {
        let response = self.call(
            "Runtime.evaluate",
            json!({"expression":source,"returnByValue":true,"awaitPromise":true,"timeout":4000}),
        )?;
        if let Some(exception) = response.get("exceptionDetails") {
            return Err(Error::new(
                ErrorKind::Execution,
                Phase::Execute,
                Certainty::Executed,
                format!("JavaScript failed: {exception}"),
            ));
        }
        response["result"].get("value").cloned().ok_or_else(|| {
            Error::new(
                ErrorKind::ResultEncoding,
                Phase::Decode,
                Certainty::Executed,
                "JavaScript result is not JSON serializable",
            )
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn reply_from_server(reply: Value) -> Result<Value> {
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let port = listener.local_addr()?.port();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(TIMEOUT)).unwrap();
            let mut socket = tungstenite::accept(stream).unwrap();
            let command: Value =
                serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
            assert_eq!(command["method"], "Runtime.evaluate");
            socket
                .send(Message::Text(
                    json!({"method":"Runtime.consoleAPICalled","params":{}})
                        .to_string()
                        .into(),
                ))
                .unwrap();
            socket
                .send(Message::Text(
                    json!({"id":999,"result":{}}).to_string().into(),
                ))
                .unwrap();
            let mut reply = reply;
            reply["id"] = command["id"].clone();
            socket
                .send(Message::Text(reply.to_string().into()))
                .unwrap();
        });
        let result = CdpSession::connect(
            &format!("ws://127.0.0.1:{port}/devtools/page/test"),
            port,
            TargetRef::new(&injectra_core::BackendId::fresh(), "test"),
        )?
        .evaluate_page("1");
        server.join().unwrap();
        result
    }

    #[test]
    fn correlates_replies_and_reports_protocol_and_script_errors() {
        assert_eq!(
            reply_from_server(json!({"result":{"result":{"value":1}}})).unwrap(),
            json!(1)
        );
        assert!(
            reply_from_server(json!({"error":{"message":"unsupported"}}))
                .unwrap_err()
                .to_string()
                .contains("CDP error")
        );
        assert!(
            reply_from_server(json!({"result":{"exceptionDetails":{"text":"script failure"}}}))
                .unwrap_err()
                .to_string()
                .contains("JavaScript failed")
        );
    }
    #[test]
    fn discovery_cannot_redirect_execution_off_endpoint() {
        for value in [
            "ws://evil.test:9222/devtools/page/1",
            "ws://127.0.0.1:9223/devtools/page/1",
            "ws://user@127.0.0.1:9222/devtools/page/1",
            "ws://127.0.0.1:9222/devtools/browser/1",
            "wss://127.0.0.1:9222/devtools/page/1",
        ] {
            assert!(validate_socket(value, 9222).is_err(), "{value}");
        }
        assert!(validate_socket("ws://127.0.0.1:9222/devtools/page/abc", 9222).is_ok());
    }
}

#[cfg(test)]
mod failure_contract {
    use super::*;
    use std::net::TcpListener;
    fn exchange(reply: Option<Value>) -> Error {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(TIMEOUT)).unwrap();
            let mut socket = tungstenite::accept(stream).unwrap();
            let request: Value =
                serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
            if let Some(mut reply) = reply {
                reply["id"] = request["id"].clone();
                socket
                    .send(Message::Text(reply.to_string().into()))
                    .unwrap();
            } else {
                socket.close(None).unwrap();
            }
        });
        let target = TargetRef::new(&injectra_core::BackendId::fresh(), "test");
        let result = CdpSession::connect(
            &format!("ws://127.0.0.1:{port}/devtools/page/test"),
            port,
            target,
        )
        .unwrap()
        .evaluate_page("effect()");
        server.join().unwrap();
        result.unwrap_err()
    }
    #[test]
    fn classifies_script_results_protocol_errors_and_lost_replies() {
        let script = exchange(Some(
            json!({"result":{"exceptionDetails":{"text":"bad script"}}}),
        ));
        assert_eq!(
            (script.kind, script.certainty),
            (ErrorKind::Execution, Certainty::Executed)
        );
        let encoding = exchange(Some(json!({"result":{"result":{"type":"undefined"}}})));
        assert_eq!(
            (encoding.kind, encoding.certainty),
            (ErrorKind::ResultEncoding, Certainty::Executed)
        );
        let protocol = exchange(Some(
            json!({"error":{"message":"execution may have timed out"}}),
        ));
        assert_eq!(protocol.certainty, Certainty::Unknown);
        let lost = exchange(None);
        assert_eq!(
            (lost.kind, lost.certainty),
            (ErrorKind::Connection, Certainty::Unknown)
        );
        let invalid = validate_socket("ws://example.test/devtools/page/x", 9222).unwrap_err();
        assert_eq!(invalid.certainty, Certainty::NotExecuted);
    }
}
