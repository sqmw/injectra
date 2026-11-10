#[path = "../../../tests/support/backend_conformance.rs"]
mod conformance;
#[test]
#[ignore = "Requires an explicitly selected owned/authorized page"]
fn real_cdp_page_contract() {
    let port = std::env::var("INJECTRA_TEST_PORT")
        .expect("port")
        .parse()
        .unwrap();
    let target = std::env::var("INJECTRA_TEST_TARGET").expect("page");
    let backend = injectra_cdp::CdpBackend::new(port).unwrap();
    conformance::verify(&backend, &target);
}
