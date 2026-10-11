//! The Um9i.dev web application.
//!
//! The binary in `main.rs` is a thin shell around [`app`], which lets the
//! integration tests drive the exact same router without binding a socket.

pub mod config;
pub mod content;
pub mod error;
mod routes;

use std::sync::Arc;

use axum::Router;

use crate::content::Site;

/// Shared, read-only application state.
///
/// Content is parsed once at startup and never mutated, so an `Arc` is all the
/// synchronisation it needs.
#[derive(Clone)]
pub struct AppState {
    pub site: Arc<Site>,
}

/// Build the full application router, including middleware.
pub fn app(state: AppState, static_dir: &std::path::Path) -> Router {
    routes::router(state, static_dir)
}
