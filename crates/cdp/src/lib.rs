//! Loopback CDP page adapter. No application names, payloads or native operations.
mod discovery;
mod session;
use injectra_core::{BackendId, Error, ErrorKind, Result, TargetRef};
use injectra_web::{WebBackend, WebSession, WebTarget};

pub struct CdpBackend {
    port: u16,
    id: BackendId,
}
impl CdpBackend {
    pub fn new(port: u16) -> Result<Self> {
        if port == 0 {
            return Err("debugging port must be nonzero".into());
        }
        Ok(Self {
            port,
            id: BackendId::fresh(),
        })
    }
}
impl WebBackend for CdpBackend {
    fn id(&self) -> &BackendId {
        &self.id
    }
    fn targets(&self) -> Result<Vec<WebTarget>> {
        Ok(discovery::discover(self.port)?
            .into_iter()
            .map(|p| p.target(&self.id))
            .collect())
    }
    fn connect(&self, target: &TargetRef) -> Result<Box<dyn WebSession>> {
        target.ensure_backend(&self.id)?;
        let pages = discovery::discover(self.port)?;
        let matches: Vec<_> = pages.iter().filter(|p| p.id == target.key()).collect();
        if matches.len() != 1 {
            return Err(Error::prepare(
                ErrorKind::TargetNotFound,
                "target must match exactly one debuggable page; run targets again",
            ));
        }
        let socket = matches[0]
            .web_socket_debugger_url
            .as_deref()
            .ok_or("target has no debugger endpoint")?;
        Ok(Box::new(session::CdpSession::connect(
            socket,
            self.port,
            target.clone(),
        )?))
    }
}
