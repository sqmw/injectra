//! Only launches owned derived apps (or verifies explicitly attached PIDs). Never terminates an existing application.
use crate::{DataMode, PreparedArtifact, invalid, paths_equal, private_dir};
use injectra_core::{Error, ErrorKind, Result};
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn command(a: &PreparedArtifact, preflight: bool) -> Result<Command> {
    let mut c = Command::new(&a.executable);
    if let Some(dir) = a.executable.parent() {
        c.current_dir(dir);
    }
    for key in [
        "ELECTRON_RUN_AS_NODE",
        "NODE_OPTIONS",
        "DYLD_INSERT_LIBRARIES",
        "INJECTRA_PREFLIGHT_NONCE",
        "INJECTRA_PREFLIGHT_FILE",
    ] {
        c.env_remove(key);
    }
    if preflight || a.data_mode == DataMode::Isolated {
        let home = a
            .root()
            .join(if preflight { "preflight-home" } else { "home" });
        let profile = home.join("profile");
        private_dir(&profile)?;
        if cfg!(target_os = "macos") {
            private_dir(&home.join("Library/Application Support"))?;
            c.env("HOME", &home)
                .env("CFFIXED_USER_HOME", &home)
                .env("XDG_CONFIG_HOME", home.join(".config"))
                .env("XDG_CACHE_HOME", home.join(".cache"));
            c.arg(format!("--user-data-dir={}", profile.display()))
                .arg("--use-mock-keychain");
        } else if cfg!(target_os = "windows") {
            let roaming = home.join("AppData/Roaming");
            let local = home.join("AppData/Local");
            private_dir(&roaming)?;
            private_dir(&local)?;
            c.env("USERPROFILE", &home)
                .env("APPDATA", &roaming)
                .env("LOCALAPPDATA", &local);
            c.arg(format!("--user-data-dir={}", profile.display()))
                .arg("--no-first-run")
                .arg("--no-default-browser-check");
        } else {
            c.arg(format!("--user-data-dir={}", profile.display()));
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        c.process_group(0);
    }
    c.stdin(Stdio::null());
    Ok(c)
}

fn confirmed_preflight(artifact_id: &str, nonce: &str, out_file: &Path, json_file: &Path) -> bool {
    if let Ok(text) = fs::read_to_string(json_file)
        && let Ok(v) = serde_json::from_str::<Value>(text.trim())
        && v["artifact"] == artifact_id
        && v["nonce"] == nonce
    {
        return true;
    }
    if let Ok(text) = fs::read_to_string(out_file) {
        return text
            .lines()
            .filter_map(|s| s.strip_prefix("INJECTRA_PREFLIGHT:"))
            .filter_map(|s| serde_json::from_str::<Value>(s).ok())
            .any(|v| v["artifact"] == artifact_id && v["nonce"] == nonce);
    }
    false
}

pub(crate) fn preflight(a: &PreparedArtifact) -> Result<()> {
    let nonce = uuid::Uuid::new_v4().to_string();
    let out = a.root().join("preflight.stdout");
    let err = a.root().join("preflight.stderr");
    let receipt = a.root().join("preflight.json");
    let mut child = command(a, true)?
        .env("INJECTRA_PREFLIGHT_NONCE", &nonce)
        .env("INJECTRA_PREFLIGHT_FILE", &receipt)
        .stdout(fs::File::create(&out)?)
        .stderr(fs::File::create(&err)?)
        .spawn()?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            if status.success() && confirmed_preflight(&a.id, &nonce, &out, &receipt) {
                return Ok(());
            }
            return Err(Error::prepare(
                ErrorKind::Unsupported,
                format!("Bootstrap preflight failed: {}", err.display()),
            ));
        }
        if start.elapsed() > Duration::from_secs(20) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(invalid("Owned bootstrap preflight timed out"));
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn find_running_exact(executable: &Path, expected_pid: Option<u32>) -> Result<Option<u32>> {
    #[cfg(target_os = "macos")]
    {
        let list = Command::new("/bin/ps")
            .args(["-axo", "pid=,comm="])
            .output()?;
        for row in String::from_utf8_lossy(&list.stdout).lines() {
            let row = row.trim();
            if let Some((pid_s, path)) = row.split_once(char::is_whitespace)
                && paths_equal(Path::new(path.trim()), executable)
            {
                let pid: u32 = pid_s
                    .parse()
                    .map_err(|_| invalid("Invalid process identity"))?;
                if expected_pid.is_none_or(|want| want == pid) {
                    return Ok(Some(pid));
                }
            }
        }
        Ok(None)
    }
    #[cfg(target_os = "windows")]
    {
        let script = "Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath } | ForEach-Object { \"$($_.ProcessId)`t$($_.ExecutablePath)\" }";
        let list = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .output()?;
        for row in String::from_utf8_lossy(&list.stdout).lines() {
            if let Some((pid_s, path)) = row.trim().split_once('\t')
                && paths_equal(Path::new(path.trim()), executable)
            {
                let pid: u32 = pid_s
                    .trim()
                    .parse()
                    .map_err(|_| invalid("Invalid process identity"))?;
                if expected_pid.is_none_or(|want| want == pid) {
                    return Ok(Some(pid));
                }
            }
        }
        Ok(None)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (executable, expected_pid);
        Ok(None)
    }
}

pub fn launch(a: &PreparedArtifact) -> Result<u32> {
    a.verify()?;
    if a.entry_strategy == "windows-v8-attach" {
        let pid = a
            .attached_pid
            .ok_or_else(|| invalid("Missing attached PID on V8 artifact"))?;
        if find_running_exact(&a.executable, Some(pid))?.is_some() {
            return Ok(pid);
        }
        return Err(Error::prepare(
            ErrorKind::TargetNotFound,
            format!("Attached target PID {pid} is no longer running with its verified executable"),
        ));
    }
    // Existing exact derived-path instance reconnects; originals are never selected.
    if let Some(pid) = find_running_exact(&a.executable, None)? {
        return Ok(pid);
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
