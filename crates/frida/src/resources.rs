use injectra_core::{Error, ErrorKind, Result};
use std::{fs, path::PathBuf};
const COMMON: &[(&str, &str)] = &[
    ("main.py", include_str!("../helper/main.py")),
    ("connection.py", include_str!("../helper/connection.py")),
    ("errors.py", include_str!("../helper/errors.py")),
    (
        "session_model.py",
        include_str!("../helper/session_model.py"),
    ),
    (
        "session_store.py",
        include_str!("../helper/session_store.py"),
    ),
    ("contracts.py", include_str!("../helper/contracts.py")),
    ("transport.py", include_str!("../helper/transport.py")),
    ("targets.py", include_str!("../helper/targets.py")),
    ("target_macos.py", include_str!("../helper/target_macos.py")),
    (
        "target_macos_identity.js",
        include_str!("../helper/target_macos_identity.js"),
    ),
    ("deadline.py", include_str!("../helper/deadline.py")),
];
pub(crate) fn cache() -> Result<PathBuf> {
    if let Some(value) = std::env::var_os("INJECTRA_CACHE_DIR") {
        return Ok(value.into());
    }
    let root = if let Some(value) = std::env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(value)
    } else {
        PathBuf::from(std::env::var_os("HOME").ok_or("HOME unavailable")?).join(".cache")
    };
    Ok(root.join("injectra"))
}
pub(crate) fn materialize(extra: &[(&str, &str)]) -> Result<PathBuf> {
    let mut hash = 0xcbf29ce484222325u64;
    for (name, source) in COMMON.iter().chain(extra.iter()) {
        for byte in name.bytes().chain(source.bytes()) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    let root = cache()?.join("frida/helpers").join(format!("{hash:016x}"));
    fs::create_dir_all(&root)?;
    for (name, source) in COMMON.iter().chain(extra.iter()) {
        if name.contains('/') || name.contains('\\') || name.starts_with('.') {
            return Err(Error::prepare(
                ErrorKind::InvalidInput,
                "invalid embedded helper resource",
            ));
        }
        let path = root.join(name);
        if path.exists() {
            if fs::read_to_string(&path)? != *source {
                return Err(Error::prepare(
                    ErrorKind::InvalidInput,
                    "Cached helper differs from embedded source; use a fresh cache",
                ));
            }
        } else {
            fs::write(&path, source)?;
        }
    }
    Ok(root)
}
