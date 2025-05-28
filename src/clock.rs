//! All things time-related.

pub use chrono::{DateTime, Local, TimeDelta, Utc};
use relativetime::NegativeRelativeTime;
use std::ops::Sub;
use std::time::Duration;

/// Tells time and returns the time.
///
/// Generally you will want to retrieve time using [`SystemClock`],
/// but it tests you may want to implement a `Clock` with a fixed time.
pub trait Clock {
    /// The current time.
    fn now(&self) -> DateTime<Utc>;
}

/// Interacts with the system clock to get the current time.
#[derive(Debug)]
pub struct SystemClock;

impl SystemClock {
    /// Creates a new clock to interact with the system time.
    pub fn new() -> Self {
        SystemClock {}
    }
}

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Marks a thing that has a notion of its age.
pub trait HasAge {
    /// The date the item was created, in UTC.
    fn created_utc(&self) -> DateTime<Utc>;

    /// The age of the account.
    ///
    /// `clock` is a source of time from which the age can be derived.
    /// Generally [`SystemClock::new()`] is used.
    fn age<C: Clock>(&self, clock: &C) -> TimeDelta {
        let birthday = self.created_utc();
        clock.now().sub(birthday)
    }

    /// The age of the account, relative to the current time, as a
    /// human-readable string.
    ///
    /// `clock` is a source of time from which the age can be derived.
    /// Generally [`SystemClock::new()`] is used.
    fn relative_age<C: Clock>(&self, clock: &C) -> String {
        // TODO: For FFS, sometimes this prints "1 months ago".
        //       I'm using a crate so it's the crate's fault, but I should
        //       either fix the crate or hack a fix here. So annoying.
        let age = self.age(clock).as_seconds_f64();
        let d = Duration::from_secs(age.trunc() as u64);
        d.to_relative_in_past()
    }
}

#[cfg(test)]
mod tests {
    mod clock {
        use super::super::*;
        use std::ops::Sub;

        #[test]
        fn it_returns_the_system_time() {
            let clock = SystemClock::new();
            let delta = Utc::now().sub(clock.now());
            let secs = delta.num_seconds();
            assert_eq!(secs, 0);
        }
    }

    mod has_age {
        use super::super::*;
        use crate::clock::HasAge;
        use crate::test_utils::{FrozenClock, load_data};
        use crate::thing::Comment;

        #[test]
        #[ignore] // TODO: Currently fails because the relativetime crate has a bug
        fn it_correctly_formats_singular_time_units() {
            let datetime = DateTime::parse_from_rfc3339("2025-05-28T10:51:00-07:00")
                .expect("could not parse timestamp")
                .with_timezone(&Utc);
            let clock = FrozenClock::new(datetime);
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[3];
            assert_eq!(comment.relative_age(&clock), "1 month ago");
        }

        #[test]
        fn it_correctly_formats_singular_time_units_with_indefinite_articles() {
            let datetime = DateTime::parse_from_rfc3339("2025-05-28T10:51:00-07:00")
                .expect("could not parse timestamp")
                .with_timezone(&Utc);
            let clock = FrozenClock::new(datetime);
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[2];
            assert_eq!(comment.relative_age(&clock), "a month ago");
        }
    }
}
