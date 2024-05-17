use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{body::Bytes, client::conn, Request, Response};

use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

pub async fn proxy_response(
    req: Request<hyper::body::Incoming>,
    proxies: &Option<Vec<(String, String)>>,
) -> Option<Response<BoxBody<Bytes, hyper::Error>>> {
    let uri = req.uri().path();
    if let Some(proxies) = proxies {
        for (api, origin) in proxies {
            if uri.starts_with(api) {
                let headers = req.headers().clone();
                let mut request_builder = Request::builder()
                    .method(req.method())
                    .uri(origin.strip_suffix('/').unwrap_or(origin).to_owned() + uri)
                    .body(req.into_body())
                    .unwrap();
                *request_builder.headers_mut() = headers;

                let stream = TcpStream::connect(
                    origin
                        .strip_prefix("http://")
                        .or_else(|| origin.strip_prefix("https://"))
                        .unwrap_or(origin),
                )
                .await
                .unwrap();

                let io = TokioIo::new(stream);
                let (mut sender, conn) = conn::http1::handshake(io).await.unwrap();

                tokio::task::spawn(async move {
                    conn.await.unwrap();
                });

                let response = sender.send_request(request_builder).await.unwrap();
                let body_bytes = response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .map_err(|never| match never {})
                    .boxed();

                return Some(Response::new(body_bytes));
            }
        }
    }
    None
}
