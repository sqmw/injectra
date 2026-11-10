#[path = "../../../tests/support/backend_conformance.rs"]
mod conformance;
#[test]
#[ignore = "Requires an explicitly selected authorized live session and page"]
fn real_gadget_page_contract() {
    let session = std::env::var("INJECTRA_TEST_SESSION").expect("explicit session");
    let target = std::env::var("INJECTRA_TEST_TARGET").expect("explicit page ID");
    let backend =
        injectra_electron_gadget::ElectronGadgetBackend::new(std::path::Path::new(&session))
            .unwrap();
    conformance::verify(&backend, &target);
}
