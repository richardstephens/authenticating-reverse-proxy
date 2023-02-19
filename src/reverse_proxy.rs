use std::{convert::Infallible, net::SocketAddr};
use std::net::IpAddr;
use std::str::FromStr;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::http::{HeaderName, HeaderValue};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper_reverse_proxy;

fn debug_request(req: &Request<Body>) -> Result<Response<Body>, Infallible> {
    let body_str = format!("{:?}", req);
    Ok(Response::new(Body::from(body_str)))
}

async fn handle(client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let headers = req.headers();
    if !headers.contains_key("Authorization") {
        let mut resp = Response::new("Authentication required".into());
        *resp.status_mut() = StatusCode::UNAUTHORIZED;
        resp.headers_mut().append(HeaderName::from_str("WWW-Authenticate").unwrap(), HeaderValue::from_str("Basic").unwrap());
        return Ok(resp);
    }
    if req.uri().path().starts_with("/") {
        match hyper_reverse_proxy::call(client_ip, "http://10.0.0.8", req)
            .await
        {
            Ok(response) => {
                Ok(response)
            }
            Err(_error) => {
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap())
            }
        }
    } else {
        debug_request(&req)
    }
}

pub async fn start_reverse_proxy() {
    let bind_addr = "127.0.0.1:8000";
    let addr: SocketAddr = bind_addr.parse().expect("Could not parse ip:port.");

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr().ip();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(remote_addr, req))) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Running server on {:?}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}