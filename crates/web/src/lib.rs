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
