use std::fmt;
use crate::service::Service;
use crate::thing::{Comment, Submission, User};

type DateTime = &'static str;   // TODO: Find an appropriate DateTime type

/// Represents a Reddit user.
pub struct Redditor<'a> {
    /// Redditor's username
    username: &'a str,

    /// A concrete connector for retrieving data from the Reddit API.
    service: Box<dyn Service>,
}

impl<'a> fmt::Debug for Redditor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Redditor {{ username = {} }}", self.username)
    }
}

impl<'a> Redditor<'a> {
    /// Creates a new client for retrieving information for Reddit users.
    ///
    /// `username` should be the Redditor's username. `service` is the
    /// actual service implementation that will be used to retrieve
    /// information about the Redditor.
    pub fn new(username: &'a str, service: Box<dyn Service>) -> Self {
        Self { username, service }
    }

    fn user(&self) -> User {
        User {}
    }

    /// The date the account was created.
    pub fn created_at(&self) -> DateTime {
        ""
    }

    /// The age of the account, in seconds.
    pub fn age(&self) -> u64 {
        0
    }

    /// Redditor's link karma
    pub fn link_karma(&self) -> u64 {
        0
    }

    /// Redditor's comment karma
    pub fn comment_karma(&self) -> u64 {
        0
    }

    /// Redditor's posts
    pub fn posts(&self) -> Vec<Submission> {
        Vec::new()
    }

    /// Redditor's comments
    pub fn comments(&self) -> Vec<Comment> {
        Vec::new()
    }
}
