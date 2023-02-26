use std::{convert::Infallible, net::SocketAddr};
use std::net::IpAddr;
use std::str::FromStr;

use http_auth_basic::Credentials;
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::http::{HeaderName, HeaderValue};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper_reverse_proxy;

use crate::github_api::check_token;

fn debug_request(req: &Request<Body>) -> Result<Response<Body>, Infallible> {
    let body_str = format!("{:?}", req);
    Ok(Response::new(Body::from(body_str)))
}

fn decode_header_b64(header_val: &HeaderValue) -> Option<Credentials> {
    match header_val.to_str() {
        Ok(header_str) => {
            match Credentials::from_header(header_str.to_string()) {
                Ok(creds) => Some(creds),
                Err(_) => Option::None
            }
        }
        Err(_) => Option::None
    }
}

async fn is_user_authenticated(header_val: &HeaderValue) -> bool {
    let credentials = decode_header_b64(&header_val);
    match credentials {
        Option::Some(creds) => {
            check_token(creds.user_id, creds.password).await
        }
        Option::None => false
    }
}

async fn handle(forward_uri: String, client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let auth_header = req.headers().get("Authorization");
    match auth_header {
        Option::None => {
            let mut resp = Response::new("Authentication required".into());
            *resp.status_mut() = StatusCode::UNAUTHORIZED;
            resp.headers_mut().append(HeaderName::from_str("WWW-Authenticate").unwrap(), HeaderValue::from_str("Basic").unwrap());
            return Ok(resp);
        }
        Option::Some(header_val) => {
            if is_user_authenticated(header_val).await {
                if req.uri().path().starts_with("/") {
                    match hyper_reverse_proxy::call(client_ip, &forward_uri, req)
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
            } else {
                let mut resp = Response::new("Authentication required".into());
                *resp.status_mut() = StatusCode::UNAUTHORIZED;
                resp.headers_mut().append(HeaderName::from_str("WWW-Authenticate").unwrap(), HeaderValue::from_str("Basic").unwrap());
                return Ok(resp);
            }
        }
    }
}

pub async fn start_reverse_proxy(forward_uri: String) {
    let bind_addr = "127.0.0.1:8000";
    let addr: SocketAddr = bind_addr.parse().expect("Could not parse ip:port.");

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr().ip();
        let our_forward_uri = forward_uri.to_string().clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(our_forward_uri.clone(), remote_addr, req))) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Running server on {:?}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}