use super::{Headers, StatusCode};

use std::fmt::Display;
use std::io::{self, Read, Write};
use std::u16;

pub struct Response {
    version: String,
    status_code: Option<StatusCode>,
    status_message: String,
    pub headers: Headers,
    pub body: String,
}

impl Response {
    pub fn parse(readable: &mut impl Read) -> Result<Response, io::Error> {
        let mut buf = Vec::new();
        let mut tmp = [0u8; 1024 * 4];

        let delim = "\r\n\r\n";

        while !tmp.windows(delim.len()).any(|win| win == delim.as_bytes()) {
            let n = readable.read(&mut tmp).unwrap_or(0);

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
            Some(code) => Response::parse_status_code(code),
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
            let n = readable.read(&mut tmp).unwrap_or(0);

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

    pub fn from(code: StatusCode) -> Response {
        Response {
            version: "HTTP/1.1".into(),
            status_message: Response::get_status_message(&code).into(),
            status_code: Some(code),
            headers: Headers::new(),
            body: String::new(),
        }
    }

    pub fn set_status_message(&mut self, status_message: String) -> &mut Response {
        self.status_message = status_message;
        self
    }

    pub fn write(&self, writable: &mut impl Write) -> Result<(), io::Error> {
        write!(writable, "{}", self)
    }

    fn get_status_message(code: &StatusCode) -> &'static str {
        match code {
            // Informational
            StatusCode::Continue => "Continue",
            StatusCode::SwitchingProtocols => "Switching Protocols",
            StatusCode::Processing => "Processing",
            StatusCode::EarlyHints => "Early Hints",

            // Successful
            StatusCode::OK => "OK",
            StatusCode::Created => "Created",
            StatusCode::Accepted => "Accepted",
            StatusCode::NonAuthoritativeInformation => "Non Authoritative Information",
            StatusCode::NoContent => "No Content",
            StatusCode::ResetContent => "Reset Content",
            StatusCode::PartialContent => "Partial Content",
            StatusCode::MultiStatus => "Multi-Status",
            StatusCode::AlreadyReported => "Already Reported",
            StatusCode::IMUsed => "IM Used",

            // Redirection
            StatusCode::MultipleChoices => "Multiple Choices",
            StatusCode::MovedPermanently => "Moved Permanently",
            StatusCode::Found => "Found",
            StatusCode::SeeOther => "See Other",
            StatusCode::NotModified => "Not Modified",
            StatusCode::UseProxy => "Use Proxy",
            StatusCode::Unused => "Unused",
            StatusCode::TemporaryRedirect => "Temporary Redirect",
            StatusCode::PermanentRedirect => "Permanent Redirect",

            // Client Errors
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::PaymentRequired => "Payment Required",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::NotAcceptable => "Not Acceptable",
            StatusCode::ProxyAuthenticationRequired => "Proxy Authentication Required",
            StatusCode::RequestTimeout => "Request Timeout",
            StatusCode::Conflict => "Conflict",
            StatusCode::Gone => "Gone",
            StatusCode::LengthRequired => "Length Required",
            StatusCode::PreconditionFailed => "Precondition Failed",
            StatusCode::ContentTooLarge => "Content Too Large",
            StatusCode::URLTooLong => "URL Too Long",
            StatusCode::UnsupportedMediaType => "Unsupported Media Type",
            StatusCode::RangeNotSatisfiable => "Range Not Satisfiable",
            StatusCode::ExpectationFailed => "Expectation Failed",
            StatusCode::ImATeapot => "I'm a teapot",
            StatusCode::MisdirectedRequest => "Misdirected Request",
            StatusCode::UnprocessableContent => "Unprocessable Content",
            StatusCode::Locked => "Locked",
            StatusCode::FailedDependency => "Failed Dependency",
            StatusCode::TooEarly => "Too Early",
            StatusCode::UpgradeRequired => "Upgrade Required",
            StatusCode::PreconditionRequired => "Precondition Required",
            StatusCode::TooManyRequests => "Too Many Requests",
            StatusCode::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            StatusCode::UnavailableForLegalReasons => "Unavailable For Legal Reasons",

            // Server Errors
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::BadGateway => "Bad Gateway",
            StatusCode::ServiceUnavailable => "Service Unavailable",
            StatusCode::GatewayTimeout => "Gateway Timeout",
            StatusCode::HTTPVersionNotSupported => "HTTP Version Not Supported",
            StatusCode::VariantAlsoNegotiates => "Variant Also Negotiates",
            StatusCode::InsufficientStorage => "Insufficient Storage",
            StatusCode::LoopDetected => "Loop Detected",
            StatusCode::NotExtended => "Not Extended",
            StatusCode::NetworkAuthenticationRequired => "Network Authentication Required",
        }
    }

    fn parse_status_code(code: &str) -> Option<StatusCode> {
        match code {
            // Informational
            "100" => Some(StatusCode::Continue),
            "101" => Some(StatusCode::SwitchingProtocols),
            "102" => Some(StatusCode::Processing),
            "103" => Some(StatusCode::EarlyHints),

            // Successful
            "200" => Some(StatusCode::OK),
            "201" => Some(StatusCode::Created),
            "202" => Some(StatusCode::Accepted),
            "203" => Some(StatusCode::NonAuthoritativeInformation),
            "204" => Some(StatusCode::NoContent),
            "205" => Some(StatusCode::ResetContent),
            "206" => Some(StatusCode::PartialContent),
            "207" => Some(StatusCode::MultiStatus),
            "208" => Some(StatusCode::AlreadyReported),
            "226" => Some(StatusCode::IMUsed),

            // Redirection
            "300" => Some(StatusCode::MultipleChoices),
            "301" => Some(StatusCode::MovedPermanently),
            "302" => Some(StatusCode::Found),
            "303" => Some(StatusCode::SeeOther),
            "304" => Some(StatusCode::NotModified),
            "305" => Some(StatusCode::UseProxy),
            "306" => Some(StatusCode::Unused),
            "307" => Some(StatusCode::TemporaryRedirect),
            "308" => Some(StatusCode::PermanentRedirect),

            // Client Error
            "400" => Some(StatusCode::BadRequest),
            "401" => Some(StatusCode::Unauthorized),
            "404" => Some(StatusCode::NotFound),
            "405" => Some(StatusCode::MethodNotAllowed),
            "406" => Some(StatusCode::NotAcceptable),
            "407" => Some(StatusCode::ProxyAuthenticationRequired),
            "410" => Some(StatusCode::Gone),
            "411" => Some(StatusCode::LengthRequired),
            "408" => Some(StatusCode::RequestTimeout),
            "409" => Some(StatusCode::Conflict),
            "412" => Some(StatusCode::PreconditionFailed),
            "413" => Some(StatusCode::ContentTooLarge),
            "414" => Some(StatusCode::URLTooLong),
            "415" => Some(StatusCode::UnsupportedMediaType),
            "416" => Some(StatusCode::RangeNotSatisfiable),
            "417" => Some(StatusCode::ExpectationFailed),
            "418" => Some(StatusCode::ImATeapot),
            "421" => Some(StatusCode::MisdirectedRequest),
            "422" => Some(StatusCode::UnprocessableContent),
            "423" => Some(StatusCode::Locked),
            "424" => Some(StatusCode::FailedDependency),
            "425" => Some(StatusCode::TooEarly),
            "426" => Some(StatusCode::UpgradeRequired),
            "428" => Some(StatusCode::PreconditionRequired),
            "429" => Some(StatusCode::TooManyRequests),
            "431" => Some(StatusCode::RequestHeaderFieldsTooLarge),
            "451" => Some(StatusCode::UnavailableForLegalReasons),

            // Server Error
            "500" => Some(StatusCode::InternalServerError),
            "501" => Some(StatusCode::NotImplemented),
            "502" => Some(StatusCode::BadGateway),
            "503" => Some(StatusCode::ServiceUnavailable),
            "504" => Some(StatusCode::GatewayTimeout),
            "505" => Some(StatusCode::HTTPVersionNotSupported),
            "506" => Some(StatusCode::VariantAlsoNegotiates),
            "507" => Some(StatusCode::InsufficientStorage),
            "508" => Some(StatusCode::LoopDetected),
            "510" => Some(StatusCode::NotExtended),
            "511" => Some(StatusCode::NetworkAuthenticationRequired),
            _ => None,
        }
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_code: u16 = match self.status_code {
            Some(code) => code as u16,
            None => 999,
        };

        write!(
            f,
            "{} {} {}\r\n{}\r\n{}",
            self.version, status_code, self.status_message, self.headers, self.body
        )
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn it_can_parse_a_response() {
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

        let res = Response::parse(&mut Cursor::new(&raw)).unwrap();

        assert_eq!(res.version, "HTTP/1.1");
        assert!(matches!(res.status_code, Some(StatusCode::OK)));
        assert_eq!(res.status_message, "OK");
        assert!(matches!(res.headers.get("Server"), Some(s) if s == "Apache"));
        assert!(matches!(res.headers.get("Cache-Control"), Some(s) if s == "no-store"));
        assert_eq!(res.body, body);
        assert_eq!(format!("{}", res), raw);
    }

    #[test]
    fn it_can_parse_redirect() {
        let raw = concat!(
            "HTTP/1.0 308 Permanent Redirect\r\n",
            "Content-Type: text/plain\r\n",
            "Location: https://mattymo.dev/\r\n",
            "Refresh: 0;url=https://mattymo.dev/\r\n",
            "server: Vercel\r\n",
            "\r\n",
        );

        let res = Response::parse(&mut Cursor::new(raw)).unwrap();

        assert_eq!(format!("{}", res), raw);
    }

    #[test]
    fn it_can_parse_no_status_message() {
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

        let req = Response::parse(&mut Cursor::new(raw)).unwrap();

        assert_eq!(format!("{}", req), raw);
    }
}
