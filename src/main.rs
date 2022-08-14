use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
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
    let Cli { port, .. } = args;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let make_svc = make_service_fn(|_| async move {
        Ok::<_, Infallible>(service_fn(move |req: Request<Body>| async move {
            let path = req.uri().path();
            let method = req.method();
            if method != Method::GET {
                return Ok::<_, Infallible>(not_found());
            }
            println!("{}", path);
            if let Ok(contents) = tokio::fs::read("./README.md").await {
                let body = contents.into();
                return Ok::<_, Infallible>(Response::new(body));
            }
            Ok::<_, Infallible>(not_found())
        }))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

/// HTTP status code 404
fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}
