//! One pending request and latest receipt, not an execution-history database.
use injectra_core::{Certainty, Result};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
pub(super) struct Journal {
    root: PathBuf,
}
impl Journal {
    pub fn open(root: &Path, ack: Option<&str>) -> Result<Self> {
        let this = Self { root: root.into() };
        let pending = this.root.join("pending.json");
        if pending.exists() {
            let value: Value =
                serde_json::from_slice(&fs::read(&pending)?).map_err(|e| e.to_string())?;
            let request = value["request_id"]
                .as_str()
                .filter(|id| !id.is_empty())
                .ok_or("Malformed unresolved request; preserved for inspection")?;
            if ack != Some(request) {
                return Err(format!("Previous request {} may have executed; review effects and pass --acknowledge-unknown {}",value["request_id"],value["request_id"]).into());
            }
            this.write(
                "receipt.json",
                &json!({"certainty":"Unknown","acknowledged":value}),
            )?;
            fs::remove_file(pending)?;
        } else if ack.is_some() {
            return Err("No unresolved request matches acknowledgement".into());
        }
        Ok(this)
    }
    fn write(&self, name: &str, value: &Value) -> Result<()> {
        let temporary = self.root.join(format!("{name}.tmp"));
        let mut options = fs::OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        let mut file = options.open(&temporary)?;
        file.write_all(&serde_json::to_vec(value).unwrap())?;
        file.sync_all()?;
        fs::rename(temporary, self.root.join(name))?;
        fs::File::open(&self.root)?.sync_all()?;
        Ok(())
    }
    pub fn begin(&self, id: &str, version: &str, target: &str, files: &Value) -> Result<()> {
        self.write(
            "pending.json",
            &json!({"request_id":id,"version":version,"target":target,"files":files}),
        )
    }
    pub fn complete(&self, value: &Value, certainty: Certainty) -> Result<()> {
        self.write("receipt.json", value)?;
        if certainty != Certainty::Unknown {
            fs::remove_file(self.root.join("pending.json"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unresolved_submission_survives_reopen_and_needs_exact_acknowledgement() {
        let root = std::env::temp_dir().join(format!(
            "injectra-journal-{}",
            injectra_bootstrap::random_token()
        ));
        fs::create_dir(&root).unwrap();
        let journal = Journal::open(&root, None).unwrap();
        journal
            .begin("request-a", "v1", "page-1", &json!({}))
            .unwrap();
        assert!(Journal::open(&root, None).is_err());
        assert!(Journal::open(&root, Some("request-b")).is_err());
        let journal = Journal::open(&root, Some("request-a")).unwrap();
        journal
            .begin("request-c", "v2", "page-1", &json!({}))
            .unwrap();
        journal
            .complete(&json!({"request_id":"request-c"}), Certainty::Executed)
            .unwrap();
        assert!(Journal::open(&root, None).is_ok());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn malformed_pending_is_preserved_instead_of_silently_acknowledged() {
        let root = std::env::temp_dir().join(format!(
            "injectra-journal-{}",
            injectra_bootstrap::random_token()
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("pending.json"), b"{}").unwrap();
        assert!(Journal::open(&root, None).is_err());
        assert!(root.join("pending.json").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
