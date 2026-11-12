//! Authenticated local workbench. Backends remain synchronous and thread-confined.
pub mod actor;
pub mod endpoint;
pub mod error;
pub mod security;

pub mod artifacts;
pub mod history;
pub mod model;
pub mod state;

pub mod http;
pub mod server;
pub use server::{Studio, StudioOptions};
