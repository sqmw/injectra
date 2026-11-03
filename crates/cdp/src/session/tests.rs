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
