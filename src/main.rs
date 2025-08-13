use http::{Method, RequestBuilder, Response};
use rustls::RootCertStore;
use std::{net::TcpStream, sync::Arc};

mod http;

fn main() {
    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let origin = "www.google.com".try_into().unwrap();
    let mut conn = rustls::ClientConnection::new(Arc::new(config), origin).unwrap();
    let mut sock = TcpStream::connect("www.google.com:443").unwrap();
    let mut tls = rustls::Stream::new(&mut conn, &mut sock);

    let request = RequestBuilder::new()
        .add_method(Method::GET)
        .add_resource("/")
        .add_version("HTTP/1.0")
        .add_header("Host", "www.google.com")
        .add_header("Connection", "close")
        .add_header("Accept-Encoding", "identity")
        .build()
        .unwrap();

    request.write(&mut tls).unwrap();

    let response = Response::parse(&mut tls).unwrap();

    println!("{}", response.body);
}
