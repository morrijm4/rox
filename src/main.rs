use std::net::TcpListener;

mod http;

fn main() {
    let port = 8080;
    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&address).unwrap();

    println!("Listening at http://{}", address);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        let request = match http::Request::parse(&stream) {
            Ok(req) => req,
            Err(code) => {
                if let Err(e) = http::Response::from(code).send(&mut stream) {
                    eprintln!("Error sending response: {}", e);
                }

                continue;
            }
        };

        let result = match request.method {
            http::Method::CONNECT => http::Response::from(http::StatusCode::OK)
                .set_status_message("Connection Established")
                .send(&mut stream),
        };

        if let Err(e) = result {
            eprintln!("Error sending response: {}", e);
        };
    }

