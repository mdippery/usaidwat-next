//! Clients for reading data from the Reddit API.

use crate::service::Service;
use crate::thing::{Comment, DateTime, Submission, TimeDelta, User, Utc};
pub use chrono::Weekday;
use chrono::{Datelike, Timelike};
use relativetime::NegativeRelativeTime;
use std::fmt;
use std::ops::Sub;
use std::time::Duration;

/// Represents a Reddit user.
pub struct Redditor {
    username: String,
    user: User,
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
    pub fn new<T: Service>(username: String, service: T) -> Option<Self> {
        let user_data = service.get_resource(&username, "about")?;
        let comment_data = service.get_resource(&username, "comments")?;
        let post_data = service.get_resource(&username, "submitted")?;
        let user = User::parse(&user_data, &comment_data, &post_data)?;
        Some(Self { username, user })
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

    /// A timeline of the user's comments, grouped by days of the week
    /// and hours of the day.
    // TODO: Test!
    pub fn timeline(&self) -> Timeline {
        Timeline::for_user(self)
    }
}

type Hour = u32;
pub type TimelineDay = [u32; 24];
type TimeMatrix = [TimelineDay; 7];

#[derive(Debug)]
pub struct Timeline {
    buckets: TimeMatrix,
}

impl Timeline {
    /// Calculate a new timeline for the given Redditor.
    pub fn for_user(user: &Redditor) -> Self {
        // TODO: Test that these calculations are correct!
        let groups = Timeline::grouped_by_weekdays_and_hours(user);
        let buckets = Timeline::group_to_matrix(groups);
        Timeline { buckets }
    }

    pub fn days(&self) -> impl Iterator<Item = (Weekday, TimelineDay)> {
        TimelineIterator::new(&self)
    }

    fn grouped_by_weekdays_and_hours(user: &Redditor) -> impl Iterator<Item = (Weekday, Hour)> {
        user.comments()
            .map(|c| (c.created_local().weekday(), c.created_local().hour()))
    }

    fn group_to_matrix(groups: impl Iterator<Item = (Weekday, Hour)>) -> TimeMatrix {
        let mut buckets = [[0; 24]; 7];
        for (weekday, hour) in groups {
            let wday = weekday.num_days_from_monday();
            assert!(wday < 7);
            assert!(hour < 24);
            buckets[wday as usize][hour as usize] += 1;
        }
        buckets
    }
}

#[derive(Debug)]
struct TimelineIterator<'a> {
    timeline: &'a Timeline,
    row: u8,
}

impl<'a> TimelineIterator<'a> {
    pub fn new(timeline: &'a Timeline) -> Self {
        Self { timeline, row: 0 }
    }
}

impl<'a> Iterator for TimelineIterator<'a> {
    type Item = (Weekday, TimelineDay);

    fn next(&mut self) -> Option<Self::Item> {
        if self.row < 7 {
            let wday = Weekday::try_from(self.row).unwrap();
            let day = self.timeline.buckets[self.row as usize];
            self.row += 1;
            Some((wday, day))
        } else {
            None
        }
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
