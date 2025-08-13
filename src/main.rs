use std::{
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use http::{Method, Request, Response, ResponseBuilder, StatusCode};
use rustls::RootCertStore;

mod http;

fn main() {
    let address = "localhost:8080";
    let listener = TcpListener::bind(address).unwrap();

    println!("Listening at http://{}\n", address);

    for connection in listener.incoming() {
        let mut downstream = match connection {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Error accepting connection {}", e);
                continue;
            }
        };

        let request = match Request::parse(&mut downstream) {
            Ok(req) => req,
            Err(status_code) => {
                ResponseBuilder::new()
                    .add_status_code(status_code)
                    .add_header("Connection", "close")
                    .build()
                    .unwrap()
                    .write(&mut downstream)
                    .unwrap_or_else(|e| println!("Error writing to downstream connection: {}", e));

                continue;
            }
        };

        println!("{}", request);

        if request.method != Method::CONNECT {
            continue;
        }

        // Parse resource
        // let (host, _port) = match request.resource.split_once(':') {
        //     Some((host, port)) => (host.to_string().try_into().unwrap(), port),
        //     None => {
        //         ResponseBuilder::new()
        //             .add_status_code(StatusCode::BadRequest)
        //             .add_header("Connection", "close")
        //             .build()
        //             .unwrap()
        //             .write(&mut downstream)
        //             .unwrap_or_else(|e| println!("Error writing to downstream connection: {}", e));

        //         continue;
        //     }
        // };

        let mut upstream = TcpStream::connect(&request.resource).unwrap();

        // Response with 200
        let response = ResponseBuilder::new()
            .add_status_code(StatusCode::OK)
            .add_status_message("Connection Established")
            .build()
            .unwrap();
        println!("{}", response);
        response.write(&mut downstream).unwrap();

        // Parse downstream request
        let mut request = Request::parse(&mut downstream).unwrap();

        // Modify request
        request.version = "HTTP/1.0".into();
        request.headers.insert("Accept-Encoding", "identity");
        request.headers.insert("Connection", "close");
        println!("{}", request);

        // Send downstream request upstream
        request.write(&mut upstream).unwrap();

        // Parse upstream response
        let response = Response::parse(&mut upstream).unwrap();
        println!("{}", response);

        // Send upstream response downstream
        response.write(&mut downstream).unwrap_or_else(|e| {
            eprintln!("Error writing response to downstream connection: {}", e)
        });
    }
}
