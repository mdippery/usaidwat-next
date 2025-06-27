//! Clients for reading data from the Reddit API.

use crate::clock::{DateTime, HasAge, Utc};
use crate::service::{self, Service};
use crate::thing::{self, Comment, Submission, User};
pub use chrono::Weekday;
use chrono::{Datelike, Timelike};

/// A client error.
#[derive(Debug)]
pub enum Error {
    /// An error from the underlying HTTP service.
    Service(service::Error),

    /// An error parsing data.
    Parse(thing::Error),
}

impl From<service::Error> for Error {
    fn from(error: service::Error) -> Self {
        Error::Service(error)
    }
}

impl From<thing::Error> for Error {
    fn from(error: thing::Error) -> Self {
        Error::Parse(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Service(err) => write!(f, "Service error: {err}"),
            Error::Parse(err) => write!(f, "Parse error: {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Service(err) => Some(err),
            Error::Parse(err) => Some(err),
        }
    }
}

/// Represents a Reddit user.
pub struct Redditor {
    username: String,
    user: User,
}

impl std::fmt::Debug for Redditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    /// Returns an [`Error`] if data cannot be parsed for the given username.
    pub fn new<T: Service>(username: String, service: T) -> Result<Self, Error> {
        let user_data = service.get_resource(&username, "about")?;
        let comment_data = service.get_resource(&username, "comments")?;
        let post_data = service.get_resource(&username, "submitted")?;
        let user = User::parse(&user_data, &comment_data, &post_data)?;
        Ok(Self { username, user })
    }

    /// The Redditor's username.
    pub fn username(&self) -> String {
        self.username.to_string()
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

    /// Redditor's posts (articles and self posts).
    pub fn submissions(&self) -> impl Iterator<Item = Submission> {
        self.user.submissions()
    }

    /// True if the user has posted at least one comment.
    pub fn has_comments(&self) -> bool {
        self.comments().count() > 0
    }

    /// True if the user has posted as least one article or self post.
    pub fn has_submissions(&self) -> bool {
        self.submissions().count() > 0
    }

    /// A timeline of the user's comments, grouped by days of the week
    /// and hours of the day.
    pub fn timeline(&self) -> Timeline {
        Timeline::for_user(self)
    }
}

impl HasAge for Redditor {
    /// The date the account was created.
    fn created_utc(&self) -> DateTime<Utc> {
        self.user.about().created_utc()
    }
}

/// A day of comments, bucketed by hour, which each hour containing the
/// number of comments for that hour.
pub type TimelineDay = [u32; 24];

type Hour = u32;
type TimeMatrix = [TimelineDay; 7];

/// A timeline of a Redditor's comments, bucketed by day of the week and hour.
///
/// Can be useful to draw a "heatmap" of a Redditor's comments, similar to the
/// GitHub activity chart.
#[derive(Debug)]
pub struct Timeline {
    buckets: TimeMatrix,
}

impl Timeline {
    /// Calculate a new timeline for the given Redditor.
    pub fn for_user(user: &Redditor) -> Self {
        let groups = Timeline::grouped_by_weekdays_and_hours(user);
        let buckets = Timeline::group_to_matrix(groups);
        Timeline { buckets }
    }

    /// Iterate through timeline, returning a 2-tuple of `(Weekday, TimelineDay)`
    /// for each day of the week.
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
        use crate::clock::HasAge;
        use crate::test_utils::FrozenClock;
        use chrono::DateTime;

        #[test]
        fn it_returns_its_username() {
            let actual_username = Redditor::test().username();
            assert_eq!(actual_username, "mipadi");
        }

        #[test]
        fn it_returns_its_creation_date() {
            let actual_date = Redditor::test().created_utc();
            let expected_date = DateTime::parse_from_rfc3339("2008-03-31T22:55:26Z").unwrap();
            assert_eq!(actual_date, expected_date);
        }

        #[test]
        fn it_returns_its_age() {
            let actual_age = Redditor::test()
                .age(&FrozenClock::default())
                .as_seconds_f64();
            let expected_age = 541016254.0;
            assert_eq!(actual_age, expected_age, "{actual_age} != {expected_age}");
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

        #[test]
        fn it_confirms_that_it_has_comments() {
            assert!(Redditor::test().has_comments())
        }

        #[test]
        fn it_confirms_that_it_has_submissions() {
            assert!(Redditor::test().has_submissions())
        }

        #[test]
        fn it_returns_a_timeline() {
            let _ = Redditor::test_empty().timeline();
            // Not really anything else to test: there are more comprehensive
            // tests for Timeline and TimelineIterator below.
        }
    }

    mod user_with_no_data {
        use crate::client::Redditor;
        use crate::clock::HasAge;
        use crate::test_utils::FrozenClock;
        use chrono::DateTime;

        #[test]
        fn it_returns_its_username() {
            let actual_username = Redditor::test_empty().username();
            assert_eq!(actual_username, "testuserpleaseignore");
        }

        #[test]
        fn it_returns_its_creation_date() {
            let actual_date = Redditor::test_empty().created_utc();
            let expected_date = DateTime::parse_from_rfc3339("2010-06-15T06:13:46Z").unwrap();
            assert_eq!(actual_date, expected_date);
        }

        #[test]
        fn it_returns_its_age() {
            let actual_age = Redditor::test_empty()
                .age(&FrozenClock::default())
                .as_seconds_f64();
            let expected_age = 471437954.0;
            assert_eq!(actual_age, expected_age, "{actual_age} != {expected_age}");
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

        #[test]
        fn it_confirms_that_it_has_comments() {
            assert!(!Redditor::test_empty().has_comments())
        }

        #[test]
        fn it_confirms_that_it_has_submissions() {
            assert!(!Redditor::test_empty().has_submissions())
        }

        #[test]
        fn it_returns_a_timeline() {
            let _ = Redditor::test_empty().timeline();
            // Not really anything else to test: there are more comprehensive
            // tests for Timeline and TimelineIterator below.
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

    mod timeline {
        use crate::client::Redditor;
        use chrono::Weekday;
        use std::iter::zip;

        #[test]
        fn it_processes_user_data() {
            let client = Redditor::test();
            let timeline = client.timeline();
            let buckets = timeline.buckets;
            #[rustfmt::skip]
            let expected_buckets = [
                [2, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 3, 0, 0, 1, 0, 3, 0, 0, 0, 1, 3],
                [1, 0, 0, 0, 0, 0, 0, 0, 1, 4, 1, 1, 1, 1, 3, 0, 1, 0, 0, 0, 3, 1, 5, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 4],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 4, 0, 0, 2, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 4, 0, 1, 4, 1, 0, 0, 0, 0, 0, 0, 0, 1],
                [3, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 2, 1, 0, 0, 0, 0, 0, 1, 0, 1, 2, 5],
            ];
            assert_eq!(buckets, expected_buckets);
        }

        #[test]
        fn it_processes_data_for_users_with_no_comments() {
            let client = Redditor::test_empty();
            let timeline = client.timeline();
            let buckets = timeline.buckets;
            #[rustfmt::skip]
            let expected_buckets = [
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ];
            assert_eq!(buckets, expected_buckets);
        }

        #[test]
        fn it_returns_an_iterator_of_its_data() {
            #[rustfmt::skip]
            let expected_items = [
                (Weekday::Mon, [2, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 3, 0, 0, 1, 0, 3, 0, 0, 0, 1, 3]),
                (Weekday::Tue, [1, 0, 0, 0, 0, 0, 0, 0, 1, 4, 1, 1, 1, 1, 3, 0, 1, 0, 0, 0, 3, 1, 5, 0]),
                (Weekday::Wed, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 4]),
                (Weekday::Thu, [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 4, 0, 0, 2, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0]),
                (Weekday::Fri, [0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1]),
                (Weekday::Sat, [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 4, 0, 1, 4, 1, 0, 0, 0, 0, 0, 0, 0, 1]),
                (Weekday::Sun, [3, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 2, 1, 0, 0, 0, 0, 0, 1, 0, 1, 2, 5]),
            ];
            let timeline = Redditor::test().timeline();
            let actual_items = timeline.days();

            let zipped = zip(actual_items, expected_items);

            for ((actual_wday, actual), (expected_wday, expected)) in zipped {
                assert_eq!(actual_wday, expected_wday);
                assert_eq!(actual, expected);
            }
        }

        #[test]
        fn it_returns_an_empty_iterator_for_users_with_no_comments() {
            #[rustfmt::skip]
            let expected_items = [
                (Weekday::Mon, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                (Weekday::Tue, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                (Weekday::Wed, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                (Weekday::Thu, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                (Weekday::Fri, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                (Weekday::Sat, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                (Weekday::Sun, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            ];

            let timeline = Redditor::test_empty().timeline();
            let actual_items = timeline.days();

            let zipped = zip(actual_items, expected_items);

            for ((actual_wday, actual), (expected_wday, expected)) in zipped {
                assert_eq!(actual_wday, expected_wday);
                assert_eq!(actual, expected);
            }
        }
    }
}
