use clap::Parser;
use hyper::header;
use hyper::header::HeaderValue;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;

use chost::addr::{Addr, Port};
use chost::response::{file::response_file_content, proxy::proxy_response};
use chost::utils::get_full_addr_string;

/// Not only can it host static files
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// path to host
    path: Option<PathBuf>,
    /// enable cors
    #[arg(short, long)]
    cors: bool,
    /// port
    #[arg(short, long, default_value_t = 7878)]
    port: Port,
    /// forwarding request to other service, the format is "${api}|${origin} ${api}|${origin}"
    #[arg(long, value_delimiter = ' ')]
    proxy: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    if let Err(e) = create_server(args).await {
        eprintln!("server error: {}", e);
    };
}

async fn create_server(args: Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Cli {
        port,
        path,
        cors,
        proxy,
    } = args;

    let path = match path {
        Some(path) => path,
        None => PathBuf::from("./"),
    };

    let proxies = proxy.map(|proxies| {
        proxies
            .iter()
            .map(|proxy| {
                let parts: Vec<&str> = proxy.split('|').take(2).collect();
                if let [api, origin] = parts[..] {
                    (api.to_owned(), origin.to_owned())
                } else {
                    panic!(
                        "invalid proxy config '{}', the right format is {{api}}|{{origin}}",
                        proxy
                    );
                }
            })
            .collect::<Vec<(String, String)>>()
    });

    let proxies_arc = Arc::new(proxies);

    let addr = Addr::new();
    let (local_addr, network_addr, bind_addr) = addr.get_address(port);

    let listener = TcpListener::bind(bind_addr).await?;

    println!("local server on {}", get_full_addr_string(&local_addr));
    println!("network server on {}", get_full_addr_string(&network_addr));

    loop {
        let path = path.clone();
        let proxies_arc = proxies_arc.clone();

        let (stream, _addr) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        let req_path = req.uri().path().strip_prefix('/').unwrap().to_owned();
                        let method = req.method().clone();

                        let path = path.clone();
                        let proxies_arc = proxies_arc.clone();

                        async move {
                            let mut resp =
                                if let Some(resp) = proxy_response(req, &proxies_arc).await {
                                    resp
                                } else {
                                    response_file_content(path, method, req_path).await
                                };
                            if cors {
                                let headers = resp.headers_mut();
                                let not_limited_value = HeaderValue::from_static("*");
                                headers.insert(
                                    header::ACCESS_CONTROL_ALLOW_ORIGIN,
                                    not_limited_value.clone(),
                                );
                                headers.insert(
                                    header::ACCESS_CONTROL_ALLOW_METHODS,
                                    not_limited_value.clone(),
                                );
                                headers.insert(
                                    header::ACCESS_CONTROL_ALLOW_HEADERS,
                                    not_limited_value,
                                );
                            }
                            Ok::<_, Infallible>(resp)
                        }
                    }),
                )
                .await
        });
    }
}
