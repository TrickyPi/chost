use hyper::{header, Body, Method, Response, StatusCode};
use std::path::PathBuf;

const NOT_FOUND: &[u8] = b"Not Found";
const DEFAULT_HTML: &str = "index.html";

pub async fn response_file_content(
    mut path: PathBuf,
    method: Method,
    req_path: String,
) -> Response<Body> {
    if method == Method::OPTIONS {
        return response_with_no_body(StatusCode::NO_CONTENT);
    }

    if method != Method::GET {
        return not_found();
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
        let builder = Response::builder();
        return builder
            .header(header::CONTENT_TYPE, get_content_type(extension))
            .body(body)
            .unwrap();
    }
    not_found()
}

fn response_with_no_body(status: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::empty())
        .unwrap()
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
