use axum::http::{HeaderMap, Method};
use injectra_studio::{endpoint, security::BrowserAuth};
use std::net::TcpListener;
#[test]
fn bearer_and_exact_origin_are_independent_requirements() {
    let auth = BrowserAuth::new("a".repeat(64), "http://127.0.0.1:19401".into()).unwrap();
    let mut h = HeaderMap::new();
    h.insert("host", "127.0.0.1:19401".parse().unwrap());
    assert_eq!(auth.check(&h, &Method::GET).unwrap_err().status, 401);
    h.insert(
        "authorization",
        format!("Bearer {}", "a".repeat(64)).parse().unwrap(),
    );
    assert!(auth.check(&h, &Method::GET).is_ok());
    assert_eq!(auth.check(&h, &Method::PUT).unwrap_err().status, 403);
    h.insert("origin", "http://127.0.0.1:19402".parse().unwrap());
    assert_eq!(auth.check(&h, &Method::PUT).unwrap_err().status, 403);
    h.insert("origin", "http://127.0.0.1:19401".parse().unwrap());
    assert!(auth.check(&h, &Method::PUT).is_ok());
    h.insert("host", "attacker.invalid:19401".parse().unwrap());
    assert_eq!(auth.check(&h, &Method::GET).unwrap_err().status, 403);
}
#[test]
fn endpoint_reuses_address_and_fails_explicitly_on_conflict() {
    let root = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let first = endpoint::bind(cache.path(), root.path()).unwrap();
    let port = first.record.port;
    let token = first.record.token.clone();
    assert!(endpoint::bind(cache.path(), root.path()).is_err());
    drop(first);
    let second = endpoint::bind(cache.path(), root.path()).unwrap();
    assert_eq!(port, second.record.port);
    assert_eq!(token, second.record.token);
    drop(second);
    let _occupied = TcpListener::bind(("127.0.0.1", port)).unwrap();
    assert_eq!(
        endpoint::bind(cache.path(), root.path())
            .err()
            .unwrap()
            .kind,
        "conflict"
    );
    assert_eq!(
        endpoint::read_record(cache.path(), root.path())
            .unwrap()
            .port,
        port
    );
}
#[cfg(unix)]
#[test]
fn endpoint_refuses_symlinked_private_metadata() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let lease = endpoint::bind(cache.path(), root.path()).unwrap();
    let dir = lease.directory.clone();
    drop(lease);
    let original = dir.join("ready.json");
    let other = dir.join("retained.json");
    std::fs::rename(&original, &other).unwrap();
    symlink(&other, &original).unwrap();
    assert!(endpoint::read_record(cache.path(), root.path()).is_err());
}
