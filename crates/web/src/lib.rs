//! Bundle loading and compilation share one browser runtime across all engines.
mod bundle;
mod contracts;
pub mod dom;
pub use bundle::{Bundle, HtmlMode};
pub use contracts::{PAGE_EVALUATE, PageContext, WebBackend, WebSession, WebTarget};

use injectra_core::Result;
use serde_json::json;

const RUNTIME: &str = include_str!("../../../web/runtime.js");

pub fn apply(bundle: &Bundle) -> Result<String> {
    let request = match bundle.html_mode {
        HtmlMode::InnerHtml => json!({
            "operation": "apply",
            "id": bundle.id,
            "css": bundle.css,
            "html": bundle.html,
        }),
        HtmlMode::Dom => {
            let dom_nodes = if bundle.html.is_empty() {
                Vec::new()
            } else {
                dom::compile_html_to_dom(&bundle.html)?
            };
            json!({
                "operation": "apply",
                "id": bundle.id,
                "css": bundle.css,
                "html": "",
                "dom": dom_nodes,
            })
        }
    };
    // The script is deliberately executable source, whereas CSS/HTML/DOM/id are JSON data.
    // Never interpolate those data fields into executable JavaScript literals.
    Ok(format!(
        "({RUNTIME})({}, async (context) => {{\n{}\n}})",
        serde_json::to_string(&request).expect("request serializes"),
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

    #[test]
    fn dom_html_mode_compiles_safe_tree_and_rejects_script_before_execution() {
        let ok_bundle = Bundle {
            id: "tt.bundle".into(),
            css: "button{}".into(),
            html: r#"<button data-k="1">Ok</button><svg><circle r="2"/></svg>"#.into(),
            js: String::new(),
            html_mode: HtmlMode::Dom,
        };
        let program = apply(&ok_bundle).unwrap();
        assert!(program.contains(r#""html":"""#));
        assert!(program.contains(r#""dom":[{"a":[["data-k","1"]],"c":["Ok"],"t":"button"}"#));
        assert!(program.contains(r#""ns":"svg""#));

        let bad_bundle = Bundle {
            id: "tt.bad".into(),
            css: String::new(),
            html: "<script>alert(1)</script>".into(),
            js: String::new(),
            html_mode: HtmlMode::Dom,
        };
        let err = apply(&bad_bundle).unwrap_err();
        assert_eq!(err.kind, injectra_core::ErrorKind::InvalidInput);
        assert_eq!(err.certainty, injectra_core::Certainty::NotExecuted);
    }
}
