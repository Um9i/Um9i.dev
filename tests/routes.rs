#![expect(clippy::unwrap_used, reason = "test helpers should fail loudly")]

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use tower::ServiceExt;
use um9i::content::Site;
use um9i::{AppState, app};

fn test_app() -> Router {
    let state = AppState {
        site: Arc::new(Site::load().unwrap()),
    };
    app(state, &Path::new(env!("CARGO_MANIFEST_DIR")).join("static"))
}

async fn get(uri: &str) -> axum::response::Response {
    test_app()
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn get_with_host(uri: &str, host: &str) -> axum::response::Response {
    test_app()
        .oneshot(
            Request::get(uri)
                .header(header::HOST, host)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn body_text(response: axum::response::Response) -> String {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn home_should_return_ok() {
    assert_eq!(get("/").await.status(), StatusCode::OK);
}

#[tokio::test]
async fn home_should_list_featured_project() {
    let body = body_text(get("/").await).await;
    assert!(body.contains(r#"href="/projects/um9i-dev""#));
}

#[tokio::test]
async fn projects_should_return_ok() {
    assert_eq!(get("/projects").await.status(), StatusCode::OK);
}

#[tokio::test]
async fn project_should_render_markdown_body() {
    let body = body_text(get("/projects/um9i-dev").await).await;
    assert!(body.contains("<h2>Stack</h2>"));
}

#[tokio::test]
async fn unknown_project_should_return_not_found() {
    assert_eq!(
        get("/projects/does-not-exist").await.status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn unknown_route_should_return_not_found() {
    assert_eq!(get("/nope").await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn healthz_should_return_ok() {
    assert_eq!(get("/healthz").await.status(), StatusCode::OK);
}

#[tokio::test]
async fn static_css_should_be_served_with_cache_control() {
    let response = get("/static/css/site.css").await;
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL).unwrap(),
        "public, max-age=3600"
    );
}

#[tokio::test]
async fn responses_should_include_content_security_policy() {
    let response = get("/").await;
    assert!(
        response
            .headers()
            .contains_key(header::CONTENT_SECURITY_POLICY)
    );
}

#[tokio::test]
async fn www_host_should_redirect_permanently() {
    let response = get_with_host("/", "www.um9i.dev").await;
    assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
}

#[tokio::test]
async fn www_redirect_should_keep_path_and_query() {
    let response = get_with_host("/projects/um9i-dev?ref=x", "www.um9i.dev").await;
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "https://um9i.dev/projects/um9i-dev?ref=x"
    );
}

#[tokio::test]
async fn www_redirect_should_ignore_host_case() {
    let response = get_with_host("/", "WWW.Um9i.dev").await;
    assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
}

#[tokio::test]
async fn apex_host_should_not_redirect() {
    assert_eq!(
        get_with_host("/", "um9i.dev").await.status(),
        StatusCode::OK
    );
}
