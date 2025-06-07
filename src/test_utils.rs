use crate::client::Redditor;
use crate::clock::{Clock, DateTime, Utc};
use crate::service::{JsonResponse, RawResponse, Service, Uri};
use std::fs;

pub fn load_data(file: &str) -> String {
    fs::read_to_string(format!("tests/data/{file}.json")).expect("could not find test data")
}

pub struct TestService {
    suffix: &'static str,
}

impl TestService {
    pub fn new(suffix: &'static str) -> Self {
        Self { suffix }
    }
}

impl Service for TestService {
    fn get(&self, uri: Uri) -> Option<RawResponse> {
        Some(fs::read_to_string(uri).expect("could not find test data"))
    }

    fn get_resource(&self, _username: &str, resource: &str) -> Option<JsonResponse> {
        let filename = format!("tests/data/{resource}_{}.json", self.suffix);
        self.get(&filename)
    }

    fn user_agent(&self) -> String {
        format!("test-service-please-ignore v{}", env!("CARGO_PKG_VERSION"))
    }
}

pub struct FrozenClock {
    datetime: DateTime<Utc>,
}

impl FrozenClock {
    pub fn new(datetime: DateTime<Utc>) -> Self {
        FrozenClock { datetime }
    }
}

impl Default for FrozenClock {
    fn default() -> Self {
        let datetime = DateTime::parse_from_rfc3339("2025-05-23T10:13:00-07:00")
            .expect("invalid date supplied")
            .with_timezone(&Utc);
        Self::new(datetime)
    }
}

impl Clock for FrozenClock {
    fn now(&self) -> DateTime<Utc> {
        self.datetime
    }
}

impl Redditor {
    /// Returns a valid Redditor with 100 submissions and 100 comments
    /// that can be used for testing purposes.
    pub fn test() -> Redditor {
        Redditor::new(String::from("mipadi"), TestService::new("mipadi")).unwrap()
    }

    /// Returns a valid Redditor with no submissions nor comments that can
    /// be used for testing purposes.
    pub fn test_empty() -> Redditor {
        Redditor::new(
            String::from("testuserpleaseignore"),
            TestService::new("empty"),
        )
        .unwrap()
    }

    /// Returns a non-existent Redditor.
    pub fn test_none() -> Option<Redditor> {
        Redditor::new(String::from("doesnotexist"), TestService::new("404"))
    }
}
