use crate::{
    error::{ApiError, Result},
    security::random_id,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct EndpointRecord {
    pub schema: u32,
    pub workspace_root: PathBuf,
    pub port: u16,
    pub token: String,
}
impl EndpointRecord {
    pub fn origin(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
    pub fn launch_url(&self) -> String {
        format!("{}/#token={}", self.origin(), self.token)
    }
}
pub struct EndpointLease {
    pub listener: TcpListener,
    pub record: EndpointRecord,
    pub directory: PathBuf,
    _lock: File,
}

pub fn default_cache() -> PathBuf {
    if let Some(path) = std::env::var_os("INJECTRA_CACHE_DIR") {
        return PathBuf::from(path);
    }
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join(".cache")
        });
    base.join("injectra")
}
pub fn directory(cache: &Path, workspace: &Path) -> Result<PathBuf> {
    let root = workspace.canonicalize()?;
    let key = format!("{:x}", Sha256::digest(root.to_string_lossy().as_bytes()));
    Ok(cache.join("studio").join(key))
}
fn private_dir(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if !meta.is_dir() || meta.file_type().is_symlink() {
                return Err(ApiError::invalid(
                    "Studio cache directory must not be a symlink",
                ));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if meta.uid() != rustix::process::geteuid().as_raw() || meta.mode() & 0o077 != 0 {
                    return Err(ApiError::invalid(
                        "Studio cache directory must be private (0700)",
                    ));
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(path)?;
        }
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
fn private_open(path: &Path, create: bool) -> Result<File> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(create)
        .create(create)
        .truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() {
        return Err(ApiError::invalid("Expected a regular private Studio file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.uid() != rustix::process::geteuid().as_raw() || meta.mode() & 0o077 != 0 {
            return Err(ApiError::invalid("Studio metadata must be private (0600)"));
        }
    }
    Ok(file)
}
pub fn read_record(cache: &Path, workspace: &Path) -> Result<EndpointRecord> {
    let root = workspace.canonicalize()?;
    let path = directory(cache, &root)?.join("ready.json");
    let file = private_open(&path, false)?;
    if file.metadata()?.len() > 16384 {
        return Err(ApiError::invalid("Invalid Studio endpoint metadata size"));
    }
    let record: EndpointRecord = serde_json::from_reader(file)?;
    if record.schema != 1 || record.workspace_root != root || record.port == 0 {
        return Err(ApiError::invalid("Studio endpoint identity mismatch"));
    }
    Ok(record)
}
pub fn bind(cache: &Path, workspace: &Path) -> Result<EndpointLease> {
    fs::create_dir_all(cache)?;
    let cache = cache.canonicalize()?;
    let root = workspace.canonicalize()?;
    let parent = cache.join("studio");
    private_dir(&parent)?;
    let directory = directory(&cache, &root)?;
    private_dir(&directory)?;
    let lock = private_open(&directory.join("service.lock"), true)?;
    fs2::FileExt::try_lock_exclusive(&lock)
        .map_err(|_| ApiError::conflict("Studio is already running for this workspace"))?;
    let saved = directory.join("ready.json");
    let existing = if saved.exists() {
        Some(read_record(&cache, &root)?)
    } else {
        None
    };
    let listener = TcpListener::bind(SocketAddrV4::new(
        Ipv4Addr::LOCALHOST,
        existing.as_ref().map_or(0, |r| r.port),
    ))
    .map_err(|e| ApiError::conflict(format!("Persisted Studio address unavailable: {e}")))?;
    listener.set_nonblocking(true)?;
    let record = match existing {
        Some(record) => record,
        None => EndpointRecord {
            schema: 1,
            workspace_root: root,
            port: listener.local_addr()?.port(),
            token: random_id()?,
        },
    };
    if !saved.exists() {
        write_private_json(&saved, &record)?;
    }
    private_dir(&directory.join("runs"))?;
    Ok(EndpointLease {
        listener,
        record,
        directory,
        _lock: lock,
    })
}
pub fn write_private_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| ApiError::invalid("Missing parent"))?;
    let temp = parent.join(format!(".pending-{}", random_id()?));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options.open(&temp)?;
    let data = serde_json::to_vec_pretty(value)?;
    file.write_all(&data)?;
    file.sync_all()?;
    if fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        fs::remove_file(temp)?;
        return Err(ApiError::invalid("Refusing symlink metadata destination"));
    }
    fs::rename(temp, path)?;
    Ok(())
}
