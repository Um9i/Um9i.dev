//! Request-level errors and how they render.

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("template rendering failed: {0}")]
    Render(#[from] askama::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => crate::routes::not_found_page().into_response(),
            Self::Render(err) => {
                tracing::error!(error = %err, "template rendering failed");
                // Deliberately plain: if templates are broken we can't use them here.
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html("<!doctype html><title>Error</title><h1>Something went wrong</h1>"),
                )
                    .into_response()
            }
        }
    }
}
