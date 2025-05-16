//! HTTPS connector for the Reddit API.
//!
//! Service structures in this module provide a low-level way to interact
//! with the Reddit API over HTTPS, essentially a specialized HTTPS client
//! specifically for Reddit.

type Uri = String; // TODO: Find a real type
type RawResponse = &'static str; // TODO: Find a real type
type JsonResponse = &'static str; // TODO: Find a real type

/// A service for retrieving information for Reddit users.
///
/// Using this trait, clients can implement different ways of connecting
/// to the Reddit API, such as an actual connector for production code,
/// and a mocked connector for testing purposes.
pub trait Service {
    /// Performs a GET request to the given URI and returns the raw body.
    fn get(&self, uri: Uri) -> RawResponse;

    /// Performs a GET request to the `resource` associated with the given
    /// `username` and returns it as a parsed JSON response.
    fn get_resource(&self, username: &str, resource: &str) -> JsonResponse;
}

/// A service that contacts the Reddit API directly to retrieve information.
pub struct RedditService;

impl RedditService {
    /// Creates a new Reddit service.
    pub fn new() -> Self {
        Self {}
    }
}

impl Service for RedditService {
    fn get(&self, uri: Uri) -> RawResponse {
        ""
    }

    fn get_resource(&self, username: &str, resource: &str) -> JsonResponse {
        let qs = match resource {
            "comments" => "?limit=100",
            "submitted" => "?limit=100",
            _ => "",
        };
        let uri = format!("https://www.reddit.com/user/{username}/{resource}.json{qs}");
        self.get(uri)
    }
}
