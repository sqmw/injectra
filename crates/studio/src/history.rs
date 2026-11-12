use crate::{
    endpoint::write_private_json,
    error::{ApiError, Result},
    model::Run,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const MAX_RUNS: usize = 10_000;

pub struct History {
    directory: PathBuf,
}
impl History {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }
    pub fn save(&self, run: &Run) -> Result<()> {
        if !(run.request_id.len() == 32 || run.request_id.len() == 64)
            || !run.request_id.bytes().all(|c| c.is_ascii_hexdigit())
        {
            return Err(ApiError::invalid("Invalid request identity"));
        }
        crate::endpoint::private_dir(&self.directory)?;
        write_private_json(
            &self.directory.join(format!("{}.json", run.request_id)),
            run,
        )
    }
    pub fn load(&self) -> Result<Vec<Run>> {
        let mut records = Vec::new();
        let entries = match fs::read_dir(&self.directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(records),
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            if records.len() >= MAX_RUNS {
                return Err(ApiError::conflict(
                    "Execution history exceeds service capacity; no records were deleted",
                ));
            }
            let meta = fs::symlink_metadata(&path)?;
            if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 4 * 1024 * 1024 {
                return Err(ApiError::invalid("Invalid execution history file"));
            }
            let mut record: Run = serde_json::from_slice(&fs::read(&path)?)?;
            if path.file_stem().and_then(|s| s.to_str()) != Some(record.request_id.as_str()) {
                return Err(ApiError::invalid("Execution history identity mismatch"));
            }
            if matches!(record.status.as_str(), "queued" | "running") {
                record.status = "unknown".into();
                record.certainty = "Unknown".into();
                record.error = Some(ApiError::unknown(
                    "Service restarted without a confirmed execution result; no replay",
                ));
                self.save(&record)?;
            }
            records.push(record);
        }
        records.sort_by_key(|r| r.created_at);
        Ok(records)
    }
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}
