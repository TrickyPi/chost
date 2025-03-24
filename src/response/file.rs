use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{body::Bytes, header, Method, Response, StatusCode};
use std::path::PathBuf;

const NOT_FOUND: &[u8] = b"Not Found";
const DEFAULT_HTML: &str = "index.html";

pub async fn response_file_content(
    mut path: PathBuf,
    method: Method,
    req_path: String,
) -> Response<BoxBody<Bytes, hyper::Error>> {
    if method == Method::OPTIONS {
        return response_with_no_body(StatusCode::NO_CONTENT);
    }

    if method != Method::GET && method != Method::HEAD {
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
        let mut builder = Response::builder();
        builder = builder.header(header::CONTENT_TYPE, get_content_type(extension));
        if method == Method::HEAD {
            return builder.body(empty()).unwrap();
        } else {
            return builder.body(full(contents)).unwrap();
        }
    }
    not_found()
}

fn response_with_no_body(status: StatusCode) -> Response<BoxBody<Bytes, hyper::Error>> {
    Response::builder().status(status).body(empty()).unwrap()
}

fn not_found() -> Response<BoxBody<Bytes, hyper::Error>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(full(NOT_FOUND))
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
            "pdf" => "application/pdf",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            &_ => "text/plain",
        },
        None => "text/plain",
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
