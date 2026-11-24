//! Bundle loading and compilation share one browser runtime across all engines.
mod bundle;
mod contracts;
pub use bundle::Bundle;
pub use contracts::{PAGE_EVALUATE, PageContext, WebBackend, WebSession, WebTarget};

use injectra_core::Result;
use serde_json::json;

const RUNTIME: &str = include_str!("../../../web/runtime.js");

pub fn apply(bundle: &Bundle) -> Result<String> {
    let request = json!({"operation":"apply", "id":bundle.id,
        "css":bundle.css, "html":bundle.html});
    // The script is deliberately executable source, whereas CSS/HTML/id are JSON data.
    // Never interpolate those data fields into executable JavaScript literals.
    Ok(format!(
        "({RUNTIME})({}, async (context) => {{\n{}\n}})",
        serde_json::to_string(&request).expect("request contains only strings"),
        bundle.js
    ))
}

pub fn remove(id: &str) -> Result<String> {
    bundle::validate_id(id)?;
    Ok(format!(
        "({RUNTIME})({}, null)",
        json!({"operation":"remove", "id":id})
    ))
}

pub fn list() -> String {
    format!("({RUNTIME})({}, null)", json!({"operation": "list"}))
}

pub fn evaluate_expression(expression: &str) -> Result<String> {
    if expression.trim().is_empty() {
        return Err("evaluation expression must not be empty".into());
    }
    Ok(format!(
        "(async () => {{\n  const __injectraValue = await (\n{expression}\n  );\n  return __injectraValue === undefined ? null : __injectraValue;\n}})()"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_program_targets_shared_runtime_list_operation() {
        let program = list();
        assert!(program.contains("injectraRuntime"));
        assert!(program.contains(r#""operation":"list""#));
    }

    #[test]
    fn evaluate_expression_rejects_blank_and_normalizes_undefined() {
        assert!(evaluate_expression("   \n\t ").is_err());
        let program = evaluate_expression("({ ok: true }) // trailing comment").unwrap();
        assert!(program.contains("await (\n({ ok: true }) // trailing comment\n  );"));
        assert!(program.contains("__injectraValue === undefined ? null : __injectraValue"));
    }
}
