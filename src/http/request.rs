use std::io;
use std::{fmt::Display, io::Write};

use super::{Headers, StatusCode};

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub resource: String,
    pub version: String,
    pub headers: Headers,
    pub body: String,
}

impl Request {
    pub fn parse<R: io::Read>(input: &mut R) -> Result<Request, StatusCode> {
        let mut buf = Vec::new();
        let mut tmp = [0u8; 1024];

        let delim = "\r\n\r\n";

        while !tmp.windows(delim.len()).any(|win| win == delim.as_bytes()) {
            let n = input.read(&mut tmp).map_err(|e| {
                eprintln!("Error reading from socket: {}", e);
                StatusCode::InternalServerError
            })?;

            if n == 0 {
                break; // Connection closed
            }

            buf.extend_from_slice(&tmp[..n]);
        }

        let s = str::from_utf8(&buf).map_err(|e| {
            eprintln!("Error converting to utf-8: {}", e);
            StatusCode::BadRequest
        })?;

        let (headers, mut body) = match s.split_once(delim) {
            Some((h, b)) => (h, String::from(b)),
            None => {
                eprintln!("Error splitting headers");
                return Err(StatusCode::BadRequest);
            }
        };

        let (head, headers) = headers.split_once("\r\n").ok_or_else(|| {
            eprintln!("Error splitting head");
            StatusCode::BadRequest
        })?;

        let mut head = head.split_whitespace();

        let method = match head.next() {
            Some(method) => Method::parse(method).ok_or_else(|| {
                eprintln!("Unknown method: {}", method);
                StatusCode::MethodNotAllowed
            })?,
            None => {
                eprintln!("Error parsing method");
                return Err(StatusCode::BadRequest);
            }
        };

        let resource = match head.next() {
            Some(resource) => String::from(resource),
            None => {
                eprintln!("Invalid resource");
                return Err(StatusCode::BadRequest);
            }
        };

        let version = match head.next() {
            Some(version) => String::from(version),
            None => {
                eprintln!("Invalid version");
                return Err(StatusCode::BadRequest);
            }
        };

        let headers = Headers::parse(headers)?;

        // Get the content length
        let content_length: usize = match headers.get("Content-Length") {
            Some(len) => len.parse().map_err(|e| {
                eprintln!("Error parsing content length: {}", e);
                StatusCode::BadRequest
            })?,
            None => 0, // Don't parse body
        };

        // Read the body if exists
        while body.len() < content_length {
            let n = input.read(&mut tmp).map_err(|e| {
                eprintln!("Error reading body: {}", e);
                StatusCode::BadRequest
            })?;

            if n == 0 {
                break; // Closed connection
            }

            let s = str::from_utf8(&tmp[..n]).map_err(|e| {
                eprintln!("Error parsing body as utf-8: {}", e);
                StatusCode::BadRequest
            })?;

            body.push_str(s);
        }

        Ok(Request {
            method,
            resource,
            version,
            headers,
            body,
        })
    }

    pub fn write<W: Write>(&self, writable: &mut W) -> Result<(), io::Error> {
        write!(writable, "{}", self)
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}\r\n{}\r\n{}",
            self.method, self.resource, self.version, self.headers, self.body
        )
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Method {
    CONNECT,
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
    TRACE,
}

impl Method {
    pub fn parse(method: &str) -> Option<Self> {
        match method {
            "GET" => Some(Method::GET),
            "PUT" => Some(Method::PUT),
            "HEAD" => Some(Method::HEAD),
            "POST" => Some(Method::POST),
            "PATCH" => Some(Method::PATCH),
            "DELETE" => Some(Method::DELETE),
            "OPTIONS" => Some(Method::OPTIONS),
            "CONNECT" => Some(Method::CONNECT),
            "TRACE" => Some(Method::TRACE),
            _ => None,
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Method::CONNECT => "CONNECT",
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::PATCH => "PATCH",
            Method::DELETE => "DELETE",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
            Method::TRACE => "TRACE",
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub struct RequestBuilder {
    method: Option<Method>,
    resource: Option<String>,
    version: Option<String>,
    headers: Option<Headers>,
    body: Option<String>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            method: None,
            resource: None,
            version: None,
            headers: None,
            body: None,
        }
    }

    pub fn build(self) -> Result<Request, &'static str> {
        Ok(Request {
            method: self.method.ok_or("missing method")?,
            resource: self.resource.ok_or("missing resource")?,
            version: self.version.unwrap_or("HTTP/1.1".into()),
            headers: self.headers.unwrap_or(Headers::new()),
            body: self.body.unwrap_or(String::new()),
        })
    }

    pub fn add_method(mut self, method: Method) -> Self {
        self.method = Some(method);
        self
    }

    pub fn add_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    pub fn add_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn add_headers(mut self, headers: Headers) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn add_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: ToString,
    {
        self.headers
            .get_or_insert_with(Headers::new)
            .insert(key, value);
        self
    }

    pub fn add_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn it_can_parse() {
        let raw_req = concat!(
            "GET / HTTP/1.1\r\n",
            "Host: mattymo.dev\r\n",
            "Accept: */*\r\n",
            "Connection: close\r\n",
            "\r\n",
        );

        let req = Request::parse(&mut Cursor::new(raw_req)).unwrap();

        assert_eq!(req.method, Method::GET);
        assert_eq!(req.resource, "/");
        assert_eq!(req.version, "HTTP/1.1");
        assert!(matches!(req.headers.get("host"), Some(value) if value == "mattymo.dev"));
        assert!(matches!(req.headers.get("accept"), Some(value) if value == "*/*"));
        assert!(matches!(req.headers.get("connection"), Some(value) if value == "close"));
    }

    #[test]
    fn it_can_parse_a_body() {
        let body = "Hello, world!";
        let raw_req: String = format!(
            concat!(
                "GET / HTTP/1.1\r\n",
                "Host: mattymo.dev\r\n",
                "Accept: */*\r\n",
                "Connection: close\r\n",
                "Content-Length: {}\r\n",
                "\r\n",
                "{}",
            ),
            body.len(),
            body,
        );

        let req = Request::parse(&mut Cursor::new(raw_req)).unwrap();

        assert_eq!(req.method, Method::GET);
        assert_eq!(req.resource, "/");
        assert_eq!(req.version, "HTTP/1.1");
        assert!(matches!(req.headers.get("host"), Some(value) if value == "mattymo.dev"));
        assert!(matches!(req.headers.get("accept"), Some(value) if value == "*/*"));
        assert!(matches!(req.headers.get("connection"), Some(value) if value == "close"));
        assert_eq!(req.body, body);
    }

    #[test]
    fn it_can_parse_connect_method() {
        let raw_req = concat!(
            "CONNECT google.com:80 HTTP/1.1\r\n",
            "Host: google.com:80\r\n",
            "User-Agent: curl/8.7.1\r\n",
            "Proxy-Connection: Keep-Alive\r\n",
            "\r\n",
        );

        let req = Request::parse(&mut Cursor::new(raw_req)).unwrap();

        assert_eq!(req.method, Method::CONNECT);
        assert_eq!(req.resource, "google.com:80");
        assert_eq!(req.version, "HTTP/1.1");
        assert!(matches!(req.headers.get("host"), Some(value) if value == "google.com:80"));
        assert!(matches!(req.headers.get("user-agent"), Some(value) if value == "curl/8.7.1"));
        assert!(
            matches!(req.headers.get("proxy-connection"), Some(value) if value == "Keep-Alive")
        );
    }

    #[test]
    fn it_can_parse_large_body() {
        let body = "a".repeat(1024 * 2);
        let raw_req: String = format!(
            concat!(
                "POST /data HTTP/1.1\r\n",
                "Host: mattymo.dev\r\n",
                "Accept: */*\r\n",
                "Connection: close\r\n",
                "Content-Length: {}\r\n",
                "\r\n",
                "{}",
            ),
            body.len(),
            body,
        );

        let req = Request::parse(&mut Cursor::new(raw_req)).unwrap();

        assert_eq!(req.method, Method::POST);
        assert_eq!(req.resource, "/data");
        assert_eq!(req.body, body);
    }

    #[test]
    fn it_can_display_a_large_body() {
        let body = "a".repeat(1024 * 2);
        let raw_req: String = format!(
            concat!(
                "POST /data HTTP/1.1\r\n",
                "Host: mattymo.dev\r\n",
                "Accept: */*\r\n",
                "Connection: close\r\n",
                "Content-Length: {}\r\n",
                "\r\n",
                "{}",
            ),
            body.len(),
            body,
        );

        let req = Request::parse(&mut Cursor::new(&raw_req)).unwrap();

        assert_eq!(format!("{}", req), raw_req);
    }

    #[test]
    fn it_can_build_a_request() {
        let request = RequestBuilder::new()
            .add_method(Method::GET)
            .add_resource("/")
            .add_header("Host", "example.com")
            .add_header("Connection", "close")
            .add_header("Accept", "*/*")
            .build()
            .unwrap();

        let expected = concat!(
            "GET / HTTP/1.1\r\n",
            "Host: example.com\r\n",
            "Connection: close\r\n",
            "Accept: */*\r\n",
            "\r\n",
        );

        assert_eq!(format!("{}", request), expected);
    }

    #[test]
    fn it_can_build_a_request_with_body() {
        let body = "My super awesome post for my blog";

        let mut headers = Headers::new();
        headers.insert("Host", "example.com");
        headers.insert("Connection", "close");
        headers.insert("Accept", "*/*");
        headers.insert("Content-Length", body.len());
        headers.insert("Content-Type", "text/plain");

        let request = RequestBuilder::new()
            .add_method(Method::POST)
            .add_resource("/api/posts")
            .add_version("HTTP/2")
            .add_headers(headers)
            .add_body(body)
            .build()
            .unwrap();

        let expected = format!(
            concat!(
                "POST /api/posts HTTP/2\r\n",
                "Host: example.com\r\n",
                "Connection: close\r\n",
                "Accept: */*\r\n",
                "Content-Length: {}\r\n",
                "Content-Type: text/plain\r\n",
                "\r\n",
                "{}",
            ),
            body.len(),
            body,
        );

        assert_eq!(format!("{}", request), expected);
    }
}
