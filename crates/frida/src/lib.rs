//! Controller-to-helper boundary; target devices are owned by explicit target providers.
mod controller;
mod resources;
mod response;
use injectra_core::{Certainty, Error, ErrorKind, Result};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionRoute {
    ElectronGadget,
    Webkit,
}
impl SessionRoute {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ElectronGadget => "electron-gadget",
            Self::Webkit => "webkit",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LauncherMode {
    Direct,
    LaunchServices,
}
impl LauncherMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::LaunchServices => "launchservices",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LaunchSessionRequest {
    pub app: PathBuf,
    pub gadget: Option<PathBuf>,
    pub arguments: Vec<String>,
    pub route: SessionRoute,
    pub launcher: LauncherMode,
    pub output: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct RegisterSessionRequest {
    pub state: PathBuf,
    pub output: PathBuf,
    pub route: SessionRoute,
}

#[derive(Clone, Debug)]
pub struct WindowsV8AttachRequest {
    pub pid: u32,
    pub exe: PathBuf,
    pub started: Option<u64>,
    pub module: Option<PathBuf>,
    pub source: String,
}

pub fn ensure_windows_v8_supported() -> Result<()> {
    controller::ensure_windows_supported()
}

#[derive(Clone)]
pub struct Bridge {
    session: PathBuf,
    resources: Vec<(&'static str, &'static str)>,
}
impl Bridge {
    pub fn new(session: &Path, resources: Vec<(&'static str, &'static str)>) -> Result<Self> {
        controller::ensure_supported()?;
        Ok(Self {
            session: std::path::absolute(session)?,
            resources,
        })
    }
    pub fn request(&self, operation: &str, payload: Value) -> Result<Value> {
        controller::invoke(
            resources::materialize(&self.resources)?,
            json!({"schema":1,"session":self.session,"operation":operation,"payload":payload}),
        )
    }
}

fn resolve_gadget(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        let abs = std::path::absolute(path)?;
        if !abs.is_file() {
            return Err(Error::prepare(
                ErrorKind::InvalidInput,
                format!("Gadget library not found: {}", abs.display()),
            ));
        }
        return Ok(abs);
    }
    if let Some(env_path) = std::env::var_os("INJECTRA_TEST_GADGET") {
        let abs = std::path::absolute(PathBuf::from(env_path))?;
        if abs.is_file() {
            return Ok(abs);
        }
    }
    let default = resources::cache()?.join("frida/gadget/FridaGadget.dylib");
    if default.is_file() {
        return Ok(default);
    }
    Err(Error::prepare(
        ErrorKind::Unsupported,
        "FridaGadget.dylib missing; pass --gadget or run make setup-gadget",
    ))
}

pub fn launch_session(request: &LaunchSessionRequest) -> Result<Value> {
    controller::ensure_supported()?;
    let app = std::path::absolute(&request.app)?;
    let gadget = resolve_gadget(request.gadget.as_deref())?;
    let output = request
        .output
        .as_deref()
        .map(std::path::absolute)
        .transpose()?;
    if let Some(out) = &output
        && out.exists()
    {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "Session output file already exists",
        ));
    }
    let root = resources::materialize(&[])?;
    controller::invoke_with(
        root,
        "session_ops.py",
        json!({
            "schema": 1,
            "operation": "launch",
            "payload": {
                "app": app,
                "gadget": gadget,
                "arguments": request.arguments,
                "route": request.route.as_str(),
                "launcher": request.launcher.as_str(),
                "output": output,
            }
        }),
        Duration::from_secs(40),
        Certainty::NotExecuted,
    )
}

pub fn register_session(request: &RegisterSessionRequest) -> Result<Value> {
    controller::ensure_supported()?;
    let state = std::path::absolute(&request.state)?;
    let output = std::path::absolute(&request.output)?;
    if output.exists() {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "Session output file already exists",
        ));
    }
    let root = resources::materialize(&[])?;
    controller::invoke_with(
        root,
        "session_ops.py",
        json!({
            "schema": 1,
            "operation": "register",
            "payload": {
                "state": state,
                "output": output,
                "route": request.route.as_str(),
            }
        }),
        Duration::from_secs(20),
        Certainty::NotExecuted,
    )
}

pub fn verify_session(session: &Path, route: Option<SessionRoute>) -> Result<Value> {
    controller::ensure_supported()?;
    let session = std::path::absolute(session)?;
    let root = resources::materialize(&[])?;
    controller::invoke_with(
        root,
        "session_ops.py",
        json!({
            "schema": 1,
            "operation": "verify",
            "payload": {
                "session": session,
                "route": route.map(SessionRoute::as_str),
            }
        }),
        Duration::from_secs(20),
        Certainty::NotExecuted,
    )
}

pub fn attach_windows_v8(request: &WindowsV8AttachRequest) -> Result<Value> {
    controller::ensure_windows_supported()?;
    if request.pid == 0 {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "Target PID must be nonzero",
        ));
    }
    if request.source.trim().is_empty() {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "V8 dispatch source must not be empty",
        ));
    }
    let exe = std::path::absolute(&request.exe)?;
    let module = request
        .module
        .as_deref()
        .map(std::path::absolute)
        .transpose()?;
    let root = resources::materialize(&[])?;
    controller::invoke_with(
        root,
        "v8_attach.py",
        json!({
            "schema": 1,
            "operation": "attach-v8",
            "payload": {
                "pid": request.pid,
                "exe": exe,
                "started": request.started,
                "module": module,
                "source": request.source,
            }
        }),
        Duration::from_secs(25),
        Certainty::Unknown,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_gadget_rejects_missing_explicit_file_before_helper_spawn() {
        let missing = std::env::temp_dir().join("injectra-missing-gadget-dylib-test");
        let err = resolve_gadget(Some(&missing)).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert_eq!(err.certainty, Certainty::NotExecuted);
    }
}
