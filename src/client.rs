//! Clients for reading data from the Reddit API.

use crate::service::Service;
use crate::thing::{Comment, DateTime, Submission, User};
use std::fmt;

/// Represents a Reddit user.
pub struct Redditor<'a> {
    username: &'a str,
    user: User,
    service: Box<dyn Service>,
}

impl<'a> fmt::Debug for Redditor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Redditor {{ username = {}, user = {:?} }}",
            self.username, self.user
        )
    }
}

impl<'a> Redditor<'a> {
    /// Creates a new client for retrieving information for Reddit users.
    ///
    /// `username` should be the Redditor's username. `service` is the
    /// actual service implementation that will be used to retrieve
    /// information about the Redditor.
    pub fn new(username: &'a str, service: Box<dyn Service>) -> Self {
        let user = User::parse("", "", "").unwrap(); // TODO: Make HTTP calls and parse
        Self {
            username,
            user,
            service,
        }
    }

    /// The date the account was created.
    pub fn created_at(&self) -> DateTime {
        self.user.about().created_at()
    }

    /// The age of the account, in seconds.
    pub fn age(&self) -> u64 {
        let current_time = 0;
        current_time - self.created_at()
    }

    /// Redditor's link karma
    pub fn link_karma(&self) -> u64 {
        self.user.about().link_karma()
    }

    /// Redditor's comment karma
    pub fn comment_karma(&self) -> u64 {
        self.user.about().comment_karma()
    }

    /// Redditor's posts
    pub fn submissions(&self) -> &Vec<Submission> {
        self.user.submissions()
    }

    /// Redditor's comments
    pub fn comments(&self) -> &Vec<Comment> {
        self.user.comments()
    }
}
