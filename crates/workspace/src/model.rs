use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceOrigin {
    Application,
    Injectra,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceStage {
    Page,
    Startup,
    Shared,
    Metadata,
}

/// Reference to code owned elsewhere. It never grants write access to that path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReference {
    pub origin: SourceOrigin,
    pub path: String,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub origin: SourceOrigin,
    pub stage: SourceStage,
    pub hash: String,
    pub bytes: usize,
    pub editable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported_from: Option<SourceReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapping {
    pub origin: SourceOrigin,
    pub path: String,
    pub revision: String,
    pub source_url: String,
    /// 1-based line in the generated execution unit where user JS starts.
    pub generated_start_line: usize,
    pub generated_end_line: usize,
    pub source_start_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedProgram {
    pub source: String,
    pub mappings: Vec<SourceMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Manifest {
    pub version: u32,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub js: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
}

impl Manifest {
    pub fn parse(bytes: &[u8]) -> injectra_core::Result<Self> {
        Self::parse_with_mode(bytes, true)
    }
    pub fn parse_import(bytes: &[u8]) -> injectra_core::Result<Self> {
        Self::parse_with_mode(bytes, false)
    }
    fn parse_with_mode(bytes: &[u8], strict: bool) -> injectra_core::Result<Self> {
        let value: Self =
            serde_json::from_slice(bytes).map_err(|e| crate::invalid(e.to_string()))?;
        if value.version != 1 {
            return Err(crate::invalid("unsupported bundle manifest version"));
        }
        if value.id.is_empty()
            || value.id.len() > 80
            || !value
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
        {
            return Err(crate::invalid("invalid bundle id"));
        }
        let assets: Vec<_> = [&value.js, &value.css, &value.html]
            .into_iter()
            .flatten()
            .collect();
        if assets.is_empty() {
            return Err(crate::invalid("bundle manifest requires an asset"));
        }
        let mut seen = std::collections::HashSet::new();
        for path in assets {
            if !strict {
                continue;
            }
            crate::filesystem::user_path(path)?;
            if path == "manifest.json"
                || path == "startup.cjs"
                || path.starts_with('.')
                || !seen.insert(path)
            {
                return Err(crate::invalid("reserved or duplicate bundle asset path"));
            }
        }
        Ok(value)
    }
    pub fn stage(&self, path: &str) -> SourceStage {
        if path == "manifest.json" {
            SourceStage::Metadata
        } else if path == "startup.cjs" {
            SourceStage::Startup
        } else if [&self.js, &self.css, &self.html]
            .into_iter()
            .flatten()
            .any(|p| p == path)
        {
            SourceStage::Page
        } else {
            SourceStage::Shared
        }
    }
}
