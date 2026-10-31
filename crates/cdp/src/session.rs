use injectra_core::{Capability, Result, Session};
use serde_json::{Value, json};
use std::{
    net::{Ipv4Addr, SocketAddr, TcpStream},
    time::{Duration, Instant},
};
use tungstenite::{Message, WebSocket};
use url::Url;

const TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn validate_socket(address: &str, port: u16) -> Result<Url> {
    let url = Url::parse(address)?;
    if url.scheme() != "ws"
        || url.host_str() != Some("127.0.0.1")
        || url.port_or_known_default() != Some(port)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !url.path().starts_with("/devtools/page/")
    {
        return Err("debugger WebSocket must be a page on the specified 127.0.0.1 port".into());
    }
    Ok(url)
}

pub(crate) struct CdpSession {
    socket: WebSocket<TcpStream>,
    next_id: u64,
}

impl CdpSession {
    pub fn connect(address: &str, port: u16) -> Result<Self> {
        let url = validate_socket(address, port)?;
        let stream =
            TcpStream::connect_timeout(&SocketAddr::from((Ipv4Addr::LOCALHOST, port)), TIMEOUT)?;
        stream.set_read_timeout(Some(TIMEOUT))?;
        stream.set_write_timeout(Some(TIMEOUT))?;
        let config = tungstenite::protocol::WebSocketConfig::default()
            .max_message_size(Some(4 * 1024 * 1024))
            .max_frame_size(Some(4 * 1024 * 1024));
        let (socket, _) =
            tungstenite::client::client_with_config(url.as_str(), stream, Some(config))?;
        Ok(Self { socket, next_id: 1 })
    }

    fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        self.socket.send(Message::Text(
            json!({"id":id,"method":method,"params":params})
                .to_string()
                .into(),
        ))?;
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or("CDP command timed out")?;
            self.socket.get_mut().set_read_timeout(Some(remaining))?;
            match self.socket.read()? {
                Message::Text(text) => {
                    let response: Value = serde_json::from_str(&text)?;
                    if response["id"].as_u64() != Some(id) {
                        continue;
                    }
                    if let Some(error) = response.get("error") {
                        return Err(format!("CDP error: {error}").into());
                    }
                    return response
                        .get("result")
                        .cloned()
                        .ok_or_else(|| "CDP reply missing result".into());
                }
                Message::Close(_) => return Err("debugger connection closed".into()),
                Message::Ping(_) => self.socket.flush()?,
                _ => {}
            }
        }
    }
}

impl Session for CdpSession {
    fn capabilities(&self) -> &[Capability] {
        &[Capability::EvaluateJavaScript]
    }
    fn evaluate(&mut self, source: &str) -> Result<Value> {
        let response = self.call(
            "Runtime.evaluate",
            json!({"expression":source,
            "returnByValue":true,"awaitPromise":true,"timeout":4000}),
        )?;
        if let Some(exception) = response.get("exceptionDetails") {
            return Err(format!("JavaScript failed: {exception}").into());
        }
        response["result"]
            .get("value")
            .cloned()
            .ok_or_else(|| "JavaScript result is not JSON serializable".into())
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
        let result =
            CdpSession::connect(&format!("ws://127.0.0.1:{port}/devtools/page/test"), port)?
                .evaluate("1");
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
