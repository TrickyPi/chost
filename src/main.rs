use clap::Parser;
use hyper::header::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Client, Server};
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chost::addr::{Addr, Port};
use chost::response::{file::response_file_content, proxy::proxy_response};
use chost::utils::get_full_addr_string;

/// Not only can it host static files
#[derive(Parser)]
struct Cli {
    /// path to host
    #[clap(parse(from_os_str))]
    path: Option<PathBuf>,
    /// enable cors
    #[clap(short, long)]
    cors: bool,
    /// port
    #[clap(short, long, default_value_t = 7878)]
    port: Port,
    /// forwarding request to other service, the format is "${api}|${origin} ${api}|${origin}"
    #[clap(long, value_delimiter = ' ')]
    proxy: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    create_server(args).await;
}

async fn create_server(args: Cli) {
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
    let cors_arc = Arc::new(cors);

    let client = Client::builder()
        .pool_idle_timeout(Duration::from_secs(1000))
        .build_http::<hyper::Body>();

    let make_svc = make_service_fn(|_| {
        let path = path.clone();
        let cors_arc = cors_arc.clone();
        let proxies_arc = proxies_arc.clone();
        let client = client.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let req_path = req.uri().path().strip_prefix('/').unwrap().to_owned();
                let method = req.method().clone();

                let path = path.clone();
                let cors_arc = cors_arc.clone();
                let proxies_arc = proxies_arc.clone();
                let client = client.clone();

                async move {
                    let mut resp =
                        if let Some(resp) = proxy_response(client, req, &proxies_arc).await {
                            resp
                        } else {
                            response_file_content(path, method, req_path).await
                        };
                    if *cors_arc {
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
                        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, not_limited_value);
                    }
                    Ok::<_, Infallible>(resp)
                }
            }))
        }
    });

    let addr = Addr::new();
    let (local_addr, network_addr) = addr.get_address(port);

    let local_server = Server::bind(&local_addr).serve(make_svc);
    let network_server = Server::bind(&network_addr).serve(make_svc);

    println!("local server on {}", get_full_addr_string(&local_addr));
    println!("network server on {}", get_full_addr_string(&network_addr));

    if let Err(e) = tokio::try_join!(local_server, network_server) {
        eprintln!("server error: {}", e);
    }
}
