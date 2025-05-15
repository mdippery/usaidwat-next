/// A service for retrieving information for Reddit users.
pub trait Service {}

/// A service that contacts the Reddit API directly to retrieve information.
pub struct RedditService;

impl Service for RedditService {}

impl RedditService {
    /// Creates a new Reddit service.
    pub fn new() -> Self {
        Self {}
    }
}
