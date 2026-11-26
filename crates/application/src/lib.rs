//! Reusable Web injection use cases; concrete backends are supplied by the caller.
use injectra_core::{Error, ErrorKind, Result, TargetRef, require_capability};
use injectra_web::{Bundle, PAGE_EVALUATE, WebBackend, WebTarget};
use serde_json::Value;
use std::path::Path;

pub fn targets(backend: &dyn WebBackend) -> Result<Vec<WebTarget>> {
    backend.targets()
}

pub fn apply(backend: &dyn WebBackend, target: &TargetRef, manifest: &Path) -> Result<Value> {
    execute(
        backend,
        target,
        &injectra_web::apply(&Bundle::load(manifest)?)?,
    )
}
pub fn remove(backend: &dyn WebBackend, target: &TargetRef, id: &str) -> Result<Value> {
    execute(backend, target, &injectra_web::remove(id)?)
}
pub fn list(backend: &dyn WebBackend, target: &TargetRef) -> Result<Value> {
    execute(backend, target, &injectra_web::list())
}
pub fn eval(backend: &dyn WebBackend, target: &TargetRef, expression: &str) -> Result<Value> {
    execute(
        backend,
        target,
        &injectra_web::evaluate_expression(expression)?,
    )
}
pub fn execute(backend: &dyn WebBackend, target: &TargetRef, program: &str) -> Result<Value> {
    target.ensure_backend(backend.id())?;
    let mut session = backend.connect(target)?;
    if session.target() != target {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "backend connected a different target",
        ));
    }
    require_capability(session.capabilities(), PAGE_EVALUATE)?;
    session.evaluate_page(program).map_err(|error| {
        let message = error.to_string();
        if error.kind == ErrorKind::Execution && message.contains("[injectra:policy:") {
            Error::new(
                ErrorKind::PolicyBlocked,
                error.phase,
                error.certainty,
                message,
            )
            .caused_by(error)
        } else {
            error
        }
    })
}

pub mod dev;
