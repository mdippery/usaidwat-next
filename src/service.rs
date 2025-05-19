//! HTTPS connector for the Reddit API.
//!
//! Service structures in this module provide a low-level way to interact
//! with the Reddit API over HTTPS, essentially a specialized HTTPS client
//! specifically for Reddit.

pub type Uri = String; // TODO: Find a real type (IntoUrl from reqwest?)
pub type RawResponse = String; // TODO: Find a real type
pub type JsonResponse = String; // TODO: Find a real type

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
    // TODO: Figure out how to actually test this because otherwise
    //       the type can remain completely wrong (probably should be
    //       an Optional).
    fn get(&self, uri: Uri) -> Option<RawResponse> {
        Some("".to_string())
    }

    fn get_resource(&self, username: &str, resource: &str) -> Option<JsonResponse> {
        let uri = self.uri(username, resource);
        self.get(uri)
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
