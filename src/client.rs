use crate::service::Service;

type DateTime = &'static str;   // TODO: Find an appropriate DateTime type

/// A Reddit Post.
pub struct Post;

/// A Reddit comment.
pub struct Comment;

/// Represents a Reddit user.
pub struct Redditor<'a> {
    /// Redditor's username
    username: &'a str,

    /// A concrete connector for retrieving data from the Reddit API.
    service: Box<dyn Service>,
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
    pub fn posts(&self) -> Vec<Post> {
        Vec::new()
    }

    /// Redditor's comments
    pub fn comments(&self) -> Vec<Comment> {
        Vec::new()
    }
}
