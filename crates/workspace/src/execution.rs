use crate::{invalid, unique_id};
use injectra_core::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutionTarget {
    pub backend_instance: String,
    pub target: String,
    pub generation: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Running,
    Succeeded,
    Failed,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionCertainty {
    NotExecuted,
    Executed,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub request_id: String,
    pub target: ExecutionTarget,
    pub revision: String,
    pub status: ExecutionStatus,
    pub certainty: ExecutionCertainty,
    pub result: Option<Value>,
    pub error: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOutcome {
    pub certainty: ExecutionCertainty,
    pub result: Option<Value>,
    pub error: Option<String>,
}
#[derive(Default)]
pub struct ExecutionLedger {
    retired: HashSet<ExecutionTarget>,
    records: Vec<ExecutionRecord>,
    active: HashMap<ExecutionTarget, String>,
    uncertain: HashMap<ExecutionTarget, String>,
    applied: HashMap<ExecutionTarget, String>,
}
impl ExecutionLedger {
    pub fn begin(
        &mut self,
        target: ExecutionTarget,
        revision: impl Into<String>,
    ) -> Result<ExecutionRecord> {
        if [&target.backend_instance, &target.target, &target.generation]
            .iter()
            .any(|v| v.is_empty())
        {
            return Err(invalid(
                "execution requires backend instance, target and generation",
            ));
        }
        let revision = revision.into();
        if revision.is_empty() {
            return Err(invalid("execution requires a revision"));
        }
        if self.retired.contains(&target) {
            return Err(invalid("target generation is retired"));
        }
        if self.active.contains_key(&target) {
            return Err(invalid("target already has an active execution"));
        }
        if self.uncertain.contains_key(&target) {
            return Err(invalid(
                "target has an unacknowledged Unknown outcome; no automatic replay",
            ));
        }
        let record = ExecutionRecord {
            request_id: unique_id()?,
            target: target.clone(),
            revision,
            status: ExecutionStatus::Running,
            certainty: ExecutionCertainty::NotExecuted,
            result: None,
            error: None,
        };
        self.active.insert(target, record.request_id.clone());
        self.records.push(record.clone());
        Ok(record)
    }
    pub fn complete(
        &mut self,
        request_id: &str,
        target: &ExecutionTarget,
        revision: &str,
        outcome: ExecutionOutcome,
    ) -> Result<ExecutionRecord> {
        if self.active.get(target).map(String::as_str) != Some(request_id) {
            return Err(invalid("stale or mismatched execution reply"));
        }
        let record = self
            .records
            .iter_mut()
            .find(|r| r.request_id == request_id)
            .ok_or_else(|| invalid("unknown request"))?;
        if record.revision != revision || record.target != *target {
            return Err(invalid("reply revision or target does not match request"));
        }
        if outcome.certainty == ExecutionCertainty::NotExecuted && outcome.error.is_none() {
            return Err(invalid("NotExecuted cannot report success"));
        }
        if outcome.certainty == ExecutionCertainty::Unknown
            || outcome.certainty == ExecutionCertainty::Executed && outcome.error.is_some()
        {
            self.applied.remove(target);
        }
        record.certainty = outcome.certainty;
        record.result = outcome.result;
        record.error = outcome.error;
        record.status = if record.certainty == ExecutionCertainty::Unknown {
            self.uncertain.insert(target.clone(), request_id.to_owned());
            ExecutionStatus::Unknown
        } else if record.error.is_some() {
            ExecutionStatus::Failed
        } else {
            self.applied.insert(target.clone(), revision.to_owned());
            ExecutionStatus::Succeeded
        };
        self.active.remove(target);
        Ok(record.clone())
    }
    /// An explicit caller decision only clears the gate; it never queues or replays code.
    pub fn acknowledge_unknown(
        &mut self,
        target: &ExecutionTarget,
        request_id: &str,
    ) -> Result<()> {
        if self.uncertain.get(target).map(String::as_str) != Some(request_id) {
            return Err(invalid(
                "Unknown acknowledgement does not match target/request",
            ));
        }
        self.uncertain.remove(target);
        Ok(())
    }
    /// Destroyed contexts cannot receive late replies; pending work becomes Unknown.
    pub fn invalidate(&mut self, target: &ExecutionTarget) -> Result<()> {
        if let Some(id) = self.active.get(target).cloned() {
            let revision = self
                .records
                .iter()
                .find(|r| r.request_id == id)
                .unwrap()
                .revision
                .clone();
            self.complete(
                &id,
                target,
                &revision,
                ExecutionOutcome {
                    certainty: ExecutionCertainty::Unknown,
                    result: None,
                    error: Some("target generation invalidated".into()),
                },
            )?;
        }
        self.applied.remove(target);
        self.retired.insert(target.clone());
        Ok(())
    }
    pub fn applied_revision(&self, target: &ExecutionTarget) -> Option<&str> {
        self.applied.get(target).map(String::as_str)
    }
    pub fn records(&self) -> &[ExecutionRecord] {
        &self.records
    }
}
