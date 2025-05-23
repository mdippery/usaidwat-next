//! All things time-related.

pub use chrono::{DateTime, Utc};

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
