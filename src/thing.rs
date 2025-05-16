//! A "thing" in the Reddit sense.
//!
//! Historically in the Reddit API and its old source code, a "Thing" was
//! any element of the Reddit system: users, posts, comments, etc. This
//! module encapsulates that idea and provides an easy way to more or less
//! work with JSON data from the Reddit API.

pub type DateTime = u64; // TODO: Find an appropriate DateTime type

/// A Reddit user account.
pub struct User {
    about: About,
    comments: Vec<Comment>,
    submissions: Vec<Submission>,
}

/// Reddit user account data.
pub struct About;

/// A Reddit comment.
pub struct Comment;

/// A Reddit Post.
pub struct Submission;

impl User {
    /// Parses text responses from the Reddit API into the associated
    /// data structures.
    ///
    /// `user_data` is the result of a call to `/users/<user>/about.json`
    /// and contains account medata, `comment_data` is the result of a call
    /// to `/users/<user>/comments.json`, and `post_data` is the result of
    /// a call to `/users/<user>/submitted.json`.
    ///
    /// Obviously parsing can fail so this method returns an `Option`.
    pub fn parse(user_data: &str, comment_data: &str, post_data: &str) -> Option<Self> {
        let about = About::parse(user_data)?;
        let comments = Comment::parse(comment_data)?;
        let submissions = Submission::parse(post_data)?;
        Some(User { about, comments, submissions })
    }

    /// Returns account data for the user.
    pub fn about(&self) -> &About {
        &self.about
    }

    /// User's comments.
    pub fn comments(&self) -> &Vec<Comment> {
        &self.comments
    }

    /// User's submissions.
    pub fn submissions(&self) -> &Vec<Submission> {
        &self.submissions
    }
}

impl About {
    /// Parses a text response from the Reddit API into user data.
    ///
    /// Specifically, `user_data` is the result of a call to
    /// `/users/<user>/about.json`.
    ///
    /// This method is generally invoked by `User`, not directly.
    fn parse(user_data: &str) -> Option<Self> {
        Some(About {})
    }

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

impl Comment {
    /// Parses a text response from the Reddit API into a list of comments.
    ///
    /// Specifically, `comment_data` is the result of a call to
    /// `/users/<user>/comments.json`.
    /// This method is generally invoked by `User`, not directly.
    fn parse(comment_data: &str) -> Option<Vec<Self>> {
        Some(vec![])
    }
}

impl Submission {
    /// Parses a text response from the Reddit API into a list of
    /// submissions (posts).
    ///
    /// Specifically, `post_data` is the result of a call to
    /// `/users/<user>/submitted.json`.
    ///
    /// This method is generally invoked by `User`, not directly.
    fn parse(post_data: &str) -> Option<Vec<Self>> {
        Some(vec![])
    }
}
