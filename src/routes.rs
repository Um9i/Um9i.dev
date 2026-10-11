//! HTTP routes, page templates and middleware.

use std::path::Path;
use std::time::Duration;

use askama::Template;
use axum::Router;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderName, HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::AppState;
use crate::content::{Project, Site};
use crate::error::AppError;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const STATIC_CACHE_CONTROL: HeaderValue = HeaderValue::from_static("public, max-age=3600");
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; img-src 'self' data:; \
     style-src 'self'; script-src 'self'; frame-ancestors 'none'; base-uri 'self'";

pub(crate) fn router(state: AppState, static_dir: &Path) -> Router {
    let static_files = Router::new()
        .fallback_service(ServeDir::new(static_dir))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            STATIC_CACHE_CONTROL,
        ));

    Router::new()
        .route("/", get(index))
        .route("/projects", get(projects))
        .route("/projects/{slug}", get(project))
        .route("/healthz", get(healthz))
        .route("/robots.txt", get(robots))
        .nest_service("/static", static_files)
        .fallback(fallback)
        .with_state(state)
        .layer(security_header(
            header::CONTENT_SECURITY_POLICY,
            CONTENT_SECURITY_POLICY,
        ))
        .layer(security_header(header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .layer(security_header(header::X_FRAME_OPTIONS, "DENY"))
        .layer(security_header(
            header::REFERRER_POLICY,
            "strict-origin-when-cross-origin",
        ))
        .layer(security_header(
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=63072000; includeSubDomains",
        ))
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .layer(TraceLayer::new_for_http())
}

fn security_header(name: HeaderName, value: &'static str) -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::if_not_present(name, HeaderValue::from_static(value))
}

/// Render an askama template into an HTML response.
fn render(template: &impl Template) -> Result<Html<String>, AppError> {
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    nav: &'static str,
    site: &'a Site,
    featured: Vec<&'a Project>,
}

async fn index(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    render(&IndexTemplate {
        nav: "home",
        site: &state.site,
        featured: state.site.featured().collect(),
    })
}

#[derive(Template)]
#[template(path = "projects.html")]
struct ProjectsTemplate<'a> {
    nav: &'static str,
    projects: &'a [Project],
}

async fn projects(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    render(&ProjectsTemplate {
        nav: "projects",
        projects: &state.site.projects,
    })
}

#[derive(Template)]
#[template(path = "project.html")]
struct ProjectTemplate<'a> {
    nav: &'static str,
    project: &'a Project,
}

async fn project(
    State(state): State<AppState>,
    UrlPath(slug): UrlPath<String>,
) -> Result<Html<String>, AppError> {
    let project = state.site.project(&slug).ok_or(AppError::NotFound)?;
    render(&ProjectTemplate {
        nav: "projects",
        project,
    })
}

async fn healthz() -> &'static str {
    "ok"
}

async fn robots() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        "User-agent: *\nAllow: /\n",
    )
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate {
    nav: &'static str,
}

async fn fallback() -> Response {
    not_found_page()
}

pub(crate) fn not_found_page() -> Response {
    let page = NotFoundTemplate { nav: "" };
    match page.render() {
        Ok(body) => (StatusCode::NOT_FOUND, Html(body)).into_response(),
        Err(err) => AppError::Render(err).into_response(),
    }
}
