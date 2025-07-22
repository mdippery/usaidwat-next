//! Services for communicating with APIs using HTTP.

use reqwest::header;
use std::{error, fmt};

/// A general service for making HTTP calls.
///
/// It might be a bit odd to refer to this trait as a "service", since
/// it appears to be more of a _client_ implementation, but think of
/// this as a proxy for a remote _service_ (even though a _client_ is used
/// to communicate with that remote service). A service might not always
/// be remote, such as when the implementation is a deterministic service
/// used for testing.
pub trait HTTPService {
    /// An appropriate user agent to use when making HTTP requests.
    fn user_agent() -> String {
        format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}

/// The result of an HTTP request.
pub type HTTPResult<T> = Result<T, HTTPError>;

/// Indicates an error has occurred when making an HTTP call.
#[derive(Debug)]
pub enum HTTPError {
    /// An error retrieving the body of a response.
    Body(reqwest::Error),

    /// An error that occurred while making an HTTP request.
    Request(reqwest::Error),

    /// An unsuccessful HTTP status code in an HTTP response.
    Http(reqwest::StatusCode),

    /// A missing Content-Type header in a response.
    MissingContentType,

    /// An invalid Content-Type header.
    InvalidContentType(header::ToStrError),

    /// A Content-Type that is not understood by the service.
    UnexpectedContentType(String),
}

impl fmt::Display for HTTPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HTTPError::Body(err) => write!(f, "Error retrieving body of HTTP response: {err}"),
            HTTPError::Request(err) => write!(f, "Error while making HTTP request: {err}"),
            HTTPError::Http(status) => write!(f, "Request returned HTTP {status}"),
            HTTPError::MissingContentType => write!(f, "Missing Content-Type header"),
            HTTPError::InvalidContentType(err) => {
                write!(f, "Invalid Content-Type header value: {err}")
            }
            HTTPError::UnexpectedContentType(content_type) => {
                write!(f, "Unexpected content type: {content_type}")
            }
        }
    }
}

impl error::Error for HTTPError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            HTTPError::Body(err) => Some(err),
            HTTPError::Request(err) => Some(err),
            HTTPError::Http(_) => None,
            HTTPError::MissingContentType => None,
            HTTPError::InvalidContentType(err) => Some(err),
            HTTPError::UnexpectedContentType(_) => None,
        }
    }
}
