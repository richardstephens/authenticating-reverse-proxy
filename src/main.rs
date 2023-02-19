use std::{convert::Infallible, net::SocketAddr};
use std::net::IpAddr;
use std::str::FromStr;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::http::{HeaderName, HeaderValue};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper_reverse_proxy;

use reverse_proxy::start_reverse_proxy;

mod reverse_proxy;

#[tokio::main]
async fn main() {
    start_reverse_proxy().await
}
