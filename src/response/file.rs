use hyper::http::response::Builder;
use hyper::{header, Body, Method, Response, StatusCode};
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;

const NOT_FOUND: &[u8] = b"Not Found";
const DEFAULT_HTML: &str = "index.html";

pub async fn response_file_content(
    mut path: PathBuf,
    cors_arc: Arc<bool>,
    method: Method,
    req_path: String,
) -> Result<Response<Body>, Infallible> {
    if method != Method::GET {
        return Ok::<_, Infallible>(not_found());
    }

    let mut original_path = path.clone();

    path.push(req_path);

    if path.is_dir() {
        path.push(DEFAULT_HTML);
    }

    let mut extension = path.extension().and_then(|f| f.to_str());

    if let Some(contents) = match tokio::fs::read(&path).await {
        Ok(contents) => Some(contents),
        Err(_) => {
            if extension.is_none() {
                original_path.push(DEFAULT_HTML);
                match tokio::fs::read(original_path).await {
                    Ok(contents) => {
                        extension = Some("html");
                        Some(contents)
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        }
    } {
        let body = contents.into();
        let mut builder = Response::builder();
        if *cors_arc {
            builder = set_cors_headers(builder)
        }
        return Ok::<_, Infallible>(
            builder
                .header(header::CONTENT_TYPE, get_content_type(extension))
                .body(body)
                .unwrap(),
        );
    }

    Ok::<_, Infallible>(not_found())
}

fn set_cors_headers(builder: Builder) -> Builder {
    builder
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "*")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOT_FOUND.into())
        .unwrap()
}

fn get_content_type(extension: Option<&str>) -> &str {
    match extension {
        Some(v) => match v {
            "html" => "text/html",
            "js" => "application/javascript",
            "css" => "text/css",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" => "image/jpg",
            "svg" => "image/svg+xml",
            "wasm" => "application/wasm",
            &_ => "text/plain",
        },
        None => "text/plain",
    }
}
