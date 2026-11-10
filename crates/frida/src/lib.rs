//! Optional Frida helper transport. No Web, Node or application assumptions.
mod resources;
mod response;
use injectra_core::{Certainty, Error, ErrorKind, Phase, Result};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub struct Bridge {
    session: PathBuf,
    resources: Vec<(&'static str, &'static str)>,
}
impl Bridge {
    pub fn new(session: &Path, resources: Vec<(&'static str, &'static str)>) -> Result<Self> {
        if !cfg!(target_os = "macos") {
            return Err(Error::prepare(
                ErrorKind::Unsupported,
                "Frida backends currently certify macOS only",
            ));
        }
        Ok(Self {
            session: std::path::absolute(session)?,
            resources,
        })
    }
    pub fn request(&self, operation: &str, payload: Value) -> Result<Value> {
        let root = resources::materialize(&self.resources)?;
        let cache = resources::cache()?;
        let python = std::env::var_os("INJECTRA_FRIDA_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| cache.join("frida/venv/bin/python3"));
        if !python.is_file() {
            return Err(Error::prepare(
                ErrorKind::Unsupported,
                "Optional Frida environment missing; run make setup-frida",
            ));
        }
        let run = cache.join("frida/requests").join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&run)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&run, fs::Permissions::from_mode(0o700))?;
        }
        let request =
            json!({"schema":1,"session":self.session,"operation":operation,"payload":payload});
        let input = run.join("request.json");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&input)?;
        file.write_all(
            serde_json::to_string(&request)
                .expect("request serializes")
                .as_bytes(),
        )?;
        drop(file);
        let output = run.join("response.json");
        let mut child = Command::new(python)
            .arg("-B")
            .arg(root.join("main.py"))
            .arg(&input)
            .stdin(Stdio::null())
            .stdout(fs::File::create(&output)?)
            .stderr(fs::File::create(run.join("diagnostic.log"))?)
            .spawn()
            .map_err(|e| {
                Error::prepare(
                    ErrorKind::Connection,
                    format!("Cannot start Frida helper: {e}"),
                )
            })?;
        let deadline = Instant::now() + Duration::from_secs(25);
        loop {
            if child.try_wait().map_err(uncertain_io)?.is_some() {
                break;
            }
            if Instant::now() >= deadline {
                // Do not kill an agent owner while a native callback may still be live.
                return Err(Error::new(
                    ErrorKind::Timeout,
                    Phase::Execute,
                    Certainty::Unknown,
                    format!(
                        "Frida helper deadline; no replay or forced unload. Inspect {}",
                        run.display()
                    ),
                ));
            }
            thread::sleep(Duration::from_millis(20));
        }
        let length = fs::metadata(&output).map_err(uncertain_io)?.len();
        if length > 4 * 1024 * 1024 {
            return Err(Error::new(
                ErrorKind::ResultEncoding,
                Phase::Decode,
                Certainty::Unknown,
                "Frida response exceeds 4 MiB",
            ));
        }
        let value: Value = serde_json::from_slice(&fs::read(output).map_err(uncertain_io)?)
            .map_err(|e| {
                Error::new(
                    ErrorKind::Protocol,
                    Phase::Decode,
                    Certainty::Unknown,
                    format!("Invalid helper response: {e}"),
                )
            })?;
        response::decode(value)
    }
}

fn uncertain_io(error: std::io::Error) -> Error {
    Error::new(
        ErrorKind::Io,
        Phase::Decode,
        Certainty::Unknown,
        error.to_string(),
    )
    .caused_by(error)
}
