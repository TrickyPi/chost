use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};
use hyper::http::response::Builder;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;

const NOTFOUND: &[u8] = b"Not Found";

/// host static files
#[derive(Parser, Debug)]
struct Cli {
    /// path to host
    #[clap(parse(from_os_str))]
    path: PathBuf,
    /// enable cors
    #[clap(short, long, default_value_t = true)]
    cors: bool,
    /// port
    #[clap(short, long, default_value_t = 7878)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    create_server(args).await;
}

async fn create_server(args: Cli) {
    let Cli { port, path, .. } = args;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let make_svc = make_service_fn(move |_| {
        let path = path.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let path = path.clone();
                async { response_file_content(path, req).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn response_file_content(
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
    
    let extension = path.extension().and_then(|extension| extension.to_str());
    let content_type = match extension {
        Some(v)=>{
            match v {
                "html" => "text/html",
                "json" => "application/json",
                "js" => "application/javascript",
                &_ => "text/plain",
            }
        },
        None => "text/plain",
    };

    if let Ok(contents) = tokio::fs::read(path).await {
        let body = contents.into();
        let builder = Response::builder();
        return Ok::<_, Infallible>(set_cors_headers(builder).header(header::CONTENT_TYPE, content_type).body(body).unwrap());
    }

    Ok::<_, Infallible>(not_found())
}

fn set_cors_headers(builder:Builder) -> Builder {
    builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "*")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}
