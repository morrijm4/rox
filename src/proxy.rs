use base64::{Engine, prelude::BASE64_STANDARD};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

use crate::{
    args::Args,
    http::{Method, Request, ResponseBuilder, StatusCode},
};

pub struct Proxy {
    args: Arc<Args>,
}

impl Proxy {
    pub fn new(args: Args) -> Self {
        Self {
            args: Arc::new(args),
        }
    }

    pub async fn run(self: Self) {
        let addr = format!("localhost:{}", self.args.port);
        let listener = TcpListener::bind(&addr).await.unwrap();

        eprintln!("Listening at http://{}\n", addr);

        loop {
            let mut downstream = match listener.accept().await {
                Ok((stream, _addr)) => stream,
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                    continue;
                }
            };

            let args = self.args.clone();
            tokio::spawn(async move { handle_connection(&mut downstream, args).await });
        }
    }
}

async fn handle_connection(downstream: &mut TcpStream, args: Arc<Args>) {
    let mut request: Request;

    loop {
        request = match Request::parse(downstream).await {
            Ok(req) => req,
            Err(status_code) => {
                if status_code == StatusCode::Unknown {
                    // TODO this is a hack fix connection close handling
                    return;
                }

                return ResponseBuilder::new()
                    .add_status_code(status_code)
                    .add_header("Connection", "close")
                    .build()
                    .unwrap()
                    .write(downstream)
                    .await
                    .unwrap_or_else(|e| eprintln!("Error sending response downstream 1: {}", e));
            }
        };

        eprintln!("{}", request);

        let user_encoded = match &args.user {
            Some(u) => BASE64_STANDARD.encode(u),
            None => break,
        };

        let auth = match request.headers.get("Proxy-Authorization") {
            Some(auth) if auth.starts_with("Basic") => auth.split_whitespace().skip(1).next(),
            _ => None,
        };

        match auth {
            Some(user) if user == user_encoded => break,
            _ => {
                let res = ResponseBuilder::new()
                    .add_status_code(StatusCode::ProxyAuthenticationRequired)
                    .add_header("Proxy-Authenticate", "Basic realm=\"rox\"")
                    .build()
                    .unwrap();

                println!("{}", res);

                res.write(downstream)
                    .await
                    .unwrap_or_else(|e| eprintln!("Error sending response downstream 1: {}", e));
            }
        }
    }

    if request.method != Method::CONNECT {
        return ResponseBuilder::new()
            .add_status_code(StatusCode::MethodNotAllowed)
            .add_header("Connection", "close")
            .build()
            .unwrap()
            .write(downstream)
            .await
            .unwrap_or_else(|e| eprintln!("Error sending response downstream 2: {}", e));
    }

    let ret = TcpStream::connect(&request.resource).await.map_err(|e| {
        ResponseBuilder::new()
            .add_status_code(StatusCode::InternalServerError)
            .add_header("Connection", "close")
            .add_body(e.to_string())
            .build()
            .unwrap()
    });

    let mut upstream = match ret {
        Ok(req) => req,
        Err(res) => {
            return res
                .write(downstream)
                .await
                .unwrap_or_else(|e| eprintln!("Error sending response downstream 3: {}", e));
        }
    };

    let response = ResponseBuilder::new()
        .add_status_code(StatusCode::OK)
        .add_status_message("Connection Established")
        .build()
        .unwrap();

    eprintln!("{}", response);

    if let Err(e) = response.write(downstream).await {
        return eprintln!("Error writing response downstream: {}", e);
    }

    let ret = tokio::io::copy_bidirectional(downstream, &mut upstream).await;

    match ret {
        Ok((outgoing_bytes, incoming_bytes)) => {
            eprintln!("Outgoing bytes send: {}", outgoing_bytes);
            eprintln!("Incoming bytes send: {}", incoming_bytes);
        }
        Err(e) => eprintln!("Error with bidirection communication: {}", e),
    }
}
