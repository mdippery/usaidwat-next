//! A "thing" in the Reddit sense.
//!
//! Historically in the Reddit API and its old source code, a "Thing" was
//! any element of the Reddit system: users, posts, comments, etc. This
//! module encapsulates that idea and provides an easy way to more or less
//! work with JSON data from the Reddit API.

/// A Reddit user account.
pub struct User;

/// A Reddit Post.
pub struct Submission;

/// A Reddit comment.
pub struct Comment;
