//! Generic Electron CJS dev-entry preparation; only a new owned copy is mutated.
use injectra_core::{Error, ErrorKind, Result};
pub use launch::launch;
pub use model::*;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
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
    let root = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".cache")))
        .ok_or_else(|| invalid("No cache location"))?;
    Ok(root.join("injectra"))
}
pub(crate) fn helper() -> Result<PathBuf> {
    let sources = [
        ("prepare.mjs", include_str!("../helper/prepare.mjs")),
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
fn run<T: DeserializeOwned>(request: Value) -> Result<T> {
    if std::env::consts::OS != "macos" || std::env::consts::ARCH != "aarch64" {
        return Err(Error::prepare(
            ErrorKind::Unsupported,
            "Preparation supports macOS Apple Silicon only",
        ));
    }
    let tools = std::env::var_os("INJECTRA_PREPARATION_TOOLS")
        .map(PathBuf::from)
        .unwrap_or(cache()?.join("preparation-tools"));
    if !tools
        .join("node_modules/@electron/asar/package.json")
        .is_file()
        || !tools.join("node_modules/acorn/package.json").is_file()
    {
        return Err(Error::prepare(
            ErrorKind::Unsupported,
            "Preparation tools are missing; run make setup-preparation",
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
    let source = fs::canonicalize(&request.source_app)?;
    if request.output_root.starts_with(&source) {
        return Err(invalid("Output is inside source application"));
    }
    private_dir(&request.output_root)?;
    let output = fs::canonicalize(&request.output_root)?;
    if output.starts_with(&source) {
        return Err(invalid("Output is inside source application"));
    }
    let id = Uuid::new_v4().to_string();
    let root = output.join(&id);
    private_dir(&root)?;
    let app = root.join("Prepared.app");
    // Select once, persist in signed config; dev must report conflicts rather than switch ports.
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let origin = format!("http://{}/", listener.local_addr()?);
    let key = injectra_bootstrap::random_token();
    let executable = app.join("Contents/MacOS").join(
        plan.probe
            .executable
            .file_name()
            .ok_or("Missing executable")?,
    );
    let ownership = json!({"owner":"injectra.preparation.v1","id":id,"app":app,"source_app":source,"data_mode":request.data_mode,"controller_origin":origin,"executable":executable});
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
impl PreparedArtifact {
    pub fn load(path: &Path) -> Result<Self> {
        let path = path.canonicalize()?;
        let artifact: Self =
            serde_json::from_slice(&fs::read(&path)?).map_err(|e| invalid(e.to_string()))?;
        if artifact.artifact_file != path || artifact.version != 1 {
            return Err(invalid("Invalid artifact identity/version"));
        }
        artifact.verify()?;
        Ok(artifact)
    }
    pub fn root(&self) -> &Path {
        self.artifact_file.parent().expect("verified artifact root")
    }
    pub fn verify(&self) -> Result<()> {
        let root = self.artifact_file.parent().ok_or("Missing artifact root")?;
        let owner: Value = serde_json::from_slice(&fs::read(root.join("ownership.json"))?)
            .map_err(|e| invalid(e.to_string()))?;
        if owner["owner"] != "injectra.preparation.v1"
            || owner["id"] != self.id
            || owner["app"].as_str() != self.app.to_str()
            || owner["source_app"].as_str() != self.source_app.to_str()
            || owner["controller_origin"] != self.controller_origin
            || owner["data_mode"] != serde_json::to_value(self.data_mode).unwrap()
            || owner["executable"].as_str() != self.executable.to_str()
            || self.app != root.join("Prepared.app")
            || self.app == self.source_app
            || self.executable.parent() != Some(self.app.join("Contents/MacOS").as_path())
            || fs::symlink_metadata(&self.app)?.file_type().is_symlink()
        {
            return Err(invalid("Artifact ownership mismatch"));
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
