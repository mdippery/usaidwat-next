//! HTTPS connector for the Reddit API.
//!
//! Service structures in this module provide a low-level way to interact
//! with the Reddit API over HTTPS, essentially a specialized HTTPS client
//! specifically for Reddit.

// TODO: Async, maybe
use reqwest::blocking::Client;
use reqwest::header::{self, HeaderMap, HeaderValue};

/// A URI.
pub type Uri<'a> = &'a str;

/// An HTTP response, which would represent any type of data: raw text, JSON, or other.
pub type RawResponse = String;

/// An HTTP response containing JSON data.
pub type JsonResponse = String;

/// A service for retrieving information for Reddit users.
///
/// Using this trait, clients can implement different ways of connecting
/// to the Reddit API, such as an actual connector for production code,
/// and a mocked connector for testing purposes.
pub trait Service {
    /// Performs a GET request to the given URI and returns the raw body.
    fn get(&self, uri: Uri) -> Option<RawResponse>;

    /// Performs a GET request to the `resource` associated with the given
    /// `username` and returns it as a parsed JSON response.
    fn get_resource(&self, username: &str, resource: &str) -> Option<JsonResponse>;

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
        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_str(&self.user_agent()).unwrap(),
        );
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
    fn get(&self, uri: Uri) -> Option<RawResponse> {
        let client = Client::new();
        // TODO: Maybe return Result instead of Option
        let resp = client.get(uri).headers(self.headers()).send().ok()?;

        // TODO: Ugh, this is ugly -- clean it up!
        if !resp.status().is_success() {
            None
        } else {
            let content_type = resp.headers().get(header::CONTENT_TYPE)?.to_str().ok()?;
            if (!content_type.starts_with("application/json")) {
                None
            } else {
                resp.text().ok()
            }
        }
    }

    fn get_resource(&self, username: &str, resource: &str) -> Option<JsonResponse> {
        let uri = self.uri(username, resource);
        self.get(&uri)
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
