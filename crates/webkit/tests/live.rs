#[path = "../../../tests/support/backend_conformance.rs"]
mod conformance;
#[test]
#[ignore = "Requires an explicitly selected authorized WKWebView session"]
fn real_webkit_page_contract() {
    let session = std::env::var("INJECTRA_TEST_SESSION").expect("session");
    let target = std::env::var("INJECTRA_TEST_TARGET").expect("page");
    let backend = injectra_webkit::WebkitBackend::new(std::path::Path::new(&session)).unwrap();
    conformance::verify(&backend, &target);
}
