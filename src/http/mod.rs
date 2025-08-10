use std::io::BufRead;
use std::io::{self, Write};
use std::net;

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub origin: String,
}

#[derive(Debug)]
pub enum Method {
    CONNECT,
}

#[repr(u16)]
#[derive(Debug)]
pub enum StatusCode {
    OK = 200,
    BadRequest = 400,
    MethodNotAllowed = 405,
}

impl Request {
    pub fn parse<R: io::Read>(input: R) -> Result<Request, StatusCode> {
        let reader = io::BufReader::new(input);

        let request: Vec<_> = reader
            .lines()
            .take_while(|line| match line {
                Ok(s) => !s.is_empty(),
                Err(_) => false,
            })
            .collect();

        println!("{:#?}", request);
        println!();

        let mut header = request
            .first()
            .ok_or(StatusCode::BadRequest)?
            .as_ref()
            .map_err(|_| StatusCode::BadRequest)?
            .split_whitespace();

        let method =
            header
                .next()
                .ok_or(StatusCode::BadRequest)
                .and_then(|method| match method {
                    "CONNECT" => Ok(Method::CONNECT),
                    _ => return Err(StatusCode::MethodNotAllowed),
                })?;

        let origin = header
            .next()
            .ok_or(StatusCode::BadRequest)
            .and_then(|o| Ok(o.to_string()))?;

        Ok(Request { method, origin })
    }
}

pub struct Response {
    code: StatusCode,
    status_message: &'static str,
}

impl Response {
    pub fn from(code: StatusCode) -> Response {
        Response {
            status_message: Response::get_status_message(&code),
            code,
        }
    }

    pub fn set_status_message(&mut self, status_message: &'static str) -> &mut Response {
        self.status_message = status_message;
        self
    }

    pub fn send(&self, stream: &mut net::TcpStream) -> Result<(), io::Error> {
        write!(
            stream,
            "HTTP/1.1 {:?} {}\r\n\r\n",
            self.code, self.status_message
        )
    }

    fn get_status_message(code: &StatusCode) -> &'static str {
        match code {
            StatusCode::OK => "OK",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
        }
    }
}
