//! A "thing" in the Reddit sense.
//!
//! Historically in the Reddit API and its old source code, a "Thing" was
//! any element of the Reddit system: users, posts, comments, etc. This
//! module encapsulates that idea and provides an easy way to more or less
//! work with JSON data from the Reddit API.

use serde::Deserialize;
use serde_json::{self, Result};

pub type DateTime = u64; // TODO: Find an appropriate DateTime type

/// A Reddit user account.
#[derive(Debug)]
pub struct User {
    about: About,
    comments: Vec<Comment>,
    submissions: Vec<Submission>,
}

/// Reddit user account data.
#[derive(Debug, Deserialize)]
pub struct About {
    name: String,
    id: String,
    created_utc: DateTime,
    link_karma: u64,
    comment_karma: u64,
}

/// A Reddit comment.
#[derive(Debug, Deserialize)]
pub struct Comment {
    id: String,
    name: String,
    subreddit_id: String,
    subreddit: String,
    link_title: String,
    link_id: String,
    created_utc: f64, // TODO: Convert this to datetime
    body: String,
    ups: u64,
    downs: u64,
}

/// A Reddit Post.
#[derive(Debug, Deserialize)]
pub struct Submission {
    id: String,
    name: String,
    permalink: String,
    author: String,
    domain: String,
    subreddit_id: String,
    subreddit: String,
    url: String, // TODO: Convert this URL struct
    title: String,
    selftext: String,
    created_utc: f64, // TODO: Convert this to datetime
    num_comments: u64,
    ups: u64,
    downs: u64,
    score: u64,
}

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
        Some(User {
            about,
            comments,
            submissions,
        })
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
        serde_json::from_str(user_data)
            .ok()
            .map(|wrapper: AboutResponse| wrapper.data)
    }

    /// The date on which the account was created.
    pub fn created_at(&self) -> DateTime {
        self.created_utc
    }

    /// User's current karma for submissions.
    pub fn link_karma(&self) -> u64 {
        self.link_karma
    }

    /// User's current karma for comments.
    pub fn comment_karma(&self) -> u64 {
        self.comment_karma
    }
}

impl Comment {
    /// Parses a text response from the Reddit API into a list of comments.
    ///
    /// Specifically, `comment_data` is the result of a call to
    /// `/users/<user>/comments.json`.
    ///
    /// This method is generally invoked by `User`, not directly.
    fn parse(comment_data: &str) -> Option<Vec<Self>> {
        serde_json::from_str(comment_data)
            .ok()
            .map(|comment_listing: ListingResponse<CommentResponse>| {
                comment_listing
                    .data
                    .children
                    .into_iter()
                    .map(|comment_wrapper| comment_wrapper.data)
                    .collect()
            })
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
        serde_json::from_str(post_data)
            .ok()
            .map(|comment_listing: ListingResponse<SubmissionResponse>| {
                comment_listing
                    .data
                    .children
                    .into_iter()
                    .map(|comment_wrapper| comment_wrapper.data)
                    .collect()
            })
    }
}

// Response wrappers
// --------------------------------------------------------------------------
// These are necessary because the Reddit API returns data wrapped in "data"
// and "children" keys, so serde_json has to first parse these parent keys
// that we don't really care about to get to the "real" data.

#[derive(Debug, Deserialize)]
struct AboutResponse {
    data: About,
}

#[derive(Debug, Deserialize)]
struct ListingResponse<T> {
    data: ChildrenResponse<T>,
}

#[derive(Debug, Deserialize)]
struct ChildrenResponse<T> {
    children: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct CommentResponse {
    data: Comment,
}

#[derive(Debug, Deserialize)]
struct SubmissionResponse {
    data: Submission,
}

// Unit tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::fs;

    fn load_data(file: &str) -> String {
        fs::read_to_string(format!("tests/data/{file}.json")).expect("could not find test data")
    }

    mod about {
        use super::super::*;
        use super::load_data;

        #[test]
        fn it_cannot_parse_invalid_data() {
            let about = About::parse(&load_data("about_404"));
            assert!(about.is_none(), "should be None, was {about:?}");
        }

        #[test]
        fn it_can_parse_valid_data() {
            let about = About::parse(&load_data("about_mipadi"));
            assert!(about.is_some());
        }

        #[test]
        fn it_parses_fields() {
            let about = About::parse(&load_data("about_mipadi")).unwrap();
            assert_eq!(about.created_at(), 1207004126);
            assert_eq!(about.link_karma(), 4892);
            assert_eq!(about.comment_karma(), 33440);
        }
    }

    mod comments {
        use super::super::*;
        use super::load_data;

        #[test]
        fn it_cannot_parse_invalid_data() {
            let comments = Comment::parse(&load_data("comments_404"));
            assert!(comments.is_none(), "should be None, was {comments:?}");
        }

        #[test]
        fn it_can_parse_valid_data() {
            let comments = Comment::parse(&load_data("comments_mipadi"));
            assert!(comments.is_some());
        }

        #[test]
        fn it_can_parse_empty_data() {
            let comments = Comment::parse(&load_data("comments_empty"));
            assert!(comments.is_some());
        }

        #[test]
        fn it_parses_fields() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            assert_eq!(comments.len(), 100);

            let expected_link_title =
                "Heisenbug: a software bug that seems to disappear or alter \
                its behavior when one attempts to study it";
            let expected_body =
                "Yep. My first experience with a Heisenbug occurred in a C++ \
                program, and disappeared when I tried to print a variable \
                with printf (only to reappear when that call was removed).";

            let comment = &comments[0];
            assert_eq!(comment.name, "t1_c79peed");
            assert_eq!(comment.subreddit_id, "t5_2qh3b");
            assert_eq!(comment.subreddit, "wikipedia");
            assert_eq!(comment.link_title, expected_link_title);
            assert_eq!(comment.link_id, "t3_142t4w");
            assert_eq!(comment.created_utc, 1354392868.0);
            assert_eq!(comment.body, expected_body);
            assert_eq!(comment.ups, 1);
            assert_eq!(comment.downs, 0);
        }

        #[test]
        fn it_returns_an_empty_collection() {
            let comments = Comment::parse(&load_data("comments_empty")).unwrap();
            assert!(comments.is_empty());
        }
    }

    mod submissions {
        use super::super::*;
        use super::load_data;

        #[test]
        fn it_cannot_parse_invalid_data() {
            let submissions = Submission::parse(&load_data("submitted_404"));
            assert!(submissions.is_none(), "should be None, was {submissions:?}");
        }

        #[test]
        fn it_can_parse_valid_data() {
            let submissions = Submission::parse(&load_data("submitted_mipadi"));
            assert!(submissions.is_some());
        }

        #[test]
        fn it_can_parse_empty_data() {
            let submissions = Submission::parse(&load_data("submitted_empty"));
            assert!(submissions.is_some());
        }

        #[test]
        fn it_parses_fields() {
            let submissions = Submission::parse(&load_data("submitted_mipadi")).unwrap();
            assert_eq!(submissions.len(), 25);

            let submission = &submissions[0];
            assert_eq!(submission.id, "3pj7rx");
            assert_eq!(submission.name, "t3_3pj7rx");
            assert_eq!(submission.permalink, "/r/short/comments/3pj7rx/science_says_being_short_makes_you_depressed/");
            assert_eq!(submission.author, "mipadi");
            assert_eq!(submission.domain, "vice.com");
            assert_eq!(submission.subreddit_id, "t5_2sgvi");
            assert_eq!(submission.subreddit, "short");
            assert_eq!(submission.url, "http://www.vice.com/read/it-sucks-to-be-a-short-guy-511");
            assert_eq!(submission.title, "Science Says Being Short Makes You Depressed");
            assert_eq!(submission.selftext, "");
            assert_eq!(submission.created_utc, 1445369797.0);
            assert_eq!(submission.num_comments, 65);
            assert_eq!(submission.ups, 12);
            assert_eq!(submission.downs, 0);
            assert_eq!(submission.score, 12);
        }

        #[test]
        fn it_parses_fields_of_self_posts() {
            let submissions = Submission::parse(&load_data("submitted_mipadi")).unwrap();
            assert_eq!(submissions.len(), 25);

            let expected_selftex =
                "It's called [Karmanaut](https://github.com/mdippery/karmanaut), \
                and it's written in Clojure and uses MongoDB as a data store. \
                It's pretty simple to set up. It's best run as a cronjob ever \
                *x* hours. Requires only Java and MongoDB (plus Leiningen to \
                build it).\n\nEventually I plan to wire it up to a web frontend. \
                The idea is to make it easy to chart your karma growth (or decline!) \
                over time, and derive interesting statistics and other data from it \
                (such as average karma gained per day, rate of change, etc.).";

            let submission = &submissions[22];
            assert_eq!(submission.id, "26e9x6");
            assert_eq!(submission.name, "t3_26e9x6");
            assert_eq!(submission.permalink, "/r/webdev/comments/26e9x6/i_created_a_tool_for_sampling_reddit_users_karma/");
            assert_eq!(submission.author, "mipadi");
            assert_eq!(submission.domain, "self.webdev");
            assert_eq!(submission.subreddit_id, "t5_2qs0q");
            assert_eq!(submission.subreddit, "webdev");
            assert_eq!(submission.url, "https://www.reddit.com/r/webdev/comments/26e9x6/i_created_a_tool_for_sampling_reddit_users_karma/");
            assert_eq!(submission.title, "I created a tool for sampling Reddit users' karma (link and comment)");
            assert_eq!(submission.selftext, expected_selftex);
            assert_eq!(submission.created_utc, 1400960795.0);
            assert_eq!(submission.num_comments, 9);
            assert_eq!(submission.ups, 6);
            assert_eq!(submission.downs, 0);
            assert_eq!(submission.score, 6);
        }

        #[test]
        fn it_returns_an_empty_collection() {
            let submissions = Submission::parse(&load_data("submitted_empty")).unwrap();
            assert!(submissions.is_empty());
        }
    }
}
