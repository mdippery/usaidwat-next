//! Clients for reading data from the Reddit API.

use crate::service::Service;
use crate::thing::{Comment, DateTime, Submission, TimeDelta, User, Utc};
use std::fmt;
use std::ops::Sub;

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
    ///
    /// Returns `None` if data cannot be parsed for the given username.
    pub fn new(username: &'a str, service: Box<dyn Service>) -> Option<Self> {
        let user_data = service.get_resource(username, "about")?;
        let comment_data = service.get_resource(username, "comments")?;
        let post_data = service.get_resource(username, "submitted")?;
        let user = User::parse(&user_data, &comment_data, &post_data)?;
        Some(Self {
            username,
            user,
            service,
        })
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

    /// Redditor's link karma
    pub fn link_karma(&self) -> u64 {
        self.user.about().link_karma()
    }

    /// Redditor's comment karma
    pub fn comment_karma(&self) -> u64 {
        self.user.about().comment_karma()
    }

    /// Redditor's comments
    pub fn comments(&self) -> &Vec<Comment> {
        self.user.comments()
    }

    /// Redditor's posts
    pub fn submissions(&self) -> &Vec<Submission> {
        self.user.submissions()
    }
}

#[cfg(test)]
mod tests {
    use crate::service::{JsonResponse, RawResponse, Service, Uri};
    use std::fs;

    struct TestService {
        suffix: &'static str,
    }

    impl Service for TestService {
        fn get(&self, uri: Uri) -> Option<RawResponse> {
            Some(fs::read_to_string(uri).expect("could not find test data"))
        }

        fn get_resource(&self, username: &str, resource: &str) -> Option<JsonResponse> {
            let filename = format!("tests/data/{resource}_{}.json", self.suffix);
            self.get(filename)
        }

        fn user_agent(&self) -> String {
            format!("test-service-please-ignore v{}", env!("CARGO_PKG_VERSION"))
        }
    }

    impl TestService {
        fn new(suffix: &'static str) -> Self {
            Self { suffix }
        }
    }

    mod user_with_data {
        use super::super::Redditor;
        use super::TestService;
        use chrono::DateTime;

        fn test_service() -> TestService {
            TestService::new("mipadi")
        }

        fn test_client<'a>() -> Redditor<'a> {
            Redditor::new("mipadi", Box::new(test_service())).unwrap()
        }

        #[test]
        fn it_returns_its_creation_date() {
            let actual_date = test_client().created_at();
            let expected_date = DateTime::parse_from_rfc3339("2008-03-31T22:55:26Z").unwrap();
            assert_eq!(actual_date, expected_date);
        }

        #[test]
        fn it_returns_its_age() {
            // TODO: Mock time so I can calculate this using ==
            let actual_age = test_client().age().as_seconds_f64();
            let expected_age = 540667427.0;
            assert!(actual_age > expected_age, "{actual_age} > {expected_age}");
        }

        #[test]
        fn it_returns_its_link_karma() {
            let actual_karma = test_client().link_karma();
            let expected_karma = 4892;
            assert_eq!(actual_karma, expected_karma)
        }

        #[test]
        fn it_returns_its_comment_karma() {
            let actual_karma = test_client().comment_karma();
            let expected_karma = 33440;
            assert_eq!(actual_karma, expected_karma)
        }

        #[test]
        fn it_returns_its_comments() {
            // TODO: Make it so I can call test_client().comments();
            //       Must change some ownership around comments(), maybe
            //       by using Rc or Arc.
            let client = test_client();
            let comments = client.comments();
            assert_eq!(comments.len(), 100);
        }

        #[test]
        fn it_returns_its_posts() {
            let client = test_client();
            let posts = client.submissions();
            assert_eq!(posts.len(), 25);
        }
    }

    mod user_with_no_data {
        use super::super::Redditor;
        use super::TestService;
        use chrono::DateTime;

        fn test_service() -> TestService {
            TestService::new("empty")
        }

        fn test_client<'a>() -> Redditor<'a> {
            Redditor::new("testuserpleaseignore", Box::new(test_service())).unwrap()
        }

        #[test]
        fn it_returns_its_creation_date() {
            let actual_date = test_client().created_at();
            let expected_date = DateTime::parse_from_rfc3339("2025-05-19T16:11:20Z").unwrap();
            assert_eq!(actual_date, expected_date);
        }

        #[test]
        fn it_returns_its_age() {
            // TODO: Mock time so I can calculate this using ==
            let actual_age = test_client().age().as_seconds_f64();
            let expected_age = 90.0;
            assert!(actual_age > expected_age, "{actual_age} > {expected_age}");
        }

        #[test]
        fn it_returns_its_link_karma() {
            let actual_karma = test_client().link_karma();
            let expected_karma = 5000;
            assert_eq!(actual_karma, expected_karma)
        }

        #[test]
        fn it_returns_its_comment_karma() {
            let actual_karma = test_client().comment_karma();
            let expected_karma = 50_000;
            assert_eq!(actual_karma, expected_karma)
        }

        #[test]
        fn it_returns_its_comments() {
            let client = test_client();
            let comments = client.comments();
            assert_eq!(comments.len(), 0);
        }

        #[test]
        fn it_returns_its_posts() {
            let client = test_client();
            let posts = client.submissions();
            assert_eq!(posts.len(), 0);
        }
    }

    mod invalid_user {
        use super::super::Redditor;
        use super::TestService;

        fn test_service() -> TestService {
            TestService::new("404")
        }

        fn test_client<'a>() -> Option<Redditor<'a>> {
            Redditor::new("doesnotexist", Box::new(test_service()))
        }

        #[test]
        fn it_is_none() {
            let client = test_client();
            assert!(client.is_none());
        }
    }
}
