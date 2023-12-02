use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;

mod addr;
use addr::{Addr, Port};

mod response;
use response::response_file_content;

mod utils;
use utils::get_full_addr_string;

/// host static files
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
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    create_server(args).await;
}

async fn create_server(args: Cli) {
    let Cli { port, path, cors } = args;

    let path = match path {
        Some(path) => path,
        None => PathBuf::from("./"),
    };

    let cors_arc = Arc::new(cors);

    let make_svc = make_service_fn(|_| {
        let path = path.clone();
        let cors_arc = cors_arc.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let path = path.clone();
                let cors_arc = cors_arc.clone();
                async { response_file_content(path, req, cors_arc).await }
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
