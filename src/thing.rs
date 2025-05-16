//! A "thing" in the Reddit sense.
//!
//! Historically in the Reddit API and its old source code, a "Thing" was
//! any element of the Reddit system: users, posts, comments, etc. This
//! module encapsulates that idea and provides an easy way to more or less
//! work with JSON data from the Reddit API.

pub type DateTime = u64;   // TODO: Find an appropriate DateTime type

/// A Reddit user account.
pub struct User;

/// Reddit user account data.
pub struct About;

/// A Reddit Post.
pub struct Submission;

/// A Reddit comment.
pub struct Comment;

impl User {
    /// Returns account data for the user.
    pub fn about(&self) -> About {
        About {}
    }

    /// User's submissions.
    pub fn submissions(&self) -> Vec<Submission> {
        Vec::new()
    }

    /// User's comments.
    pub fn comments(&self) -> Vec<Comment> {
        Vec::new()
    }
}

impl About {
    /// The date on which the account was created.
    pub fn created_at(&self) -> DateTime {
        0
    }

    /// User's current karma for submissions.
    pub fn link_karma(&self) -> u64 {
        0
    }

    /// User's current karma for comments.
    pub fn comment_karma(&self) -> u64 {
        0
    }
}
