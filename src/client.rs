use crate::service::Service;

pub struct RedditClient<'a> {
    username: &'a str,
    service: Box<dyn Service>,
}

impl<'a> RedditClient<'a> {
    /// Creates a new client for retrieving information for Reddit users.
    ///
    /// `username` should be the Redditor's username. `service` is the
    /// actual service implementation that will be used to retrieve
    /// information about the Redditor.
    pub fn new(username: &'a str, service: Box<dyn Service>) -> Self {
        Self { username, service }
    }
}
