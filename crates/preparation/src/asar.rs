//! Pure-Rust ASAR reader and payload-preserving append rewriter (zero Node/npm dependency).
use injectra_core::{Error, ErrorKind, Result};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
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

fn is_safe_relative(rel: &str) -> bool {
    !rel.is_empty()
        && !rel.starts_with('/')
        && !rel.contains('\\')
        && !rel.contains('\0')
        && rel
            .split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

fn unpacked_dir(asar_path: &Path) -> PathBuf {
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

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let meta = fs::symlink_metadata(&from)?;
        if meta.file_type().is_symlink() {
            return Err(Error::prepare(
                ErrorKind::Unsupported,
                "Symlink inside .asar.unpacked is unsupported in pure-Rust append_rewrite",
            ));
        }
        if meta.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Payload-preserving ASAR append rewriter: preserves original packed payload bytes and
/// opaque/virtual/missing-unpacked header records while computing SHA-256 block integrity
/// for all replaced/added files. Returns the SHA-256 hex digest of the new header JSON string.
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
        copy_dir_recursive(&src_unpacked, &dst_unpacked)?;
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
        let mut blocks = Vec::new();
        if !bytes.is_empty() {
            for chunk in bytes.chunks(block_size) {
                blocks.push(Value::String(sha256_hex(chunk)));
            }
        }
        node_obj.insert("size".into(), json!(bytes.len()));
        node_obj.insert(
            "integrity".into(),
            json!({
                "algorithm": "SHA256",
                "hash": sha256_hex(bytes),
                "blockSize": block_size,
                "blocks": blocks,
            }),
        );

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
        let raw = read_header(&dst).unwrap();
        assert_eq!(digest, sha256_hex(raw.header_json.as_bytes()));
        assert_eq!(raw.header["files"]["virtual"], header["files"]["virtual"]);
        assert_eq!(raw.header["files"]["missing"], header["files"]["missing"]);
        assert_eq!(extract_file(&dst, "main.js").unwrap(), b"updated");
        assert_eq!(
            extract_file(&dst, ".injectra/bootstrap.cjs").unwrap(),
            b"owned"
        );
        assert_eq!(extract_file(&src, "main.js").unwrap(), b"abc");

        // Original packed payload bytes stay intact at offset 0.
        let dst_bytes = fs::read(&dst).unwrap();
        let payload_start = 8 + raw.header_size as usize;
        assert_eq!(&dst_bytes[payload_start..payload_start + 3], b"abc");

        // Unsafe path is rejected before mutation.
        let mut bad = BTreeMap::new();
        bad.insert("../escape.js".into(), b"x".to_vec());
        assert_eq!(
            append_rewrite(&src, &tmp.join("bad.asar"), &bad)
                .unwrap_err()
                .kind,
            ErrorKind::InvalidInput
        );

        let _ = fs::remove_dir_all(&tmp);
    }
}
