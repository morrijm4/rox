mod headers;
mod request;
mod response;

use std::fmt::Display;

pub use headers::*;
pub use request::*;
pub use response::*;

#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum StatusCode {
    Unknown = 999,

    // Informational
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,
    EarlyHints = 103,

    // Successful
    OK = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    IMUsed = 226,

    // Redirection
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    Unused = 306,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    // Client Error
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    ContentTooLarge = 413,
    URLTooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableContent = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,

    // Server Error
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,
}

impl StatusCode {
    fn parse(code: &str) -> StatusCode {
        match code {
            // Informational
            "100" => StatusCode::Continue,
            "101" => StatusCode::SwitchingProtocols,
            "102" => StatusCode::Processing,
            "103" => StatusCode::EarlyHints,

            // Successful
            "200" => StatusCode::OK,
            "201" => StatusCode::Created,
            "202" => StatusCode::Accepted,
            "203" => StatusCode::NonAuthoritativeInformation,
            "204" => StatusCode::NoContent,
            "205" => StatusCode::ResetContent,
            "206" => StatusCode::PartialContent,
            "207" => StatusCode::MultiStatus,
            "208" => StatusCode::AlreadyReported,
            "226" => StatusCode::IMUsed,

            // Redirection
            "300" => StatusCode::MultipleChoices,
            "301" => StatusCode::MovedPermanently,
            "302" => StatusCode::Found,
            "303" => StatusCode::SeeOther,
            "304" => StatusCode::NotModified,
            "305" => StatusCode::UseProxy,
            "306" => StatusCode::Unused,
            "307" => StatusCode::TemporaryRedirect,
            "308" => StatusCode::PermanentRedirect,

            // Client Error
            "400" => StatusCode::BadRequest,
            "401" => StatusCode::Unauthorized,
            "404" => StatusCode::NotFound,
            "405" => StatusCode::MethodNotAllowed,
            "406" => StatusCode::NotAcceptable,
            "407" => StatusCode::ProxyAuthenticationRequired,
            "410" => StatusCode::Gone,
            "411" => StatusCode::LengthRequired,
            "408" => StatusCode::RequestTimeout,
            "409" => StatusCode::Conflict,
            "412" => StatusCode::PreconditionFailed,
            "413" => StatusCode::ContentTooLarge,
            "414" => StatusCode::URLTooLong,
            "415" => StatusCode::UnsupportedMediaType,
            "416" => StatusCode::RangeNotSatisfiable,
            "417" => StatusCode::ExpectationFailed,
            "418" => StatusCode::ImATeapot,
            "421" => StatusCode::MisdirectedRequest,
            "422" => StatusCode::UnprocessableContent,
            "423" => StatusCode::Locked,
            "424" => StatusCode::FailedDependency,
            "425" => StatusCode::TooEarly,
            "426" => StatusCode::UpgradeRequired,
            "428" => StatusCode::PreconditionRequired,
            "429" => StatusCode::TooManyRequests,
            "431" => StatusCode::RequestHeaderFieldsTooLarge,
            "451" => StatusCode::UnavailableForLegalReasons,

            // Server Error
            "500" => StatusCode::InternalServerError,
            "501" => StatusCode::NotImplemented,
            "502" => StatusCode::BadGateway,
            "503" => StatusCode::ServiceUnavailable,
            "504" => StatusCode::GatewayTimeout,
            "505" => StatusCode::HTTPVersionNotSupported,
            "506" => StatusCode::VariantAlsoNegotiates,
            "507" => StatusCode::InsufficientStorage,
            "508" => StatusCode::LoopDetected,
            "510" => StatusCode::NotExtended,
            "511" => StatusCode::NetworkAuthenticationRequired,
            _ => StatusCode::Unknown,
        }
    }

    fn get_status_message(&self) -> &'static str {
        match self {
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
            StatusCode::Unknown => "Unknown",
            _ => "Unknown",
        }
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone() as u16)
    }
}
