//! Minimal page-increment preparation. No editable workspace or concrete backend.
use injectra_core::{Error, ErrorKind, Result, TargetRef};
use injectra_web::{Bundle, WebBackend};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

pub struct Revision {
    pub hash: String,
    pub files: Value,
    pub program: String,
    pub js_start_line: usize,
    pub js_lines: usize,
}
fn content(bundle: &Bundle) -> Value {
    json!([bundle.id, bundle.js, bundle.css, bundle.html])
}
impl Revision {
    pub fn source_location(&self, error: &str) -> Option<Value> {
        for frame in error.lines().filter(|l| {
            l.contains("injectra://request/") && l.contains(&format!("?revision={}&", self.hash))
        }) {
            let mut parts = frame.trim_end_matches(')').rsplitn(3, ':');
            let column = parts.next()?.parse::<usize>().ok()?;
            let line = parts.next()?.parse::<usize>().ok()?;
            if line >= self.js_start_line && line < self.js_start_line + self.js_lines {
                return Some(
                    json!({"file":self.files["js"],"line":line-self.js_start_line+1,"column":column}),
                );
            }
        }
        None
    }
    /// Double capture refuses edits observed during reading; callers debounce saves.
    pub fn load(manifest: &Path) -> Result<Self> {
        if fs::metadata(manifest)?.len() > 1024 * 1024 {
            return Err("Manifest exceeds 1 MiB".into());
        }
        let before = fs::read(manifest)?;
        let bundle = Bundle::load(manifest)?;
        let again = Bundle::load(manifest)?;
        if before != fs::read(manifest)? || content(&bundle) != content(&again) {
            return Err(Error::prepare(
                ErrorKind::InvalidInput,
                "Increment changed during capture",
            ));
        }
        let hash = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&json!([before, content(&bundle)])).unwrap())
        );
        let description: Value = serde_json::from_slice(&before)
            .map_err(|e| Error::prepare(ErrorKind::InvalidInput, e.to_string()))?;
        let files = json!({"manifest":manifest, "js":description["js"],"css":description["css"],"html":description["html"]});
        let mut program = injectra_web::apply(&bundle)?;
        // The shared generator places user JS immediately after the last wrapper prefix.
        let prefix = program
            .find("async (context) => {\n")
            .ok_or("Missing shared runtime source boundary")?;
        let line = program[..prefix].bytes().filter(|b| *b == b'\n').count() + 2;
        program.push_str(&format!(
            "\n//# sourceURL=injectra://increment/{hash}/page.js?user_start={line}\n"
        ));
        Ok(Self {
            hash,
            files,
            program,
            js_start_line: line,
            js_lines: bundle.js.lines().count(),
        })
    }
}

/// Tracks attempts, not merely successful saves. Execution is always explicitly dispatched.
#[derive(Default)]
pub struct Updates {
    last_attempt: Option<String>,
}
impl Updates {
    pub fn changed(&self, revision: &Revision) -> bool {
        self.last_attempt.as_ref() != Some(&revision.hash)
    }
    pub fn apply(
        &mut self,
        backend: &dyn WebBackend,
        target: &TargetRef,
        revision: &Revision,
    ) -> Result<Value> {
        self.last_attempt = Some(revision.hash.clone());
        crate::execute(backend, target, &revision.program)
    }
}
