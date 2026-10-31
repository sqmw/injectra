//! A loopback-only CDP adapter. No Electron application names or bundle logic.
mod discovery;
mod session;

use injectra_core::{Engine, Result, Session, Target};

pub struct CdpEngine {
    port: u16,
}

impl CdpEngine {
    pub fn new(port: u16) -> Result<Self> {
        if port == 0 {
            return Err("debugging port must be nonzero".into());
        }
        Ok(Self { port })
    }
}

impl Engine for CdpEngine {
    fn targets(&self) -> Result<Vec<Target>> {
        Ok(discovery::discover(self.port)?
            .into_iter()
            .map(|page| page.target())
            .collect())
    }

    fn connect(&self, target_id: &str) -> Result<Box<dyn Session>> {
        let pages = discovery::discover(self.port)?;
        let matches: Vec<_> = pages.iter().filter(|page| page.id == target_id).collect();
        if matches.len() != 1 {
            return Err("target must match exactly one debuggable page; run targets again".into());
        }
        let page = matches[0];
        let socket = page
            .web_socket_debugger_url
            .as_deref()
            .ok_or("target has no debugger endpoint")?;
        Ok(Box::new(session::CdpSession::connect(socket, self.port)?))
    }
}
