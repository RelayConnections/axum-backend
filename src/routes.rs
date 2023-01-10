use axum::{response::IntoResponse, response::Response, http::{StatusCode, header, HeaderValue}, body::{self, Full}, extract::Path};
use include_dir::{include_dir, Dir};
use mime_guess::mime;

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/public_html");

pub async fn page_not_found() -> Response {
    Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed("PAGE NOT FOUND".to_owned()))
            .unwrap()
}

pub async fn serve_index() -> impl IntoResponse {
    match STATIC_DIR.get_file("index.html") {
        // File does not exist
        None => return page_not_found().await,
        // File exists
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime::TEXT_HTML_UTF_8.as_ref()).unwrap(),
            )
            .body(body::boxed(Full::from(file.contents())))
            .unwrap(),
    }
}

pub async fn serve_file(Path(path): Path<String>) -> impl IntoResponse {
    // Split path by /
    let path = path.trim_start_matches('/').to_string();

    // Try to figure out the MIME type
    let mime_type = mime_guess::from_path(path.clone()).first_or_text_plain();

    match STATIC_DIR.get_file(path) {
        // File does not exist
        None => return page_not_found().await,
        // File exists
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(body::boxed(Full::from(file.contents())))
            .unwrap(),
    }
}