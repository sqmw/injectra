//! Only launches owned derived apps. Never terminates an existing application.
use crate::{DataMode, PreparedArtifact, invalid, private_dir};
use injectra_core::{Error, ErrorKind, Result};
use serde_json::Value;
use std::{
    fs,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
fn command(a: &PreparedArtifact, preflight: bool) -> Result<Command> {
    let mut c = Command::new(&a.executable);
    for key in [
        "ELECTRON_RUN_AS_NODE",
        "NODE_OPTIONS",
        "DYLD_INSERT_LIBRARIES",
        "INJECTRA_PREFLIGHT_NONCE",
    ] {
        c.env_remove(key);
    }
    if preflight || a.data_mode == DataMode::Isolated {
        let home = a
            .root()
            .join(if preflight { "preflight-home" } else { "home" });
        private_dir(&home.join("Library/Application Support"))?;
        c.env("HOME", &home)
            .env("CFFIXED_USER_HOME", &home)
            .env("XDG_CONFIG_HOME", home.join(".config"))
            .env("XDG_CACHE_HOME", home.join(".cache"));
        c.arg(format!(
            "--user-data-dir={}",
            home.join("profile").display()
        ))
        .arg("--use-mock-keychain");
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        c.process_group(0);
    }
    c.stdin(Stdio::null());
    Ok(c)
}
pub(crate) fn preflight(a: &PreparedArtifact) -> Result<()> {
    let nonce = uuid::Uuid::new_v4().to_string();
    let out = a.root().join("preflight.stdout");
    let err = a.root().join("preflight.stderr");
    let mut child = command(a, true)?
        .env("INJECTRA_PREFLIGHT_NONCE", &nonce)
        .stdout(fs::File::create(&out)?)
        .stderr(fs::File::create(&err)?)
        .spawn()?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let confirmed = fs::read_to_string(&out)?
                .lines()
                .filter_map(|s| s.strip_prefix("INJECTRA_PREFLIGHT:"))
                .filter_map(|s| serde_json::from_str::<Value>(s).ok())
                .any(|v| v["artifact"] == a.id && v["nonce"] == nonce);
            if status.success() && confirmed {
                return Ok(());
            }
            return Err(Error::prepare(
                ErrorKind::Unsupported,
                format!("Bootstrap preflight failed: {}", err.display()),
            ));
        }
        if start.elapsed() > Duration::from_secs(20) {
            child.kill()?;
            child.wait()?;
            return Err(invalid("Owned bootstrap preflight timed out"));
        }
        thread::sleep(Duration::from_millis(50));
    }
}
pub fn launch(a: &PreparedArtifact) -> Result<u32> {
    a.verify()?;
    // Existing exact derived-path instance reconnects; originals are never selected.
    let list = Command::new("/bin/ps")
        .args(["-axo", "pid=,comm="])
        .output()?;
    for row in String::from_utf8_lossy(&list.stdout).lines() {
        let row = row.trim();
        if let Some((pid, path)) = row.split_once(char::is_whitespace)
            && path.trim() == a.executable.to_string_lossy()
        {
            return pid.parse().map_err(|_| invalid("Invalid process identity"));
        }
    }
    let log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(a.root().join("process.log"))?;
    let mut child = command(a, false)?
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;
    let pid = child.id();
    thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(pid)
}
