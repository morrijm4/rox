use tokio::net::{TcpListener, TcpStream};

use crate::{
    args::Args,
    http::{Method, Request, ResponseBuilder, StatusCode},
};

pub struct Cli {}

impl Cli {
    pub async fn start() {
        let args = match Args::parse() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", e);
                return Self::help();
            }
        };

        Self::start_with(args).await;
    }

    pub async fn start_with(args: Args) {
        if args.help {
            return Self::help();
        }

        if args.version {
            return Self::version();
        }

        let addr = format!("localhost:{}", args.port);
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

            tokio::spawn(async move { handle_connection(&mut downstream).await });
        }
    }

    fn version() {
        println!("{}", env!("CARGO_PKG_VERSION"))
    }

    fn help() {
        println!(
            "
USAGE: rox[EXE] [OPTIONS] [-P | --protocol]=<PROTOCOL>

PROTOCOLS:
    http (default)

OPTIONS:
    -h, --help              Print help
    -v, --version           Print version
    -p, --port=<PORT>       Specify port for proxy server (default: protocol convention)
"
        )
    }
}

async fn handle_connection(downstream: &mut TcpStream) {
    let ret = Request::parse(downstream).await.map_err(|status_code| {
        ResponseBuilder::new()
            .add_status_code(status_code)
            .add_header("Connection", "close")
            .build()
            .unwrap()
    });

    let request = match ret {
        Ok(req) => req,
        Err(res) => {
            return res
                .write(downstream)
                .await
                .unwrap_or_else(|e| eprintln!("Error sending response downstream: {}", e));
        }
    };

    eprintln!("{}", request);

    if request.method != Method::CONNECT {
        return ResponseBuilder::new()
            .add_status_code(StatusCode::MethodNotAllowed)
            .add_header("Connection", "close")
            .build()
            .unwrap()
            .write(downstream)
            .await
            .unwrap_or_else(|e| eprintln!("Error sending response downstream: {}", e));
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
                .unwrap_or_else(|e| eprintln!("Error sending response downstream: {}", e));
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

    let ret = tokio::io::copy_bidirectional(downstream, &mut upstream)
        .await
        .map_err(|e| {
            eprintln!("Error with bidirection communication: {}", e);
        });

    if let Ok((outgoing_bytes, incoming_bytes)) = ret {
        eprintln!("Outgoing bytes send: {}", outgoing_bytes);
        eprintln!("Incoming bytes send: {}", incoming_bytes);
    }
}
