//! All things time-related.

use std::ops::Sub;
use std::time::Duration;
pub use chrono::{DateTime, Local, TimeDelta, Utc};
use relativetime::NegativeRelativeTime;

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
    /// The date the item was created.
    fn created_at(&self) -> DateTime<Utc>;

    /// The age of the account.
    ///
    /// `clock` is a source of time from which the age can be derived.
    /// Generally [`SystemClock::new()`] is used.
    fn age<C: Clock>(&self, clock: C) -> TimeDelta {
        let birthday = self.created_at();
        clock.now().sub(birthday)
    }

    /// The age of the account, relative to the current time, as a
    /// human-readable string.
    ///
    /// `clock` is a source of time from which the age can be derived.
    /// Generally [`SystemClock::new()`] is used.
    fn relative_age<C: Clock>(&self, clock: C) -> String {
        let age = self.age(clock).as_seconds_f64();
        let d = Duration::from_secs(age.trunc() as u64);
        d.to_relative_in_past()
    }
}

mod tests {
    use super::*;
    use std::ops::Sub;

    #[test]
    fn it_returns_the_system_time() {
        let clock = SystemClock::new();
        let delta = Utc::now().sub(clock.now());
        let secs = delta.num_seconds();
        assert_eq!(secs, 0);
    }
}
