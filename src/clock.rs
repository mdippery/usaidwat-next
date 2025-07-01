//! All things time-related.

pub use chrono::{DateTime, Local, TimeDelta, Utc};
use regex::Regex;
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
#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Marks a thing that has a notion of its age.
pub trait HasAge {
    /// The date the item was created, in UTC.
    fn created_utc(&self) -> DateTime<Utc>;

    /// The date the item was created, in local time.
    fn created_local(&self) -> DateTime<Local> {
        self.created_utc().with_timezone(&Local)
    }

    /// The age of the account.
    ///
    /// `clock` is a source of time from which the age can be derived.
    /// Generally [`SystemClock::default()`] is used.
    fn age<C: Clock>(&self, clock: &C) -> TimeDelta {
        let birthday = self.created_utc();
        clock.now().sub(birthday)
    }

    /// The age of the account, relative to the current time, as a
    /// human-readable string.
    ///
    /// `clock` is a source of time from which the age can be derived.
    /// Generally [`SystemClock::default()`] is used.
    fn relative_age<C: Clock>(&self, clock: &C) -> String {
        let age = self.age(clock).as_seconds_f64();
        let d = Duration::from_secs(age.trunc() as u64);
        let s = d.to_relative_in_past();

        // The relativetime crate sometimes prints things like "1 months ago".
        // Unfortunately, the crate is no longer updated and isn't even on
        // GitHub anymore, so it's not likely to be updated any time soon,
        // so let's just hack around the bug here until we have a better fix.
        let re = Regex::new("^1 (?<unit>[a-z]+)s ago$").unwrap();
        re.replace(&s, "1 $unit ago").to_string()
    }
}

#[cfg(test)]
mod tests {
    mod clock {
        use super::super::*;
        use std::ops::Sub;

        #[test]
        fn it_returns_the_system_time() {
            let clock = SystemClock::default();
            let delta = Utc::now().sub(clock.now());
            let secs = delta.num_seconds();
            assert_eq!(secs, 0);
        }
    }

    mod has_age {
        use super::super::*;
        use crate::clock::HasAge;
        use crate::test_utils::FrozenClock;

        #[derive(Debug)]
        struct ThingWithAge {
            created_utc: DateTime<Utc>,
        }

        impl ThingWithAge {
            pub fn new(timestamp: i64) -> Self {
                let created_utc = DateTime::from_timestamp(timestamp, 0).unwrap();
                Self { created_utc }
            }
        }

        impl HasAge for ThingWithAge {
            fn created_utc(&self) -> DateTime<Utc> {
                self.created_utc
            }
        }

        #[test]
        #[ignore]
        fn it_returns_its_age() {
            todo!("test this!");
        }

        #[test]
        #[ignore]
        fn it_returns_its_age_as_a_relative_string() {
            todo!("test this!");
        }

        #[test]
        fn it_correctly_formats_singular_time_units() {
            let datetime = DateTime::parse_from_rfc3339("2025-05-28T10:51:00-07:00")
                .expect("could not parse timestamp")
                .with_timezone(&Utc);
            let clock = FrozenClock::new(datetime);
            let thing = ThingWithAge::new(1744177355);
            assert_eq!(thing.relative_age(&clock), "1 month ago");
        }

        #[test]
        fn it_correctly_formats_singular_time_units_with_indefinite_articles() {
            let datetime = DateTime::parse_from_rfc3339("2025-05-28T10:51:00-07:00")
                .expect("could not parse timestamp")
                .with_timezone(&Utc);
            let clock = FrozenClock::new(datetime);
            let thing = ThingWithAge::new(1744481059);
            assert_eq!(thing.relative_age(&clock), "a month ago");
        }
    }
}
