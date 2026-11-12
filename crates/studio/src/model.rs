use crate::{actor::Target, error::ApiError};
use injectra_workspace::{
    ExecutionCertainty, ExecutionRecord, ExecutionStatus, ExecutionTarget, SourceMapping,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct Run {
    pub request_id: String,
    pub target: String,
    pub generation: String,
    pub backend_instance: String,
    pub revision: String,
    pub status: String,
    pub certainty: String,
    pub result: Option<Value>,
    pub error: Option<ApiError>,
    pub source_mappings: Vec<SourceMapping>,
    pub snapshot: PathBuf,
    pub created_at: u64,
}
impl Run {
    pub fn new(record: &ExecutionRecord, mappings: Vec<SourceMapping>, snapshot: PathBuf) -> Self {
        Self {
            request_id: record.request_id.clone(),
            target: record.target.target.clone(),
            generation: record.target.generation.clone(),
            backend_instance: record.target.backend_instance.clone(),
            revision: record.revision.clone(),
            status: "queued".into(),
            certainty: "NotExecuted".into(),
            result: None,
            error: None,
            source_mappings: mappings,
            snapshot,
            created_at: now(),
        }
    }
    pub fn update(&mut self, record: &ExecutionRecord) {
        self.status = match record.status {
            ExecutionStatus::Running if self.status == "queued" => "queued",
            ExecutionStatus::Running => "running",
            ExecutionStatus::Succeeded => "succeeded",
            ExecutionStatus::Failed => "failed",
            ExecutionStatus::Unknown => "unknown",
        }
        .into();
        self.certainty = match record.certainty {
            ExecutionCertainty::NotExecuted => "NotExecuted",
            ExecutionCertainty::Executed => "Executed",
            ExecutionCertainty::Unknown => "Unknown",
        }
        .into();
        self.result = record.result.clone();
        if self.error.is_none() {
            self.error = record
                .error
                .as_ref()
                .map(|s| ApiError::new(502, "execution", s.clone(), &self.certainty));
        }
    }
    pub fn identity(&self) -> ExecutionTarget {
        ExecutionTarget {
            backend_instance: self.backend_instance.clone(),
            target: self.target.clone(),
            generation: self.generation.clone(),
        }
    }
}
pub fn target_identity(target: &Target) -> ExecutionTarget {
    ExecutionTarget {
        backend_instance: target.backend_instance.clone(),
        target: target.id.clone(),
        generation: target.generation.clone(),
    }
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Startup {
    pub saved_revision: Option<String>,
    pub applied_revision: Option<String>,
    pub status: String,
    pub pending_restart: bool,
    pub artifact: Option<String>,
    pub instance: Option<String>,
    pub error: Option<String>,
}
pub fn attribution(message: &str, mappings: &[SourceMapping]) -> Option<Value> {
    for mapping in mappings {
        let prefix = format!("{}:", mapping.source_url);
        for (_, rest) in message
            .match_indices(&prefix)
            .map(|(i, _)| (i, &message[i + prefix.len()..]))
        {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            let Ok(line) = digits.parse::<usize>() else {
                continue;
            };
            if line >= mapping.generated_start_line && line <= mapping.generated_end_line {
                return Some(
                    serde_json::json!({"origin":mapping.origin,"path":mapping.path,"line":mapping.source_start_line+line-mapping.generated_start_line,"source_url":mapping.source_url}),
                );
            }
        }
    }
    None
}
