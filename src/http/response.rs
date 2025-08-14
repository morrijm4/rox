use std::fmt::Display;
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::{Headers, StatusCode};

pub struct Response {
    pub version: String,
    pub status_code: StatusCode,
    pub status_message: String,
    pub headers: Headers,
    pub body: String,
}

impl Response {
    pub async fn parse<R>(readable: &mut R) -> Result<Response, io::Error>
    where
        R: AsyncRead + Unpin,
    {
        let mut buf = Vec::new();
        let mut tmp = [0u8; 1024 * 4];

        let delim = "\r\n\r\n";

        while !tmp.windows(delim.len()).any(|win| win == delim.as_bytes()) {
            let n = readable.read(&mut tmp).await.unwrap_or(0);

            if n == 0 {
                break; // Connection closed
            }

            buf.extend_from_slice(&tmp[..n]);
        }

        let s = String::from_utf8_lossy(&buf);

        let (headers, mut body) = match s.split_once(delim) {
            Some((h, b)) => (h, String::from(b)),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid HTTP response",
                ));
            }
        };

        let (mut head, headers) = match headers.split_once("\r\n") {
            Some((head, headers)) => (head.split_whitespace(), headers),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid HTTP headers in response",
                ));
            }
        };

        let version = match head.next() {
            Some(v) => v.to_string(),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid HTTP version in response",
                ));
            }
        };

        let status_code = match head.next() {
            Some(code) => StatusCode::parse(code),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid HTTP status code in response",
                ));
            }
        };

        let status_message = head.collect::<Vec<_>>().join(" ");

        let headers = match Headers::parse(headers) {
            Ok(h) => h,
            Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "Invalid headers")),
        };

        let content_length = match headers.get("Content-Length") {
            Some(len) => match len.parse() {
                Ok(len) => len,
                Err(e) => {
                    let msg = "Error parsing content length";
                    eprintln!("{}: {}", msg, e);
                    return Err(io::Error::new(io::ErrorKind::Other, msg));
                }
            },
            None => usize::MAX,
        };

        while body.len() < content_length {
            let n = readable.read(&mut tmp).await.unwrap_or(0);

            if n == 0 {
                break; // Connection closed
            }

            match str::from_utf8(&tmp[..n]) {
                Ok(s) => body.push_str(s),
                Err(e) => {
                    let msg = "Error parsing response body as utf-8";
                    eprintln!("{}: {}", msg, e);
                    return Err(io::Error::new(io::ErrorKind::Other, msg));
                }
            }
        }

        Ok(Response {
            version,
            status_code,
            status_message,
            headers,
            body,
        })
    }

    pub fn from(status_code: StatusCode) -> Response {
        Response {
            version: "HTTP/1.1".into(),
            status_message: status_code.get_status_message().into(),
            status_code,
            headers: Headers::new(),
            body: String::new(),
        }
    }

    pub fn set_status_message(&mut self, status_message: String) -> &mut Response {
        self.status_message = status_message;
        self
    }

    pub async fn write<W>(&self, writable: &mut W) -> Result<(), tokio::io::Error>
    where
        W: AsyncWrite + Unpin,
    {
        writable.write_all(format!("{}", self).as_bytes()).await
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}\r\n{}\r\n{}",
            self.version, self.status_code, self.status_message, self.headers, self.body
        )
    }
}

#[derive(Debug)]
pub struct ResponseBuilder {
    version: Option<String>,
    status_code: Option<StatusCode>,
    status_message: Option<String>,
    headers: Option<Headers>,
    body: Option<String>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            version: None,
            status_code: None,
            status_message: None,
            headers: None,
            body: None,
        }
    }

    pub fn build(self) -> Result<Response, &'static str> {
        let status_code = self.status_code.ok_or("missing status code")?;
        let mut headers = self.headers.unwrap_or_else(Headers::new);
        let body = self.body.unwrap_or_else(String::new);

        if headers.get("Content-Length") == None && body.len() != 0 {
            headers.insert("Content-Length", body.len());
        }

        Ok(Response {
            version: self.version.unwrap_or("HTTP/1.1".into()),
            status_code,
            status_message: self
                .status_message
                .unwrap_or_else(|| status_code.get_status_message().into()),
            headers,
            body,
        })
    }

    pub fn add_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn add_status_code(mut self, status_code: StatusCode) -> Self {
        self.status_code = Some(status_code);
        self
    }

    pub fn add_status_message(mut self, status_message: impl Into<String>) -> Self {
        self.status_message = Some(status_message.into());
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

    #[tokio::test]
    async fn it_can_parse_a_response() {
        let body = "Hello, world!";
        let raw = format!(
            concat!(
                "HTTP/1.1 200 OK\r\n",
                "Server: Apache\r\n",
                "Date: Fri, 21 Jun 2024 12:52:39 GMT\r\n",
                "Content-Length: {}\r\n",
                "Content-Type: text/html\r\n",
                "Cache-Control: no-store\r\n",
                "\r\n",
                "{}",
            ),
            body.len(),
            body,
        );

        let res = Response::parse(&mut Cursor::new(&raw)).await.unwrap();

        assert_eq!(res.version, "HTTP/1.1");
        assert!(matches!(res.status_code, StatusCode::OK));
        assert_eq!(res.status_message, "OK");
        assert!(matches!(res.headers.get("Server"), Some(s) if s == "Apache"));
        assert!(matches!(res.headers.get("Cache-Control"), Some(s) if s == "no-store"));
        assert_eq!(res.body, body);
        assert_eq!(format!("{}", res), raw);
    }

    #[tokio::test]
    async fn it_can_parse_redirect() {
        let raw = concat!(
            "HTTP/1.0 308 Permanent Redirect\r\n",
            "Content-Type: text/plain\r\n",
            "Location: https://mattymo.dev/\r\n",
            "Refresh: 0;url=https://mattymo.dev/\r\n",
            "server: Vercel\r\n",
            "\r\n",
        );

        let res = Response::parse(&mut Cursor::new(raw)).await.unwrap();

        assert_eq!(format!("{}", res), raw);
    }

    #[tokio::test]
    async fn it_can_parse_no_status_message() {
        let raw = concat!(
            "HTTP/2 200 \r\n",
            "accept-ranges: bytes\r\n",
            "access-control-allow-origin: *\r\n",
            "age: 441818\r\n",
            "cache-control: public, max-age=0, must-revalidate\r\n",
            "content-disposition: inline\r\n",
            "content-type: text/html; charset=utf-8\r\n",
            "date: Wed, 13 Aug 2025 00:56:14 GMT\r\n",
            "etag: \"bc062ceac5bc5a3c1500479170877885\"\r\n",
            "server: Vercel\r\n",
            "strict-transport-security: max-age=63072000\r\n",
            "vary: RSC, Next-Router-State-Tree, Next-Router-Prefetch, Next-Router-Segment-Prefetch\r\n",
            "x-matched-path: /\r\n",
            "x-nextjs-prerender: 1\r\n",
            "x-nextjs-stale-time: 300\r\n",
            "x-vercel-cache: HIT\r\n",
            "x-vercel-id: iad1::wm9ss-1755046574561-a67fa49d9dae\r\n",
            "\r\n",
        );

        let req = Response::parse(&mut Cursor::new(raw)).await.unwrap();

        assert_eq!(format!("{}", req), raw);
    }
}
