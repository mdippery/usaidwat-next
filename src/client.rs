//! Clients for reading data from the Reddit API.

use crate::service::Service;
use crate::thing::{Comment, DateTime, Submission, TimeDelta, User, Utc};
use relativetime::NegativeRelativeTime;
use std::fmt;
use std::ops::Sub;
use std::time::Duration;

/// Represents a Reddit user.
pub struct Redditor {
    username: String,
    user: User,
    service: Box<dyn Service>,
}

impl fmt::Debug for Redditor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Redditor {{ username = {}, user = {:?} }}",
            self.username, self.user
        )
    }
}

impl Redditor {
    /// Creates a new client for retrieving information for Reddit users.
    ///
    /// `username` should be the Redditor's username. `service` is the
    /// actual service implementation that will be used to retrieve
    /// information about the Redditor.
    ///
    /// Returns `None` if data cannot be parsed for the given username.
    pub fn new(username: String, service: Box<dyn Service>) -> Option<Self> {
        let user_data = service.get_resource(&username, "about")?;
        let comment_data = service.get_resource(&username, "comments")?;
        let post_data = service.get_resource(&username, "submitted")?;
        let user = User::parse(&user_data, &comment_data, &post_data)?;
        Some(Self {
            username,
            user,
            service,
        })
    }

    /// The Redditor's username.
    pub fn username(&self) -> String {
        self.username.to_string()
    }

    /// The date the account was created.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.user.about().created_at()
    }

    /// The age of the account.
    pub fn age(&self) -> TimeDelta {
        let birthday = self.created_at();
        Utc::now().sub(birthday)
    }

    /// The age of the account, relative to the current time.
    pub fn relative_age(&self) -> String {
        let age = self.age().as_seconds_f64();
        let d = Duration::from_secs(age.trunc() as u64);
        d.to_relative_in_past()
    }

    /// Redditor's link karma
    pub fn link_karma(&self) -> i64 {
        self.user.about().link_karma()
    }

    /// Redditor's comment karma
    pub fn comment_karma(&self) -> i64 {
        self.user.about().comment_karma()
    }

    /// Redditor's comments
    pub fn comments(&self) -> impl Iterator<Item = Comment> {
        self.user.comments()
    }

    /// Redditor's posts
    pub fn submissions(&self) -> impl Iterator<Item = Submission> {
        self.user.submissions()
    }
}

#[cfg(test)]
mod tests {
    mod user_with_data {
        use crate::client::Redditor;
        use chrono::DateTime;

        #[test]
        fn it_returns_its_username() {
            let actual_username = Redditor::test().username();
            assert_eq!(actual_username, "mipadi");
        }

        #[test]
        fn it_returns_its_creation_date() {
            let actual_date = Redditor::test().created_at();
            let expected_date = DateTime::parse_from_rfc3339("2008-03-31T22:55:26Z").unwrap();
            assert_eq!(actual_date, expected_date);
        }

        #[test]
        fn it_returns_its_age() {
            // TODO: Mock time so I can calculate this using ==
            let actual_age = Redditor::test().age().as_seconds_f64();
            let expected_age = 540667427.0;
            assert!(actual_age > expected_age, "{actual_age} > {expected_age}");
        }

        #[test]
        fn it_returns_its_link_karma() {
            let actual_karma = Redditor::test().link_karma();
            let expected_karma = 11729;
            assert_eq!(actual_karma, expected_karma)
        }

        #[test]
        fn it_returns_its_comment_karma() {
            let actual_karma = Redditor::test().comment_karma();
            let expected_karma = 121995;
            assert_eq!(actual_karma, expected_karma)
        }

        #[test]
        fn it_returns_its_comments() {
            let count = Redditor::test().comments().count();
            assert_eq!(count, 100);
        }

        #[test]
        fn it_returns_its_posts() {
            let count = Redditor::test().submissions().count();
            assert_eq!(count, 100);
        }
    }

    mod user_with_no_data {
        use crate::client::Redditor;
        use chrono::DateTime;

        #[test]
        fn it_returns_its_username() {
            let actual_username = Redditor::test_empty().username();
            assert_eq!(actual_username, "testuserpleaseignore");
        }

        #[test]
        fn it_returns_its_creation_date() {
            let actual_date = Redditor::test_empty().created_at();
            let expected_date = DateTime::parse_from_rfc3339("2010-06-15T06:13:46Z").unwrap();
            assert_eq!(actual_date, expected_date);
        }

        #[test]
        fn it_returns_its_age() {
            // TODO: Mock time so I can calculate this using ==
            let actual_age = Redditor::test_empty().age().as_seconds_f64();
            let expected_age = 90.0;
            assert!(actual_age > expected_age, "{actual_age} > {expected_age}");
        }

        #[test]
        fn it_returns_its_link_karma() {
            let actual_karma = Redditor::test_empty().link_karma();
            let expected_karma = 0;
            assert_eq!(actual_karma, expected_karma)
        }

        #[test]
        fn it_returns_its_comment_karma() {
            let actual_karma = Redditor::test_empty().comment_karma();
            let expected_karma = 0;
            assert_eq!(actual_karma, expected_karma)
        }

        #[test]
        fn it_returns_its_comments() {
            let count = Redditor::test_empty().comments().count();
            assert_eq!(count, 0);
        }

        #[test]
        fn it_returns_its_posts() {
            let count = Redditor::test_empty().submissions().count();
            assert_eq!(count, 0);
        }
    }

    mod invalid_user {
        use crate::client::Redditor;

        #[test]
        fn it_is_none() {
            let client = Redditor::test_none();
            assert!(client.is_none());
        }
    }
}
