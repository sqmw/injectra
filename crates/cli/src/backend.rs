use crate::args::{Backend, Endpoint};
use injectra_core::Result;
use injectra_web::WebBackend;
pub(crate) fn create(endpoint: Endpoint) -> Result<Box<dyn WebBackend>> {
    match endpoint.backend.unwrap_or(Backend::Cdp) {
        Backend::Cdp => Ok(Box::new(injectra_cdp::CdpBackend::new(
            endpoint.port.ok_or("--port is required for CDP")?,
        )?)),
        Backend::ElectronGadget => Ok(Box::new(
            injectra_electron_gadget::ElectronGadgetBackend::new(
                &endpoint.session.ok_or("--session is required for Gadget")?,
            )?,
        )),
    }
}
