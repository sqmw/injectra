//! Controller-side helper execution. This host OS is independent of the target device.
use crate::{resources, response};
use injectra_core::{Certainty, Error, ErrorKind, Phase, Result};
use serde_json::Value;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
pub(crate) fn ensure_supported() -> Result<()> {
    if !cfg!(target_os = "macos") {
        return Err(Error::prepare(
            ErrorKind::Unsupported,
            "The local Python controller is currently supported on macOS; target-device support is selected separately",
        ));
    }
    Ok(())
}
pub(crate) fn invoke(root: PathBuf, request: Value) -> Result<Value> {
    invoke_with(
        root,
        "main.py",
        request,
        Duration::from_secs(25),
        Certainty::Unknown,
    )
}

pub(crate) fn invoke_with(
    root: PathBuf,
    entry: &str,
    request: Value,
    timeout: Duration,
    timeout_certainty: Certainty,
) -> Result<Value> {
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
        .arg(root.join(entry))
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
    let deadline = Instant::now() + timeout;
    loop {
        if child
            .try_wait()
            .map_err(|e| uncertain_io(e, timeout_certainty))?
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            // Do not kill an agent owner while a native callback may still be live.
            return Err(Error::new(
                ErrorKind::Timeout,
                Phase::Execute,
                timeout_certainty,
                format!(
                    "Frida helper deadline; no replay or forced unload. Inspect {}",
                    run.display()
                ),
            ));
        }
        thread::sleep(Duration::from_millis(20));
    }
    let length = fs::metadata(&output)
        .map_err(|e| uncertain_io(e, timeout_certainty))?
        .len();
    if length > 4 * 1024 * 1024 {
        return Err(Error::new(
            ErrorKind::ResultEncoding,
            Phase::Decode,
            timeout_certainty,
            "Frida response exceeds 4 MiB",
        ));
    }
    let value: Value =
        serde_json::from_slice(&fs::read(output).map_err(|e| uncertain_io(e, timeout_certainty))?)
            .map_err(|e| {
                Error::new(
                    ErrorKind::Protocol,
                    Phase::Decode,
                    timeout_certainty,
                    format!("Invalid helper response: {e}"),
                )
            })?;
    response::decode(value)
}

fn uncertain_io(error: std::io::Error, certainty: Certainty) -> Error {
    Error::new(ErrorKind::Io, Phase::Decode, certainty, error.to_string()).caused_by(error)
}
