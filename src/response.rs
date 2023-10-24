use hyper::http::response::Builder;
use hyper::{header, Body, Method, Request, Response, StatusCode};
use std::convert::Infallible;
use std::path::PathBuf;

const NOT_FOUND: &[u8] = b"Not Found";

pub async fn response_file_content(
    mut path: PathBuf,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let req_path = req.uri().path().strip_prefix("/").unwrap();
    path.push(req_path);
    let method = req.method();
    if method != Method::GET || !path.exists() {
        return Ok::<_, Infallible>(not_found());
    }
    if path.is_dir() {
        path.push("index.html");
    }

    let content_type = get_content_type(&path);

    if let Ok(contents) = tokio::fs::read(&path).await {
        let body = contents.into();
        let builder = Response::builder();
        return Ok::<_, Infallible>(
            set_cors_headers(builder)
                .header(header::CONTENT_TYPE, content_type)
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

fn get_content_type(path: &PathBuf) -> &str {
    let extension = path.extension().and_then(|f| f.to_str());
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
