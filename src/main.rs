use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net,
};

mod http;

fn main() {
    run();
}

fn _request_google() -> Result<(), io::Error> {
    let mut stream = net::TcpStream::connect("google.com:80")?;

    let request = b"GET / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: rox\r\nAccept: */*\r\n\r\n";
    stream.write_all(request)?;

    let mut reader = BufReader::new(&stream);
    let mut buf = String::new();

    while reader.read_line(&mut buf)? > 0 {
        print!("{}", buf);
        buf.clear();
    }

    Ok(())
}

fn run() {
    let port = 8080;
    let address = format!("127.0.0.1:{}", port);
    let listener = net::TcpListener::bind(&address).unwrap();

    println!("Listening at http://{}", address);

    for connection in listener.incoming() {
        if let Err(e) = handle_connection(connection) {
            eprint!("Error handling connection {}", e);
        }
    }
}

fn handle_connection(connection: Result<net::TcpStream, io::Error>) -> Result<(), io::Error> {
    let mut stream = connection?;

    let request = match http::Request::parse(&stream) {
        Ok(req) => req,
        Err(code) => return http::Response::from(code).send(&mut stream),
    };

    assert_eq!(request.method, http::Method::CONNECT);

    let mut upstream = match net::TcpStream::connect(request.path) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Error connecting to upstream server: {}", e);
            return http::Response::from(http::StatusCode::BadGateway).send(&mut stream);
        }
    };

    http::Response::from(http::StatusCode::OK)
        .set_status_message("Connection Established")
        .send(&mut stream)?;

    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024 * 16];

    while !buf.ends_with("\r\n\r\n".as_bytes()) {
        let n = stream.read(&mut tmp)?;

        if n == 0 {
            break; // Connection closed
        }

        buf.extend_from_slice(&tmp[..n]);
    }

    println!("{}", String::from_utf8_lossy(&buf));
    upstream.write_all(&buf)?;
    buf.clear();

    let delim = "\r\n\r\n".as_bytes();

    while !buf.windows(delim.len()).any(|win| win == delim) {
        let n = upstream.read(&mut tmp)?;

        println!("read {} bytes", n);

        if n == 0 {
            break;
        }

        buf.extend_from_slice(&tmp[..n]);
    }

    println!("done.");
    println!("{}", String::from_utf8_lossy(&buf));
    stream.write_all(&buf)?;

    Ok(())
}
