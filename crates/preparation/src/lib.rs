//! Generic Electron (CJS & ESM) dev-entry preparation across macOS and Windows; only a new owned copy is mutated.
use injectra_core::{Error, ErrorKind, Result};
pub use launch::launch;
pub use model::*;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use uuid::Uuid;
mod launch;
mod model;

pub(crate) fn invalid(message: impl Into<String>) -> Error {
    Error::prepare(ErrorKind::InvalidInput, message)
}
pub(crate) fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn canonical_path(path: &Path) -> Result<PathBuf> {
    let canonical = fs::canonicalize(path)?;
    #[cfg(windows)]
    {
        if let Some(text) = canonical.to_str()
            && let Some(stripped) = text.strip_prefix(r"\\?\")
            && !stripped.starts_with(r"UNC\")
        {
            return Ok(PathBuf::from(stripped));
        }
    }
    Ok(canonical)
}
pub(crate) fn paths_equal(a: &Path, b: &Path) -> bool {
    if cfg!(windows) {
        a.to_string_lossy()
            .replace('/', "\\")
            .eq_ignore_ascii_case(&b.to_string_lossy().replace('/', "\\"))
    } else {
        a == b
    }
}
pub(crate) fn path_starts_with(path: &Path, base: &Path) -> bool {
    if cfg!(windows) {
        let p = path
            .to_string_lossy()
            .replace('/', "\\")
            .to_ascii_lowercase();
        let mut b = base
            .to_string_lossy()
            .replace('/', "\\")
            .to_ascii_lowercase();
        if !b.ends_with('\\') {
            b.push('\\');
        }
        p == b.trim_end_matches('\\') || p.starts_with(&b)
    } else {
        path.starts_with(base)
    }
}
pub(crate) fn private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
pub(crate) fn private_write(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::fs::OpenOptions;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
fn cache() -> Result<PathBuf> {
    if let Some(p) = std::env::var_os("INJECTRA_CACHE_DIR") {
        return Ok(p.into());
    }
    if let Some(p) = std::env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(p).join("injectra"));
    }
    #[cfg(windows)]
    {
        if let Some(p) = std::env::var_os("LOCALAPPDATA") {
            return Ok(PathBuf::from(p).join("injectra"));
        }
        if let Some(p) = std::env::var_os("USERPROFILE") {
            return Ok(PathBuf::from(p).join(".cache").join("injectra"));
        }
    }
    let root = std::env::var_os("HOME")
        .map(|p| PathBuf::from(p).join(".cache"))
        .ok_or_else(|| invalid("No cache location"))?;
    Ok(root.join("injectra"))
}
pub(crate) fn helper() -> Result<PathBuf> {
    let sources = [
        ("prepare.mjs", include_str!("../helper/prepare.mjs")),
        ("entry.mjs", include_str!("../helper/entry.mjs")),
        ("archive.mjs", include_str!("../helper/archive.mjs")),
        ("copy_tree.mjs", include_str!("../helper/copy_tree.mjs")),
        (
            "pe_integrity.mjs",
            include_str!("../helper/pe_integrity.mjs"),
        ),
        ("platform.py", include_str!("../helper/platform.py")),
        ("bootstrap.cjs", injectra_bootstrap::BOOTSTRAP_SOURCE),
        ("evaluation.cjs", injectra_bootstrap::EVALUATION_SOURCE),
    ];
    let version = sha(sources
        .iter()
        .flat_map(|(_, s)| s.as_bytes())
        .copied()
        .collect::<Vec<_>>()
        .as_slice());
    let root = cache()?.join("preparation-helpers").join(version);
    private_dir(&root)?;
    for (name, source) in sources {
        let p = root.join(name);
        if p.exists() {
            if fs::read(&p)? != source.as_bytes() {
                return Err(invalid("Materialized helper changed unexpectedly"));
            }
        } else {
            private_write(&p, source.as_bytes())?;
        }
    }
    Ok(root.join("prepare.mjs"))
}
fn is_supported_host() -> bool {
    (std::env::consts::OS == "macos" && std::env::consts::ARCH == "aarch64")
        || (std::env::consts::OS == "windows" && matches!(std::env::consts::ARCH, "x86_64" | "x86"))
}
fn run<T: DeserializeOwned>(request: Value) -> Result<T> {
    if !is_supported_host() {
        return Err(Error::prepare(
            ErrorKind::Unsupported,
            "Preparation supports macOS Apple Silicon and Windows x64/x86 only",
        ));
    }
    let tools = std::env::var_os("INJECTRA_PREPARATION_TOOLS")
        .map(PathBuf::from)
        .unwrap_or(cache()?.join("preparation-tools"));
    let base_ok = tools
        .join("node_modules/@electron/asar/package.json")
        .is_file()
        && tools.join("node_modules/acorn/package.json").is_file();
    let win_ok = !cfg!(windows)
        || (tools
            .join("node_modules/chromium-pickle-js/package.json")
            .is_file()
            && tools.join("node_modules/resedit/package.json").is_file());
    if !base_ok || !win_ok {
        return Err(Error::prepare(
            ErrorKind::Unsupported,
            "Preparation tools are missing; run make setup-preparation (or set INJECTRA_PREPARATION_TOOLS)",
        ));
    }
    let mut child =
        Command::new(std::env::var_os("INJECTRA_NODE").unwrap_or_else(|| "node".into()))
            .arg(helper()?)
            .env("INJECTRA_PREPARATION_TOOLS", tools)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&request).map_err(|e| invalid(e.to_string()))?)?;
    let output = child.wait_with_output()?;
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| {
        Error::prepare(
            ErrorKind::Protocol,
            format!(
                "Preparation helper returned no valid result: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        )
        .caused_by(e)
    })?;
    if !output.status.success() || value["ok"] != true {
        let kind = if value["kind"] == "Unsupported" {
            ErrorKind::Unsupported
        } else {
            ErrorKind::Io
        };
        return Err(Error::prepare(
            kind,
            value["message"].as_str().unwrap_or("Preparation failed"),
        ));
    }
    serde_json::from_value(value["value"].clone())
        .map_err(|e| Error::prepare(ErrorKind::Protocol, "Invalid preparation result").caused_by(e))
}
pub fn probe(app: &Path) -> Result<Probe> {
    run(json!({"operation":"probe","source_app":app}))
}
pub fn plan(request: &PrepareRequest) -> Result<PreparationPlan> {
    run(json!({"operation":"plan","source_app":request.source_app,"data_mode":request.data_mode}))
}
pub fn prepare(request: &PrepareRequest) -> Result<PreparedArtifact> {
    let plan = plan(request)?;
    let source = canonical_path(&plan.probe.source_app)?;
    if path_starts_with(&request.output_root, &source) {
        return Err(invalid("Output is inside source application"));
    }
    private_dir(&request.output_root)?;
    let output = canonical_path(&request.output_root)?;
    if path_starts_with(&output, &source) {
        return Err(invalid("Output is inside source application"));
    }
    let id = Uuid::new_v4().to_string();
    let root = output.join(&id);
    private_dir(&root)?;
    let platform = std::env::consts::OS.to_owned();
    let (app, executable) = if cfg!(target_os = "macos") {
        let app = root.join("Prepared.app");
        let executable = app.join("Contents/MacOS").join(
            plan.probe
                .executable
                .file_name()
                .ok_or("Missing executable")?,
        );
        (app, executable)
    } else {
        let app = root.join("PreparedApp");
        let probed_exe = canonical_path(&plan.probe.executable)?;
        let rel_exe = probed_exe
            .strip_prefix(&source)
            .map_err(|_| invalid("Resolved executable is outside source root"))?;
        let executable = app.join(rel_exe);
        (app, executable)
    };
    // Select once, persist in signed/owned config; dev must report conflicts rather than switch ports.
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let origin = format!("http://{}/", listener.local_addr()?);
    let key = injectra_bootstrap::random_token();
    let ownership = json!({"owner":"injectra.preparation.v1","platform":platform,"id":id,"app":app,"source_app":source,"data_mode":request.data_mode,"controller_origin":origin,"executable":executable});
    private_write(
        &root.join("ownership.json"),
        &serde_json::to_vec(&ownership).unwrap(),
    )?;
    private_write(
        &root.join("pairing.json"),
        &serde_json::to_vec(&json!({"artifact":id,"key":key})).unwrap(),
    )?;
    let result: Value = run(
        json!({"operation":"prepare","source_app":source,"app":app,"data_mode":request.data_mode,"config":{"version":1,"artifact":id,"key":key,"controller_origin":origin}}),
    )?;
    let artifact = PreparedArtifact {
        platform,
        entry_strategy: plan.probe.entry_strategy,
        attached_pid: None,
        version: 1,
        id,
        source_app: source,
        app,
        executable,
        artifact_file: root.join("artifact.json"),
        controller_origin: origin,
        data_mode: request.data_mode,
        package_root: plan.probe.package_root,
        main_entry: plan.probe.main_entry,
        baseline: plan.probe.baseline,
        changes: serde_json::from_value(result["changes"].clone())
            .map_err(|e| invalid(e.to_string()))?,
        excluded_unsealed: serde_json::from_value(result["excluded_unsealed"].clone())
            .map_err(|e| invalid(e.to_string()))?,
    };
    launch::preflight(&artifact)?;
    private_write(
        &artifact.artifact_file,
        &serde_json::to_vec_pretty(&artifact).unwrap(),
    )?;
    Ok(artifact)
}

/// Materialize an owned Bootstrap runtime directory for a running process attached via Windows V8
/// without copying or mutating the target application on disk.
pub fn prepare_attached_bootstrap(
    output_root: &Path,
    executable: &Path,
    pid: u32,
) -> Result<(PreparedArtifact, PathBuf)> {
    let exe = canonical_path(executable)?;
    let source = exe
        .parent()
        .ok_or_else(|| invalid("Executable has no parent directory"))?
        .to_path_buf();
    private_dir(output_root)?;
    let output = canonical_path(output_root)?;
    if path_starts_with(&output, &source) {
        return Err(invalid("Output is inside source application"));
    }
    let id = Uuid::new_v4().to_string();
    let root = output.join(&id);
    private_dir(&root)?;
    let app = root.join("AttachedRuntime");
    let runtime_dir = app.join(".injectra");
    private_dir(&runtime_dir)?;
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let origin = format!("http://{}/", listener.local_addr()?);
    let key = injectra_bootstrap::random_token();
    let config = json!({"version":1,"artifact":id,"key":key,"controller_origin":origin});
    let bootstrap_path = runtime_dir.join("bootstrap.cjs");
    private_write(
        &bootstrap_path,
        injectra_bootstrap::BOOTSTRAP_SOURCE.as_bytes(),
    )?;
    private_write(
        &runtime_dir.join("evaluation.cjs"),
        injectra_bootstrap::EVALUATION_SOURCE.as_bytes(),
    )?;
    private_write(
        &runtime_dir.join("config.json"),
        &serde_json::to_vec(&config).unwrap(),
    )?;
    let platform = std::env::consts::OS.to_owned();
    let ownership = json!({
        "owner":"injectra.preparation.v1",
        "platform":platform,
        "entry_strategy":"windows-v8-attach",
        "attached_pid":pid,
        "id":id,
        "app":app,
        "source_app":source,
        "data_mode":DataMode::Shared,
        "controller_origin":origin,
        "executable":exe
    });
    private_write(
        &root.join("ownership.json"),
        &serde_json::to_vec(&ownership).unwrap(),
    )?;
    private_write(
        &root.join("pairing.json"),
        &serde_json::to_vec(&json!({"artifact":id,"key":key})).unwrap(),
    )?;
    let artifact = PreparedArtifact {
        platform,
        entry_strategy: "windows-v8-attach".into(),
        attached_pid: Some(pid),
        version: 1,
        id,
        source_app: source,
        app,
        executable: exe,
        artifact_file: root.join("artifact.json"),
        controller_origin: origin,
        data_mode: DataMode::Shared,
        package_root: "AttachedRuntime".into(),
        main_entry: ".injectra/bootstrap.cjs".into(),
        baseline: BTreeMap::new(),
        changes: vec![json!({
            "path": ".injectra/bootstrap.cjs",
            "origin": "injectra",
            "reason": "Owned standalone Bootstrap runtime for verified Windows V8 attach",
            "before": Value::Null,
            "after": sha(injectra_bootstrap::BOOTSTRAP_SOURCE.as_bytes())
        })],
        excluded_unsealed: Vec::new(),
    };
    private_write(
        &artifact.artifact_file,
        &serde_json::to_vec_pretty(&artifact).unwrap(),
    )?;
    Ok((artifact, bootstrap_path))
}

impl PreparedArtifact {
    pub fn load(path: &Path) -> Result<Self> {
        let path = canonical_path(path)?;
        let artifact: Self =
            serde_json::from_slice(&fs::read(&path)?).map_err(|e| invalid(e.to_string()))?;
        if !paths_equal(&artifact.artifact_file, &path) || artifact.version != 1 {
            return Err(invalid("Invalid artifact identity/version"));
        }
        artifact.verify()?;
        Ok(artifact)
    }
    pub fn root(&self) -> &Path {
        self.artifact_file.parent().expect("verified artifact root")
    }
    pub fn is_windows_artifact(&self) -> bool {
        if !self.platform.is_empty() {
            self.platform == "windows"
        } else {
            self.app.file_name().and_then(|s| s.to_str()) == Some("PreparedApp")
        }
    }
    pub fn verify(&self) -> Result<()> {
        let root = self.artifact_file.parent().ok_or("Missing artifact root")?;
        let owner: Value = serde_json::from_slice(&fs::read(root.join("ownership.json"))?)
            .map_err(|e| invalid(e.to_string()))?;
        let owner_app = owner["app"].as_str().map(Path::new);
        let owner_source = owner["source_app"].as_str().map(Path::new);
        let owner_exe = owner["executable"].as_str().map(Path::new);
        let common_mismatch = owner["owner"] != "injectra.preparation.v1"
            || owner["id"] != self.id
            || owner_app.is_none_or(|p| !paths_equal(p, &self.app))
            || owner_source.is_none_or(|p| !paths_equal(p, &self.source_app))
            || owner["controller_origin"] != self.controller_origin
            || owner["data_mode"] != serde_json::to_value(self.data_mode).unwrap()
            || owner_exe.is_none_or(|p| !paths_equal(p, &self.executable))
            || paths_equal(&self.app, &self.source_app)
            || fs::symlink_metadata(&self.app)?.file_type().is_symlink();
        if common_mismatch {
            return Err(invalid("Artifact ownership mismatch"));
        }
        if self.entry_strategy == "windows-v8-attach" {
            if !paths_equal(&self.app, &root.join("AttachedRuntime"))
                || self.attached_pid.is_none()
                || owner["attached_pid"].as_u64() != self.attached_pid.map(u64::from)
            {
                return Err(invalid("Attached runtime artifact ownership mismatch"));
            }
        } else if self.is_windows_artifact() {
            if !paths_equal(&self.app, &root.join("PreparedApp"))
                || !path_starts_with(&self.executable, &self.app)
                || !self.executable.is_file()
            {
                return Err(invalid("Windows artifact layout mismatch"));
            }
        } else if !paths_equal(&self.app, &root.join("Prepared.app"))
            || self
                .executable
                .parent()
                .is_none_or(|p| !paths_equal(p, &self.app.join("Contents/MacOS")))
        {
            return Err(invalid("macOS artifact layout mismatch"));
        }
        if !self.controller_origin.starts_with("http://127.0.0.1:")
            || !self.controller_origin.ends_with('/')
            || self.controller_origin[17..self.controller_origin.len() - 1]
                .parse::<u16>()
                .ok()
                .filter(|p| *p > 0)
                .is_none()
        {
            return Err(invalid("Invalid loopback controller address"));
        }
        Ok(())
    }
    pub fn bridge_key(&self) -> Result<String> {
        self.verify()?;
        let pairing: Value = serde_json::from_slice(&fs::read(self.root().join("pairing.json"))?)
            .map_err(|e| invalid(e.to_string()))?;
        if pairing["artifact"] != self.id {
            return Err(invalid("Pairing mismatch"));
        }
        pairing["key"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| invalid("Missing pairing key"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attached_bootstrap_artifact_roundtrips_and_verifies_ownership() {
        let tmp = std::env::temp_dir().join(format!("injectra-attach-prep-{}", Uuid::new_v4()));
        let app_dir = tmp.join("app");
        let out_dir = tmp.join("out");
        fs::create_dir_all(&app_dir).unwrap();
        let fake_exe = app_dir.join("Host.exe");
        fs::write(&fake_exe, b"MZ").unwrap();

        let (artifact, bootstrap) = prepare_attached_bootstrap(&out_dir, &fake_exe, 4242).unwrap();
        assert_eq!(artifact.entry_strategy, "windows-v8-attach");
        assert_eq!(artifact.attached_pid, Some(4242));
        assert!(bootstrap.is_file());
        let loaded = PreparedArtifact::load(&artifact.artifact_file).unwrap();
        assert_eq!(loaded.id, artifact.id);
        assert_eq!(loaded.attached_pid, Some(4242));
        assert!(!loaded.bridge_key().unwrap().is_empty());

        let _ = fs::remove_dir_all(&tmp);
    }
}
