//! HTTPS connector for the Reddit API.
//!
//! Service structures in this module provide a low-level way to interact
//! with the Reddit API over HTTPS, essentially a specialized HTTPS client
//! specifically for Reddit.

use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::{Client, IntoUrl};
use std::fmt::Formatter;
use std::result;

/// A service error.
#[derive(Debug)]
pub enum Error {
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Body(err) => write!(f, "Error retrieving body of HTTP response: {err}"),
            Error::Request(err) => write!(f, "Error while making HTTP request: {err}"),
            Error::Http(status) => write!(f, "Request returned HTTP {status}"),
            Error::MissingContentType => write!(f, "Missing Content-Type header"),
            Error::InvalidContentType(err) => write!(f, "Invalid Content-Type header value: {err}"),
            Error::UnexpectedContentType(content_type) => {
                write!(f, "Unexpected content type: {content_type}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Body(err) => Some(err),
            Error::Request(err) => Some(err),
            Error::Http(_) => None,
            Error::MissingContentType => None,
            Error::InvalidContentType(err) => Some(err),
            Error::UnexpectedContentType(_) => None,
        }
    }
}

/// The result of an HTTP request.
pub type Result = result::Result<String, Error>;

/// A service for retrieving information for Reddit users.
///
/// Using this trait, clients can implement different ways of connecting
/// to the Reddit API, such as an actual connector for production code,
/// and a mocked connector for testing purposes.
pub trait Service {
    /// Performs a GET request to the given URI and returns the raw body.
    async fn get(&self, uri: impl IntoUrl) -> Result;

    /// Performs a GET request to the `resource` associated with the given
    /// `username` and returns it as a parsed JSON response.
    async fn get_resource(&self, username: &str, resource: &str) -> Result;

    /// An appropriate user agent to use for HTTP requests.
    fn user_agent(&self) -> String {
        format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}

/// A service that contacts the Reddit API directly to retrieve information.
pub struct RedditService;

impl RedditService {
    /// Creates a new Reddit service.
    pub fn new() -> Self {
        Self {}
    }

    fn headers(&self) -> HeaderMap {
        let ua = self.user_agent();
        let ua = HeaderValue::from_str(&ua).expect(&format!("could not get user agent from {ua}"));
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, ua);
        headers
    }

    fn query_string(&self, resource: &str) -> &str {
        match resource {
            "comments" => "?limit=100",
            "submitted" => "?limit=100",
            _ => "",
        }
    }

    fn uri(&self, username: &str, resource: &str) -> String {
        let qs = self.query_string(resource);
        format!("https://www.reddit.com/user/{username}/{resource}.json{qs}")
    }
}

impl Service for RedditService {
    async fn get(&self, uri: impl IntoUrl) -> Result {
        let client = Client::new();
        let resp = client
            .get(uri)
            .headers(self.headers())
            .send()
            .await
            .map_err(Error::Request)?;

        if !resp.status().is_success() {
            Err(Error::Http(resp.status()))
        } else {
            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .ok_or(Error::MissingContentType)?
                .to_str()
                .map_err(Error::InvalidContentType)?;
            if !content_type.starts_with("application/json") {
                Err(Error::UnexpectedContentType(content_type.to_string()))
            } else {
                resp.text().await.map_err(Error::Body)
            }
        }
    }

    async fn get_resource(&self, username: &str, resource: &str) -> Result {
        let uri = self.uri(username, resource);
        self.get(&uri).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn it_returns_user_agent_with_version_number() {
        let service = RedditService::new();
        let user_agent = service.user_agent();
        let version_re = Regex::new(r"^[a-z]+ v\d+\.\d+\.\d+(-(alpha|beta)\.\d+)?$").unwrap();
        assert!(
            version_re.is_match(&user_agent),
            "{} does not match {}",
            user_agent,
            version_re,
        );
    }

    #[test]
    fn it_returns_a_query_string_with_comment_limits() {
        let service = RedditService::new();
        let qs = service.query_string("comments");
        assert_eq!(qs, "?limit=100");
    }

    #[test]
    fn it_returns_a_query_string_with_post_limits() {
        let service = RedditService::new();
        let qs = service.query_string("submitted");
        assert_eq!(qs, "?limit=100");
    }

    #[test]
    fn it_returns_an_empty_query_string_for_profiles() {
        let service = RedditService::new();
        let qs = service.query_string("about");
        assert_eq!(qs, "");
    }

    #[test]
    fn it_returns_a_uri_for_comments() {
        let service = RedditService::new();
        let actual_uri = service.uri("mipadi", "comments");
        let expected_uri = "https://www.reddit.com/user/mipadi/comments.json?limit=100";
        assert_eq!(actual_uri, expected_uri);
    }

    #[test]
    fn it_returns_a_uri_for_posts() {
        let service = RedditService::new();
        let actual_uri = service.uri("mipadi", "submitted");
        let expected_uri = "https://www.reddit.com/user/mipadi/submitted.json?limit=100";
        assert_eq!(actual_uri, expected_uri);
    }

    #[test]
    fn it_returns_a_uri_for_profiles() {
        let service = RedditService::new();
        let actual_uri = service.uri("mipadi", "about");
        let expected_uri = "https://www.reddit.com/user/mipadi/about.json";
        assert_eq!(actual_uri, expected_uri);
    }
}
