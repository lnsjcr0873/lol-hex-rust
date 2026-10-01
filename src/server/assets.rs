use axum::body::Body;
use axum::http::{header, HeaderValue, Response, StatusCode};
use axum::response::IntoResponse;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "server/static/"]
pub struct StaticAssets;

pub async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() || path == "static" {
        path = "index.html".to_string();
    } else if path.starts_with("static/") {
        path = path.strip_prefix("static/").unwrap_or(&path).to_string();
    }

    match StaticAssets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            let content_type = HeaderValue::from_str(mime.as_ref())
                .unwrap_or(HeaderValue::from_static("application/octet-stream"));
            Response::builder()
                .header(header::CONTENT_TYPE, content_type)
                .body(Body::from(content.data))
                .unwrap_or_else(|_| Response::new(Body::empty()))
        }
        None => {
            // Fallback to index.html for SPA routing
            if let Some(content) = StaticAssets::get("index.html") {
                Response::builder()
                    .header(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))
                    .body(Body::from(content.data))
                    .unwrap_or_else(|_| Response::new(Body::empty()))
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("404 Not Found"))
                    .unwrap_or_else(|_| Response::new(Body::empty()))
            }
        }
    }
}
