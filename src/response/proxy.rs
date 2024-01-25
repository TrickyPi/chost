use hyper::{client::HttpConnector, Body, Client, Request, Response, StatusCode};

pub async fn proxy_response(
    client: Client<HttpConnector>,
    req: Request<Body>,
    proxies: &Option<Vec<(String, String)>>,
) -> Option<Response<Body>> {
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
                let response = client.request(request_builder).await.unwrap();
                let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
                let body = String::from_utf8(body.to_vec()).unwrap();

                let mut resp = Response::new(Body::from(body));
                *resp.status_mut() = StatusCode::OK;
                return Some(resp);
            }
        }
    }
    None
}
