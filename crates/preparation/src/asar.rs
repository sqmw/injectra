//! Pure-Rust ASAR reader, directory packer, full stream rebuilder, and payload-preserving append rewriter (zero Node/npm dependency).
use injectra_core::{Error, ErrorKind, Result};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

const MAX_HEADER_BYTES: usize = 16 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 16 * 1024 * 1024;
const DEFAULT_BLOCK_SIZE: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct RawHeader {
    pub header_size: u64,
    pub header_json: String,
    pub header: Value,
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn file_integrity(bytes: &[u8], block_size: usize) -> Value {
    let mut blocks = Vec::new();
    let mut offset = 0usize;
    while offset + block_size <= bytes.len() {
        blocks.push(Value::String(sha256_hex(
            &bytes[offset..offset + block_size],
        )));
        offset += block_size;
    }
    blocks.push(Value::String(sha256_hex(&bytes[offset..])));
    json!({
        "algorithm": "SHA256",
        "hash": sha256_hex(bytes),
        "blockSize": block_size,
        "blocks": blocks,
    })
}

fn stream_copy_with_integrity<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    expected_len: u64,
    block_size: usize,
) -> Result<Value> {
    let mut file_hasher = Sha256::new();
    let mut block_hasher = Sha256::new();
    let mut blocks = Vec::new();
    let mut block_filled = 0usize;
    let mut remaining = expected_len;
    let mut buf = vec![0u8; 64 * 1024];
    while remaining > 0 {
        let want = (remaining as usize).min(buf.len());
        let n = reader.read(&mut buf[..want])?;
        if n == 0 {
            return Err(Error::prepare(
                ErrorKind::Io,
                "Unexpected EOF while streaming ASAR payload",
            ));
        }
        writer.write_all(&buf[..n])?;
        file_hasher.update(&buf[..n]);
        let mut slice = &buf[..n];
        while !slice.is_empty() {
            let take = (block_size - block_filled).min(slice.len());
            block_hasher.update(&slice[..take]);
            block_filled += take;
            slice = &slice[take..];
            if block_filled == block_size {
                let done = std::mem::take(&mut block_hasher);
                blocks.push(Value::String(format!("{:x}", done.finalize())));
                block_filled = 0;
            }
        }
        remaining -= n as u64;
    }
    blocks.push(Value::String(format!("{:x}", block_hasher.finalize())));
    Ok(json!({
        "algorithm": "SHA256",
        "hash": format!("{:x}", file_hasher.finalize()),
        "blockSize": block_size,
        "blocks": blocks,
    }))
}

fn hash_file_stream(path: &Path, block_size: usize) -> Result<(u64, Value)> {
    let meta = fs::metadata(path)?;
    let size = meta.len();
    let mut file = File::open(path)?;
    let mut sink = io::sink();
    let integrity = stream_copy_with_integrity(&mut file, &mut sink, size, block_size)?;
    Ok((size, integrity))
}

fn is_safe_relative(rel: &str) -> bool {
    !rel.is_empty()
        && !rel.starts_with('/')
        && !rel.contains('\\')
        && !rel.contains('\0')
        && rel
            .split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

pub fn unpacked_dir(asar_path: &Path) -> PathBuf {
    let mut name = asar_path.as_os_str().to_os_string();
    name.push(".unpacked");
    PathBuf::from(name)
}

fn encode_pickle_header(header_json: &str) -> Vec<u8> {
    let text = header_json.as_bytes();
    let pad = (4 - (text.len() % 4)) % 4;
    let string_obj_size = (4 + text.len() + pad) as u32;
    let header_size = string_obj_size + 4;
    let mut out = Vec::with_capacity(8 + header_size as usize);
    out.extend_from_slice(&4u32.to_le_bytes());
    out.extend_from_slice(&header_size.to_le_bytes());
    out.extend_from_slice(&string_obj_size.to_le_bytes());
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(text);
    out.resize(out.len() + pad, 0);
    out
}

pub fn read_header(asar_path: &Path) -> Result<RawHeader> {
    let mut file = File::open(asar_path)?;
    let mut prefix = [0u8; 16];
    file.read_exact(&mut prefix)?;
    let header_size = u64::from(u32::from_le_bytes(prefix[4..8].try_into().unwrap()));
    let json_len = u32::from_le_bytes(prefix[12..16].try_into().unwrap()) as usize;
    if json_len == 0 || json_len > MAX_HEADER_BYTES || header_size < (json_len as u64 + 8) {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "Invalid ASAR pickle header boundary",
        ));
    }
    let mut json_bytes = vec![0u8; json_len];
    file.read_exact(&mut json_bytes)?;
    let header_json = String::from_utf8(json_bytes)
        .map_err(|e| Error::prepare(ErrorKind::InvalidInput, e.to_string()).caused_by(e))?;
    let header: Value = serde_json::from_str(&header_json)
        .map_err(|e| Error::prepare(ErrorKind::InvalidInput, e.to_string()).caused_by(e))?;
    Ok(RawHeader {
        header_size,
        header_json,
        header,
    })
}

pub fn stat_entry(asar_path: &Path, rel_path: &str) -> Result<Option<Value>> {
    if !is_safe_relative(rel_path) {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "Unsafe relative ASAR entry path",
        ));
    }
    let raw = read_header(asar_path)?;
    let mut node = &raw.header;
    for part in rel_path.split('/') {
        let Some(next) = node.get("files").and_then(|f| f.get(part)) else {
            return Ok(None);
        };
        node = next;
    }
    Ok(Some(node.clone()))
}

pub fn extract_file(asar_path: &Path, rel_path: &str) -> Result<Vec<u8>> {
    if !is_safe_relative(rel_path) {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "Unsafe relative ASAR entry path",
        ));
    }
    let raw = read_header(asar_path)?;
    let mut node = &raw.header;
    for part in rel_path.split('/') {
        node = node
            .get("files")
            .and_then(|f| f.get(part))
            .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "ASAR entry not found"))?;
    }
    if node.get("files").is_some() || node.get("link").is_some() {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "ASAR entry is not a regular file",
        ));
    }
    let size = node
        .get("size")
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "Invalid ASAR entry size"))?;
    if size > MAX_ENTRY_BYTES {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "ASAR entry exceeds safety bound",
        ));
    }
    if node.get("unpacked").and_then(Value::as_bool) == Some(true) {
        let root = unpacked_dir(asar_path);
        let root_real = crate::canonical_path(&root)?;
        let candidate = crate::canonical_path(&root.join(rel_path))?;
        if !crate::path_starts_with(&candidate, &root_real) {
            return Err(Error::prepare(
                ErrorKind::Unsupported,
                "Unpacked ASAR entry resolves outside unpacked directory",
            ));
        }
        return Ok(fs::read(candidate)?);
    }
    let offset: u64 = node
        .get("offset")
        .and_then(Value::as_str)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "Invalid ASAR entry offset"))?;
    let data_start = 8u64
        .checked_add(raw.header_size)
        .and_then(|v| v.checked_add(offset))
        .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "ASAR offset overflow"))?;
    let mut file = File::open(asar_path)?;
    file.seek(SeekFrom::Start(data_start))?;
    let mut buf = vec![0u8; size as usize];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

fn validate_internal_symlink(root_real: &Path, link_path: &Path) -> Result<PathBuf> {
    let raw_target = fs::read_link(link_path)?;
    let resolved = crate::canonical_path(link_path)?;
    if !crate::path_starts_with(&resolved, root_real) {
        return Err(Error::prepare(
            ErrorKind::Unsupported,
            format!(
                "Symlink resolves outside archive root: {}",
                link_path.display()
            ),
        ));
    }
    Ok(raw_target)
}

fn copy_dir_preserving_links(src_root_real: &Path, src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let meta = fs::symlink_metadata(&from)?;
        if meta.file_type().is_symlink() {
            let raw_target = validate_internal_symlink(src_root_real, &from)?;
            #[cfg(unix)]
            {
                if fs::symlink_metadata(&to).is_ok() {
                    let _ = fs::remove_file(&to);
                }
                std::os::unix::fs::symlink(&raw_target, &to)?;
            }
            #[cfg(not(unix))]
            {
                let _ = raw_target;
                return Err(Error::prepare(
                    ErrorKind::Unsupported,
                    "Symlink inside .asar.unpacked is unsupported on this host",
                ));
            }
        } else if meta.is_dir() {
            copy_dir_preserving_links(src_root_real, &from, &to)?;
        } else {
            fs::copy(&from, &to)?;
            #[cfg(unix)]
            {
                fs::set_permissions(&to, meta.permissions())?;
            }
        }
    }
    Ok(())
}

fn is_executable_meta(meta: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        (meta.permissions().mode() & 0o111) != 0
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        false
    }
}

fn pack_dir_recursive(
    root_real: &Path,
    dir: &Path,
    payload_out: &mut File,
    next_offset: &mut u64,
) -> Result<Value> {
    let mut files_map = Map::new();
    let mut entries: Vec<fs::DirEntry> = fs::read_dir(dir)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry
            .file_name()
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| {
                Error::prepare(ErrorKind::InvalidInput, "Non-UTF8 path inside ASAR source")
            })?;
        let full = entry.path();
        let meta = fs::symlink_metadata(&full)?;
        if meta.file_type().is_symlink() {
            let _ = validate_internal_symlink(root_real, &full)?;
            let target_real = crate::canonical_path(&full)?;
            let rel = target_real
                .strip_prefix(root_real)
                .map_err(|_| {
                    Error::prepare(
                        ErrorKind::Unsupported,
                        "Symlink target resolves outside archive root",
                    )
                })?
                .to_string_lossy()
                .replace('\\', "/");
            if !is_safe_relative(&rel) {
                return Err(Error::prepare(
                    ErrorKind::Unsupported,
                    "Unsafe symlink target inside archive",
                ));
            }
            files_map.insert(name, json!({ "link": rel }));
        } else if meta.is_dir() {
            let child = pack_dir_recursive(root_real, &full, payload_out, next_offset)?;
            files_map.insert(name, child);
        } else if meta.is_file() {
            let size = meta.len();
            let mut src = File::open(&full)?;
            let integrity =
                stream_copy_with_integrity(&mut src, payload_out, size, DEFAULT_BLOCK_SIZE)?;
            let mut node = Map::new();
            node.insert("size".into(), json!(size));
            node.insert("offset".into(), Value::String(next_offset.to_string()));
            if is_executable_meta(&meta) {
                node.insert("executable".into(), Value::Bool(true));
            }
            node.insert("integrity".into(), integrity);
            *next_offset = next_offset
                .checked_add(size)
                .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "ASAR offset overflow"))?;
            files_map.insert(name, Value::Object(node));
        }
    }
    Ok(json!({ "files": files_map }))
}

/// Pack a directory into an ASAR archive with 4 MiB SHA-256 block integrity.
/// Returns the SHA-256 hex digest of the header JSON string.
pub fn create_package(src_dir: &Path, destination: &Path) -> Result<String> {
    let root_real = crate::canonical_path(src_dir)?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload_tmp = destination.with_extension("asar.payload.tmp");
    let mut payload_file = File::create(&payload_tmp)?;
    let mut next_offset = 0u64;
    let pack_res = pack_dir_recursive(&root_real, &root_real, &mut payload_file, &mut next_offset);
    if let Err(err) = pack_res {
        let _ = fs::remove_file(&payload_tmp);
        return Err(err);
    }
    let header = pack_res?;
    payload_file.flush()?;
    drop(payload_file);

    let header_json = serde_json::to_string(&header)
        .map_err(|e| Error::prepare(ErrorKind::InvalidInput, e.to_string()).caused_by(e))?;
    let header_bytes = encode_pickle_header(&header_json);
    let mut out = File::create(destination)?;
    out.write_all(&header_bytes)?;
    let mut payload_in = File::open(&payload_tmp)?;
    io::copy(&mut payload_in, &mut out)?;
    out.sync_all()?;
    let _ = fs::remove_file(&payload_tmp);
    Ok(sha256_hex(header_json.as_bytes()))
}

struct RebuildCtx<'a> {
    src_file: File,
    src_payload_start: u64,
    src_unpacked: PathBuf,
    dst_unpacked: PathBuf,
    payload_out: File,
    next_offset: u64,
    replacements: &'a BTreeMap<String, Vec<u8>>,
    consumed: BTreeSet<String>,
}

fn rebuild_node(prefix: &str, node: &Value, ctx: &mut RebuildCtx<'_>) -> Result<Value> {
    let Some(obj) = node.as_object() else {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "Invalid ASAR header node",
        ));
    };
    if let Some(files) = obj.get("files").and_then(Value::as_object) {
        let mut next_files = Map::new();
        for (name, child) in files {
            let rel = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            if !is_safe_relative(&rel) {
                return Err(Error::prepare(
                    ErrorKind::Unsupported,
                    format!("Unsupported: unsafe archive path: {rel}"),
                ));
            }
            next_files.insert(name.clone(), rebuild_node(&rel, child, ctx)?);
        }
        let mut out = Map::new();
        out.insert("files".into(), Value::Object(next_files));
        if obj.get("unpacked").and_then(Value::as_bool) == Some(true) {
            out.insert("unpacked".into(), Value::Bool(true));
        }
        return Ok(Value::Object(out));
    }

    if let Some(link) = obj.get("link").and_then(Value::as_str) {
        if !is_safe_relative(link) {
            return Err(Error::prepare(
                ErrorKind::Unsupported,
                format!("Unsupported: unsafe archive symlink: {link}"),
            ));
        }
        let mut out = Map::new();
        out.insert("link".into(), Value::String(link.to_owned()));
        if obj.get("unpacked").and_then(Value::as_bool) == Some(true) {
            out.insert("unpacked".into(), Value::Bool(true));
        }
        return Ok(Value::Object(out));
    }

    let is_unpacked = obj.get("unpacked").and_then(Value::as_bool) == Some(true);
    let is_exec = obj.get("executable").and_then(Value::as_bool) == Some(true);
    let block_size = obj
        .get("integrity")
        .and_then(|v| v.get("blockSize"))
        .and_then(Value::as_u64)
        .map(|n| n as usize)
        .unwrap_or(DEFAULT_BLOCK_SIZE);
    if block_size == 0 {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "Invalid ASAR integrity block size",
        ));
    }

    let mut out = Map::new();
    if is_exec {
        out.insert("executable".into(), Value::Bool(true));
    }
    if is_unpacked {
        out.insert("unpacked".into(), Value::Bool(true));
        let dst_path = ctx.dst_unpacked.join(prefix);
        if let Some(bytes) = ctx.replacements.get(prefix) {
            ctx.consumed.insert(prefix.to_owned());
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dst_path, bytes)?;
            out.insert("size".into(), json!(bytes.len()));
            out.insert("integrity".into(), file_integrity(bytes, block_size));
        } else {
            let src_path = ctx.src_unpacked.join(prefix);
            let (size, integrity) = hash_file_stream(&src_path, block_size)?;
            out.insert("size".into(), json!(size));
            out.insert("integrity".into(), integrity);
        }
        return Ok(Value::Object(out));
    }

    if let Some(bytes) = ctx.replacements.get(prefix) {
        ctx.consumed.insert(prefix.to_owned());
        ctx.payload_out.write_all(bytes)?;
        out.insert("size".into(), json!(bytes.len()));
        out.insert("offset".into(), Value::String(ctx.next_offset.to_string()));
        out.insert("integrity".into(), file_integrity(bytes, block_size));
        ctx.next_offset = ctx
            .next_offset
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "ASAR offset overflow"))?;
        return Ok(Value::Object(out));
    }

    let size = obj
        .get("size")
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "Invalid ASAR entry size"))?;
    let offset: u64 = obj
        .get("offset")
        .and_then(Value::as_str)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "Invalid ASAR entry offset"))?;
    let data_start = ctx
        .src_payload_start
        .checked_add(offset)
        .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "ASAR offset overflow"))?;
    ctx.src_file.seek(SeekFrom::Start(data_start))?;
    let integrity =
        stream_copy_with_integrity(&mut ctx.src_file, &mut ctx.payload_out, size, block_size)?;
    out.insert("size".into(), json!(size));
    out.insert("offset".into(), Value::String(ctx.next_offset.to_string()));
    out.insert("integrity".into(), integrity);
    ctx.next_offset = ctx
        .next_offset
        .checked_add(size)
        .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "ASAR offset overflow"))?;
    Ok(Value::Object(out))
}

fn verify_preserved_flags(prefix: &str, orig: &Value, rebuilt: &Value) -> Result<()> {
    let Some(orig_obj) = orig.as_object() else {
        return Ok(());
    };
    let Some(rebuilt_obj) = rebuilt.as_object() else {
        return Err(Error::prepare(
            ErrorKind::Io,
            format!("Rebuilt ASAR node missing: {prefix}"),
        ));
    };
    for flag in ["unpacked", "link", "executable"] {
        if orig_obj.get(flag) != rebuilt_obj.get(flag) {
            return Err(Error::prepare(
                ErrorKind::Io,
                format!("Archive state changed: {prefix}: {flag}"),
            ));
        }
    }
    if let Some(orig_files) = orig_obj.get("files").and_then(Value::as_object) {
        let Some(rebuilt_files) = rebuilt_obj.get("files").and_then(Value::as_object) else {
            return Err(Error::prepare(
                ErrorKind::Io,
                format!("Archive directory lost: {prefix}"),
            ));
        };
        for (name, child) in orig_files {
            let rel = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            let got = rebuilt_files.get(name).ok_or_else(|| {
                Error::prepare(ErrorKind::Io, format!("Archive entry missing: {rel}"))
            })?;
            verify_preserved_flags(&rel, child, got)?;
        }
    }
    Ok(())
}

/// Full streaming ASAR rebuilder (used on macOS .app archives): preserves directories, symlinks,
/// unpacked trees, executable flags, and 4 MiB SHA-256 block integrity while replacing/inserting
/// declared owned entries. Returns the SHA-256 hex digest of the rebuilt header JSON string.
pub fn rebuild(
    source: &Path,
    destination: &Path,
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<String> {
    let raw = read_header(source)?;
    let src_payload_start = 8u64
        .checked_add(raw.header_size)
        .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "ASAR header overflow"))?;
    let src_unpacked = unpacked_dir(source);
    let dst_unpacked = unpacked_dir(destination);
    if src_unpacked.is_dir() {
        let src_unpacked_real = crate::canonical_path(&src_unpacked)?;
        copy_dir_preserving_links(&src_unpacked_real, &src_unpacked_real, &dst_unpacked)?;
    }

    let payload_tmp = destination.with_extension("asar.payload.tmp");
    let mut ctx = RebuildCtx {
        src_file: File::open(source)?,
        src_payload_start,
        src_unpacked,
        dst_unpacked,
        payload_out: File::create(&payload_tmp)?,
        next_offset: 0,
        replacements,
        consumed: BTreeSet::new(),
    };

    let rebuilt_res = rebuild_node("", &raw.header, &mut ctx);
    let mut next_header = match rebuilt_res {
        Ok(h) => h,
        Err(err) => {
            drop(ctx);
            let _ = fs::remove_file(&payload_tmp);
            return Err(err);
        }
    };

    // Insert new replacement files (creating missing parent directory nodes).
    for (name, bytes) in replacements {
        if ctx.consumed.contains(name) {
            continue;
        }
        if !is_safe_relative(name) {
            drop(ctx);
            let _ = fs::remove_file(&payload_tmp);
            return Err(Error::prepare(
                ErrorKind::InvalidInput,
                format!("Unsafe replacement path: {name}"),
            ));
        }
        let parts: Vec<&str> = name.split('/').collect();
        let mut cursor = &mut next_header;
        for dir_part in &parts[..parts.len() - 1] {
            let files_obj = cursor
                .as_object_mut()
                .ok_or_else(|| {
                    Error::prepare(ErrorKind::InvalidInput, "Invalid ASAR directory node")
                })?
                .entry("files")
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
                .ok_or_else(|| {
                    Error::prepare(ErrorKind::InvalidInput, "ASAR files field is not an object")
                })?;
            cursor = files_obj
                .entry((*dir_part).to_owned())
                .or_insert_with(|| json!({ "files": {} }));
            if cursor.get("files").is_none() || cursor.get("link").is_some() {
                drop(ctx);
                let _ = fs::remove_file(&payload_tmp);
                return Err(Error::prepare(
                    ErrorKind::InvalidInput,
                    "Replacement parent is not a directory",
                ));
            }
        }
        let files_obj = cursor
            .as_object_mut()
            .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "Invalid ASAR directory node"))?
            .entry("files")
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or_else(|| {
                Error::prepare(ErrorKind::InvalidInput, "ASAR files field is not an object")
            })?;
        let leaf = *parts.last().unwrap();
        ctx.payload_out.write_all(bytes)?;
        let node = json!({
            "size": bytes.len(),
            "offset": ctx.next_offset.to_string(),
            "integrity": file_integrity(bytes, DEFAULT_BLOCK_SIZE),
        });
        ctx.next_offset = ctx
            .next_offset
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "ASAR offset overflow"))?;
        files_obj.insert(leaf.to_owned(), node);
    }

    ctx.payload_out.flush()?;
    drop(ctx);

    let next_header_json = serde_json::to_string(&next_header)
        .map_err(|e| Error::prepare(ErrorKind::InvalidInput, e.to_string()).caused_by(e))?;
    let header_bytes = encode_pickle_header(&next_header_json);
    let mut out = File::create(destination)?;
    out.write_all(&header_bytes)?;
    let mut payload_in = File::open(&payload_tmp)?;
    io::copy(&mut payload_in, &mut out)?;
    out.sync_all()?;
    let _ = fs::remove_file(&payload_tmp);

    let verified = read_header(destination)?;
    verify_preserved_flags("", &raw.header, &verified.header)?;
    for (name, expected) in replacements {
        let actual = extract_file(destination, name)?;
        if sha256_hex(&actual) != sha256_hex(expected) {
            return Err(Error::prepare(
                ErrorKind::Io,
                format!("Rebuilt ASAR verification mismatch: {name}"),
            ));
        }
    }
    Ok(sha256_hex(next_header_json.as_bytes()))
}

/// Payload-preserving ASAR append rewriter (used on Windows archives): preserves original packed
/// payload bytes and opaque/virtual/missing-unpacked header records while computing SHA-256 block
/// integrity for all replaced/added files. Returns the SHA-256 hex digest of the new header JSON.
pub fn append_rewrite(
    source: &Path,
    destination: &Path,
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<String> {
    let raw = read_header(source)?;
    let source_len = fs::metadata(source)?.len();
    let payload_start = 8u64
        .checked_add(raw.header_size)
        .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "ASAR header overflow"))?;
    if source_len < payload_start {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "Invalid ASAR payload boundary",
        ));
    }
    let mut header = raw.header;
    let mut next_offset = source_len - payload_start;
    let mut appended: Vec<&[u8]> = Vec::new();

    let src_unpacked = unpacked_dir(source);
    let dst_unpacked = unpacked_dir(destination);
    if src_unpacked.is_dir() {
        let src_unpacked_real = crate::canonical_path(&src_unpacked)?;
        copy_dir_preserving_links(&src_unpacked_real, &src_unpacked_real, &dst_unpacked)?;
    }

    for (name, bytes) in replacements {
        if !is_safe_relative(name) {
            return Err(Error::prepare(
                ErrorKind::InvalidInput,
                format!("Unsafe replacement path: {name}"),
            ));
        }
        let parts: Vec<&str> = name.split('/').collect();
        let mut cursor = &mut header;
        for dir_part in &parts[..parts.len() - 1] {
            let files_obj = cursor
                .as_object_mut()
                .ok_or_else(|| {
                    Error::prepare(ErrorKind::InvalidInput, "Invalid ASAR directory node")
                })?
                .entry("files")
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
                .ok_or_else(|| {
                    Error::prepare(ErrorKind::InvalidInput, "ASAR files field is not an object")
                })?;
            cursor = files_obj
                .entry((*dir_part).to_owned())
                .or_insert_with(|| json!({"files": {}}));
            if cursor.get("files").is_none() || cursor.get("link").is_some() {
                return Err(Error::prepare(
                    ErrorKind::InvalidInput,
                    "Replacement parent is not a directory",
                ));
            }
        }
        let files_obj = cursor
            .as_object_mut()
            .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "Invalid ASAR directory node"))?
            .entry("files")
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or_else(|| {
                Error::prepare(ErrorKind::InvalidInput, "ASAR files field is not an object")
            })?;
        let leaf = *parts.last().unwrap();
        let mut node_obj = match files_obj.get(leaf) {
            Some(existing) => {
                if existing.get("files").is_some() || existing.get("link").is_some() {
                    return Err(Error::prepare(
                        ErrorKind::InvalidInput,
                        "Replacement target is not a regular file",
                    ));
                }
                existing.as_object().cloned().unwrap_or_default()
            }
            None => Map::new(),
        };

        let block_size = node_obj
            .get("integrity")
            .and_then(|v| v.get("blockSize"))
            .and_then(Value::as_u64)
            .map(|n| n as usize)
            .unwrap_or(DEFAULT_BLOCK_SIZE);
        if block_size == 0 {
            return Err(Error::prepare(
                ErrorKind::InvalidInput,
                "Invalid ASAR integrity block size",
            ));
        }
        node_obj.insert("size".into(), json!(bytes.len()));
        node_obj.insert("integrity".into(), file_integrity(bytes, block_size));

        if node_obj.get("unpacked").and_then(Value::as_bool) == Some(true) {
            let out_path = dst_unpacked.join(name);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(out_path, bytes)?;
        } else {
            node_obj.insert("offset".into(), Value::String(next_offset.to_string()));
            next_offset = next_offset
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| Error::prepare(ErrorKind::InvalidInput, "ASAR offset overflow"))?;
            appended.push(bytes.as_slice());
        }
        files_obj.insert(leaf.to_owned(), Value::Object(node_obj));
    }

    let next_header_json = serde_json::to_string(&header)
        .map_err(|e| Error::prepare(ErrorKind::InvalidInput, e.to_string()).caused_by(e))?;
    let header_bytes = encode_pickle_header(&next_header_json);

    let mut out = File::create(destination)?;
    out.write_all(&header_bytes)?;
    let mut src_file = File::open(source)?;
    src_file.seek(SeekFrom::Start(payload_start))?;
    io::copy(&mut src_file, &mut out)?;
    for chunk in appended {
        out.write_all(chunk)?;
    }
    out.sync_all()?;

    // Verify all replaced entries read back with exact SHA-256 match.
    for (name, expected) in replacements {
        let actual = extract_file(destination, name)?;
        if sha256_hex(&actual) != sha256_hex(expected) {
            return Err(Error::prepare(
                ErrorKind::Io,
                format!("Appended ASAR verification mismatch: {name}"),
            ));
        }
    }
    Ok(sha256_hex(next_header_json.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn write_synthetic_asar(path: &Path, header: &Value, payload: &[u8]) {
        let json_str = serde_json::to_string(header).unwrap();
        let mut out = encode_pickle_header(&json_str);
        out.extend_from_slice(payload);
        fs::write(path, out).unwrap();
    }

    #[test]
    fn append_rewrite_preserves_original_payload_opaque_records_and_updates_integrity() {
        let tmp = std::env::temp_dir().join(format!("injectra-rust-asar-{}", Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let src = tmp.join("before.asar");
        let dst = tmp.join("after.asar");
        let header = json!({
            "files": {
                "main.js": { "size": 3, "offset": "0" },
                "virtual": { "size": -1000, "offset": "0" },
                "missing": { "size": 10, "unpacked": true }
            }
        });
        write_synthetic_asar(&src, &header, b"abc");

        let mut replacements = BTreeMap::new();
        replacements.insert("main.js".into(), b"updated".to_vec());
        replacements.insert(".injectra/bootstrap.cjs".into(), b"owned".to_vec());

        let digest = append_rewrite(&src, &dst, &replacements).unwrap();
        let after = read_header(&dst).unwrap();
        assert_eq!(digest, sha256_hex(after.header_json.as_bytes()));
        assert_eq!(after.header["files"]["virtual"], header["files"]["virtual"]);
        assert_eq!(after.header["files"]["missing"], header["files"]["missing"]);
        assert_eq!(extract_file(&dst, "main.js").unwrap(), b"updated");
        assert_eq!(
            extract_file(&dst, ".injectra/bootstrap.cjs").unwrap(),
            b"owned"
        );
        assert_eq!(extract_file(&src, "main.js").unwrap(), b"abc");

        let dst_bytes = fs::read(&dst).unwrap();
        let start = 8 + after.header_size as usize;
        assert_eq!(&dst_bytes[start..start + 3], b"abc");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn create_package_and_rebuild_preserve_symlinks_executable_unpacked_and_zero_byte_integrity() {
        let tmp = std::env::temp_dir().join(format!("injectra-rust-rebuild-{}", Uuid::new_v4()));
        let src_dir = tmp.join("src");
        fs::create_dir_all(src_dir.join("bin")).unwrap();
        fs::write(src_dir.join("main.js"), b"console.log('orig');").unwrap();
        fs::write(src_dir.join("empty.txt"), b"").unwrap();
        let tool = src_dir.join("bin/tool.sh");
        fs::write(&tool, b"#!/bin/sh\necho ok\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).unwrap();
            std::os::unix::fs::symlink("../main.js", src_dir.join("bin/link.js")).unwrap();
        }

        let packed = tmp.join("orig.asar");
        let pack_digest = create_package(&src_dir, &packed).unwrap();
        let raw = read_header(&packed).unwrap();
        assert_eq!(pack_digest, sha256_hex(raw.header_json.as_bytes()));
        assert_eq!(
            raw.header["files"]["empty.txt"]["integrity"]["blocks"],
            json!([sha256_hex(b"")])
        );
        #[cfg(unix)]
        {
            assert_eq!(
                stat_entry(&packed, "bin/tool.sh").unwrap().unwrap()["executable"],
                json!(true)
            );
            assert_eq!(
                stat_entry(&packed, "bin/link.js").unwrap().unwrap()["link"],
                json!("main.js")
            );
        }

        // Also test unpacked file retention through rebuild
        let unpacked_src = tmp.join("with_unpacked.asar");
        fs::create_dir_all(unpacked_dir(&unpacked_src).join("native")).unwrap();
        fs::write(
            unpacked_dir(&unpacked_src).join("native/addon.bin"),
            b"native-bytes",
        )
        .unwrap();
        let mut header_with_unpacked = raw.header.clone();
        header_with_unpacked["files"]["native"] = json!({
            "unpacked": true,
            "files": {
                "addon.bin": { "size": 12, "unpacked": true }
            }
        });
        let orig_bytes = fs::read(&packed).unwrap();
        let orig_payload = &orig_bytes[(8 + raw.header_size as usize)..];
        write_synthetic_asar(&unpacked_src, &header_with_unpacked, orig_payload);

        let rebuilt = tmp.join("rebuilt.asar");
        let mut replacements = BTreeMap::new();
        replacements.insert("main.js".into(), b"console.log('patched');".to_vec());
        replacements.insert(".injectra/bootstrap.cjs".into(), b"boot".to_vec());

        let rebuilt_digest = rebuild(&unpacked_src, &rebuilt, &replacements).unwrap();
        let rebuilt_raw = read_header(&rebuilt).unwrap();
        assert_eq!(
            rebuilt_digest,
            sha256_hex(rebuilt_raw.header_json.as_bytes())
        );
        assert_eq!(
            extract_file(&rebuilt, "main.js").unwrap(),
            b"console.log('patched');"
        );
        assert_eq!(extract_file(&rebuilt, "empty.txt").unwrap(), b"");
        assert_eq!(
            extract_file(&rebuilt, ".injectra/bootstrap.cjs").unwrap(),
            b"boot"
        );
        assert_eq!(
            extract_file(&rebuilt, "native/addon.bin").unwrap(),
            b"native-bytes"
        );
        #[cfg(unix)]
        {
            assert_eq!(
                stat_entry(&rebuilt, "bin/tool.sh").unwrap().unwrap()["executable"],
                json!(true)
            );
            assert_eq!(
                stat_entry(&rebuilt, "bin/link.js").unwrap().unwrap()["link"],
                json!("main.js")
            );
        }

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn rebuild_rejects_unsafe_paths_and_escaping_symlinks() {
        let tmp = std::env::temp_dir().join(format!("injectra-rust-unsafe-{}", Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let bad_asar = tmp.join("bad.asar");
        write_synthetic_asar(
            &bad_asar,
            &json!({ "files": { "escape": { "link": "../outside" } } }),
            b"",
        );
        let err = rebuild(&bad_asar, &tmp.join("out.asar"), &BTreeMap::new()).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Unsupported);
        let _ = fs::remove_dir_all(&tmp);
    }
}
