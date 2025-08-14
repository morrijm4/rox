use http::{Method, Request, ResponseBuilder, StatusCode};
use std::error;
use tokio::net::{TcpListener, TcpStream};

mod http;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let addr = "localhost:8080";
    let listener = TcpListener::bind(addr).await?;

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
