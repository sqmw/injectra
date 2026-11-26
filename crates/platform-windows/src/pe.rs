//! Pure-Rust read-only PE architecture and Electron INTEGRITY/ELECTRONASAR resource reader.
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

const MAX_RSRC_BYTES: usize = 16 * 1024 * 1024;
const MAX_INTEGRITY_JSON_BYTES: usize = 1024 * 1024;
const MAX_SECTIONS: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeArch {
    X86,
    X64,
    Arm64,
}

impl PeArch {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X86 => "x86",
            Self::X64 => "x64",
            Self::Arm64 => "arm64",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeIntegrityRecord {
    pub file: String,
    pub alg: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeSummary {
    pub arch: PeArch,
    pub asar_integrity: Vec<PeIntegrityRecord>,
}

#[derive(Clone, Copy)]
struct SectionHeader {
    virtual_size: u32,
    virtual_address: u32,
    raw_size: u32,
    raw_offset: u32,
}

fn u16_le(bytes: &[u8], off: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(off..off + 2)?.try_into().ok()?,
    ))
}

fn u32_le(bytes: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(off..off + 4)?.try_into().ok()?,
    ))
}

fn rva_to_file_offset(sections: &[SectionHeader], rva: u32) -> Option<(u64, usize)> {
    for sec in sections {
        let span = sec.virtual_size.max(sec.raw_size);
        let end = sec.virtual_address.checked_add(span)?;
        if rva >= sec.virtual_address && rva < end {
            let delta = rva - sec.virtual_address;
            if delta >= sec.raw_size {
                return None;
            }
            let file_off = u64::from(sec.raw_offset).checked_add(u64::from(delta))?;
            let avail = (sec.raw_size - delta) as usize;
            return Some((file_off, avail));
        }
    }
    None
}

fn read_utf16_dir_name(rsrc: &[u8], offset: usize) -> Option<String> {
    let char_len = usize::from(u16_le(rsrc, offset)?);
    if char_len == 0 || char_len > 256 {
        return None;
    }
    let start = offset.checked_add(2)?;
    let byte_len = char_len.checked_mul(2)?;
    let slice = rsrc.get(start..start.checked_add(byte_len)?)?;
    let units: Vec<u16> = slice
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&units).ok()
}

fn find_named_subdir(rsrc: &[u8], dir_off: usize, expected_upper: &str) -> Option<usize> {
    let named_count = usize::from(u16_le(rsrc, dir_off.checked_add(12)?)?);
    let entries_start = dir_off.checked_add(16)?;
    for idx in 0..named_count {
        let entry_off = entries_start.checked_add(idx.checked_mul(8)?)?;
        let name_field = u32_le(rsrc, entry_off)?;
        let data_field = u32_le(rsrc, entry_off + 4)?;
        if (name_field & 0x8000_0000) == 0 || (data_field & 0x8000_0000) == 0 {
            continue;
        }
        let name_off = (name_field & 0x7fff_ffff) as usize;
        if let Some(name) = read_utf16_dir_name(rsrc, name_off)
            && name.eq_ignore_ascii_case(expected_upper)
        {
            return Some((data_field & 0x7fff_ffff) as usize);
        }
    }
    None
}

fn find_first_leaf_data_entry(rsrc: &[u8], dir_off: usize) -> Option<(u32, u32)> {
    let named_count = usize::from(u16_le(rsrc, dir_off.checked_add(12)?)?);
    let id_count = usize::from(u16_le(rsrc, dir_off.checked_add(14)?)?);
    let total = named_count.checked_add(id_count)?;
    let entries_start = dir_off.checked_add(16)?;
    for idx in 0..total {
        let entry_off = entries_start.checked_add(idx.checked_mul(8)?)?;
        let data_field = u32_le(rsrc, entry_off + 4)?;
        if (data_field & 0x8000_0000) != 0 {
            continue;
        }
        let leaf_off = data_field as usize;
        let data_rva = u32_le(rsrc, leaf_off)?;
        let data_size = u32_le(rsrc, leaf_off.checked_add(4)?)?;
        return Some((data_rva, data_size));
    }
    None
}

fn parse_integrity_json(bytes: &[u8]) -> Vec<PeIntegrityRecord> {
    let trimmed = match bytes.iter().rposition(|&b| b != 0) {
        Some(last) => &bytes[..=last],
        None => return Vec::new(),
    };
    serde_json::from_slice::<Vec<PeIntegrityRecord>>(trimmed).unwrap_or_default()
}

pub fn inspect_pe(path: &Path) -> Option<PeSummary> {
    let mut file = File::open(path).ok()?;
    let mut dos = [0u8; 64];
    file.read_exact(&mut dos).ok()?;
    if &dos[0..2] != b"MZ" {
        return None;
    }
    let pe_off = u64::from(u32_le(&dos, 0x3c)?);
    file.seek(SeekFrom::Start(pe_off)).ok()?;
    let mut pe_head = [0u8; 24];
    file.read_exact(&mut pe_head).ok()?;
    if &pe_head[0..4] != b"PE\0\0" {
        return None;
    }
    let machine = u16_le(&pe_head, 4)?;
    let arch = match machine {
        0x014c => PeArch::X86,
        0x8664 => PeArch::X64,
        0xaa64 => PeArch::Arm64,
        _ => return None,
    };
    let num_sections = usize::from(u16_le(&pe_head, 6)?).min(MAX_SECTIONS);
    let opt_size = usize::from(u16_le(&pe_head, 20)?);
    if opt_size > 4096 {
        return Some(PeSummary {
            arch,
            asar_integrity: Vec::new(),
        });
    }
    let mut opt = vec![0u8; opt_size];
    file.read_exact(&mut opt).ok()?;
    let mut sec_buf = vec![0u8; num_sections * 40];
    file.read_exact(&mut sec_buf).ok()?;
    let mut sections = Vec::with_capacity(num_sections);
    for chunk in sec_buf.chunks_exact(40) {
        sections.push(SectionHeader {
            virtual_size: u32_le(chunk, 8)?,
            virtual_address: u32_le(chunk, 12)?,
            raw_size: u32_le(chunk, 16)?,
            raw_offset: u32_le(chunk, 20)?,
        });
    }

    let magic = u16_le(&opt, 0).unwrap_or(0);
    let data_dir_off = match magic {
        0x10b => 96usize,
        0x20b => 112usize,
        _ => {
            return Some(PeSummary {
                arch,
                asar_integrity: Vec::new(),
            });
        }
    };
    let num_dirs = u32_le(&opt, data_dir_off.checked_sub(4)?).unwrap_or(0);
    if num_dirs <= 2 {
        return Some(PeSummary {
            arch,
            asar_integrity: Vec::new(),
        });
    }
    let rsrc_dir_entry = data_dir_off + 2 * 8;
    let rsrc_rva = u32_le(&opt, rsrc_dir_entry).unwrap_or(0);
    let rsrc_size = u32_le(&opt, rsrc_dir_entry + 4).unwrap_or(0) as usize;
    if rsrc_rva == 0 || rsrc_size == 0 || rsrc_size > MAX_RSRC_BYTES {
        return Some(PeSummary {
            arch,
            asar_integrity: Vec::new(),
        });
    }
    let Some((rsrc_file_off, avail)) = rva_to_file_offset(&sections, rsrc_rva) else {
        return Some(PeSummary {
            arch,
            asar_integrity: Vec::new(),
        });
    };
    let read_len = rsrc_size.min(avail);
    let mut rsrc = vec![0u8; read_len];
    if file.seek(SeekFrom::Start(rsrc_file_off)).is_err() || file.read_exact(&mut rsrc).is_err() {
        return Some(PeSummary {
            arch,
            asar_integrity: Vec::new(),
        });
    }

    let asar_integrity = (|| -> Option<Vec<PeIntegrityRecord>> {
        let type_dir = find_named_subdir(&rsrc, 0, "INTEGRITY")?;
        let name_dir = find_named_subdir(&rsrc, type_dir, "ELECTRONASAR")?;
        let (data_rva, data_size) = find_first_leaf_data_entry(&rsrc, name_dir)?;
        let data_size = data_size as usize;
        if data_size == 0 || data_size > MAX_INTEGRITY_JSON_BYTES {
            return None;
        }
        let (data_file_off, data_avail) = rva_to_file_offset(&sections, data_rva)?;
        if data_size > data_avail {
            return None;
        }
        let mut payload = vec![0u8; data_size];
        file.seek(SeekFrom::Start(data_file_off)).ok()?;
        file.read_exact(&mut payload).ok()?;
        Some(parse_integrity_json(&payload))
    })()
    .unwrap_or_default();

    Some(PeSummary {
        arch,
        asar_integrity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn build_synthetic_pe(arch: PeArch, integrity_json: Option<&str>) -> Vec<u8> {
        let is_64 = arch != PeArch::X86;
        let machine: u16 = match arch {
            PeArch::X86 => 0x014c,
            PeArch::X64 => 0x8664,
            PeArch::Arm64 => 0xaa64,
        };
        let opt_size: u16 = if is_64 { 240 } else { 224 };
        let pe_off = 0x80usize;
        let rsrc_rva = 0x2000u32;
        let rsrc_raw_off = 0x400u32;

        let mut rsrc = Vec::new();
        if let Some(json) = integrity_json {
            // Layout inside .rsrc:
            // 0x00: Root dir (16) + 1 entry (8) = 24
            // 0x18: Type dir (16) + 1 entry (8) = 24
            // 0x30: Name dir (16) + 1 entry (8) = 24
            // 0x48: Leaf IMAGE_RESOURCE_DATA_ENTRY (16)
            // 0x58: UTF-16LE string "INTEGRITY" (2 + 18 = 20)
            // 0x6c: UTF-16LE string "ELECTRONASAR" (2 + 24 = 26)
            // 0x88: JSON payload bytes
            let str_integrity_off = 0x58u32;
            let str_electronasar_off = 0x6cu32;
            let payload_off = 0x88u32;
            rsrc.resize(payload_off as usize, 0);

            // Root dir (0x00)
            rsrc[12..14].copy_from_slice(&1u16.to_le_bytes());
            rsrc[16..20].copy_from_slice(&(0x8000_0000 | str_integrity_off).to_le_bytes());
            rsrc[20..24].copy_from_slice(&(0x8000_0000 | 0x18u32).to_le_bytes());

            // Type dir (0x18)
            rsrc[0x18 + 12..0x18 + 14].copy_from_slice(&1u16.to_le_bytes());
            rsrc[0x18 + 16..0x18 + 20]
                .copy_from_slice(&(0x8000_0000 | str_electronasar_off).to_le_bytes());
            rsrc[0x18 + 20..0x18 + 24].copy_from_slice(&(0x8000_0000 | 0x30u32).to_le_bytes());

            // Name dir (0x30)
            rsrc[0x30 + 14..0x30 + 16].copy_from_slice(&1u16.to_le_bytes());
            rsrc[0x30 + 16..0x30 + 20].copy_from_slice(&1033u32.to_le_bytes());
            rsrc[0x30 + 20..0x30 + 24].copy_from_slice(&0x48u32.to_le_bytes());

            // Leaf data entry (0x48)
            rsrc[0x48..0x4c].copy_from_slice(&(rsrc_rva + payload_off).to_le_bytes());
            rsrc[0x4c..0x50].copy_from_slice(&(json.len() as u32).to_le_bytes());

            let write_u16_str = |buf: &mut [u8], off: usize, s: &str| {
                let units: Vec<u16> = s.encode_utf16().collect();
                buf[off..off + 2].copy_from_slice(&(units.len() as u16).to_le_bytes());
                for (i, u) in units.into_iter().enumerate() {
                    buf[off + 2 + i * 2..off + 4 + i * 2].copy_from_slice(&u.to_le_bytes());
                }
            };
            write_u16_str(&mut rsrc, str_integrity_off as usize, "INTEGRITY");
            write_u16_str(&mut rsrc, str_electronasar_off as usize, "ELECTRONASAR");
            rsrc.extend_from_slice(json.as_bytes());
        }

        let mut img = vec![0u8; rsrc_raw_off as usize + rsrc.len()];
        img[0..2].copy_from_slice(b"MZ");
        img[0x3c..0x40].copy_from_slice(&(pe_off as u32).to_le_bytes());
        img[pe_off..pe_off + 4].copy_from_slice(b"PE\0\0");
        img[pe_off + 4..pe_off + 6].copy_from_slice(&machine.to_le_bytes());
        let num_sections: u16 = if rsrc.is_empty() { 0 } else { 1 };
        img[pe_off + 6..pe_off + 8].copy_from_slice(&num_sections.to_le_bytes());
        img[pe_off + 20..pe_off + 22].copy_from_slice(&opt_size.to_le_bytes());

        let opt_off = pe_off + 24;
        let magic: u16 = if is_64 { 0x20b } else { 0x10b };
        img[opt_off..opt_off + 2].copy_from_slice(&magic.to_le_bytes());
        let dd_off = opt_off + if is_64 { 112 } else { 96 };
        img[dd_off - 4..dd_off].copy_from_slice(&16u32.to_le_bytes());
        if !rsrc.is_empty() {
            img[dd_off + 16..dd_off + 20].copy_from_slice(&rsrc_rva.to_le_bytes());
            img[dd_off + 20..dd_off + 24].copy_from_slice(&(rsrc.len() as u32).to_le_bytes());
            let sec_off = opt_off + usize::from(opt_size);
            img[sec_off..sec_off + 5].copy_from_slice(b".rsrc");
            img[sec_off + 8..sec_off + 12].copy_from_slice(&(rsrc.len() as u32).to_le_bytes());
            img[sec_off + 12..sec_off + 16].copy_from_slice(&rsrc_rva.to_le_bytes());
            img[sec_off + 16..sec_off + 20].copy_from_slice(&(rsrc.len() as u32).to_le_bytes());
            img[sec_off + 20..sec_off + 24].copy_from_slice(&rsrc_raw_off.to_le_bytes());
            img[rsrc_raw_off as usize..].copy_from_slice(&rsrc);
        }
        img
    }

    #[test]
    fn parses_pe32_and_pe64_arch_and_electron_integrity_resource() {
        let dir = std::env::temp_dir().join(format!(
            "injectra-pe-unit-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();

        let json = r#"[{"file":"resources\\app.asar","alg":"sha256","value":"deadbeef"}]"#;
        let pe64_path = dir.join("app64.exe");
        fs::write(&pe64_path, build_synthetic_pe(PeArch::X64, Some(json))).unwrap();
        let s64 = inspect_pe(&pe64_path).expect("valid PE64");
        assert_eq!(s64.arch, PeArch::X64);
        assert_eq!(s64.asar_integrity.len(), 1);
        assert_eq!(s64.asar_integrity[0].file, r"resources\app.asar");
        assert_eq!(s64.asar_integrity[0].value, "deadbeef");

        let pe32_path = dir.join("app32.exe");
        fs::write(&pe32_path, build_synthetic_pe(PeArch::X86, None)).unwrap();
        let s32 = inspect_pe(&pe32_path).expect("valid PE32");
        assert_eq!(s32.arch, PeArch::X86);
        assert!(s32.asar_integrity.is_empty());

        let invalid = dir.join("not_pe.bin");
        fs::write(&invalid, b"not a PE").unwrap();
        assert!(inspect_pe(&invalid).is_none());

        let _ = fs::remove_dir_all(&dir);
    }
}
