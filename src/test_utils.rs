use crate::clock::{Clock, DateTime, Utc};
use crate::http::{HTTPResult, HTTPService};
use crate::reddit::client::Redditor;
use crate::reddit::service::Service;
use reqwest::IntoUrl;
use std::fs;

pub fn do_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}

pub fn load_data(file: &str) -> String {
    fs::read_to_string(format!("tests/data/{file}.json")).expect("could not find test data")
}

pub fn load_output(filename: &str) -> String {
    let filename = format!("tests/output/{filename}.out");
    String::from(
        fs::read_to_string(&filename)
            .expect(&format!("could not load test data from {filename}"))
            .trim_end(),
    )
}

pub struct TestService<'a> {
    suffix: &'a str,
}

impl<'a> TestService<'a> {
    pub fn new(suffix: &'a str) -> Self {
        Self { suffix }
    }
}

impl<'a> HTTPService for TestService<'a> {
    async fn get<U>(&self, uri: U) -> HTTPResult<String>
    where
        U: IntoUrl + Send,
    {
        Ok(fs::read_to_string(uri.as_str()).expect("could not find test data"))
    }

    fn user_agent() -> String {
        format!("test-service-please-ignore v{}", env!("CARGO_PKG_VERSION"))
    }
}

impl<'a> Service for TestService<'a> {
    async fn get_resource(&self, _username: &str, resource: &str) -> HTTPResult<String> {
        let filename = format!("tests/data/{resource}_{}.json", self.suffix);
        self.get(&filename).await
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
    pub async fn test() -> Redditor {
        Redditor::new_with_service(String::from("mipadi"), TestService::new("mipadi"))
            .await
            .unwrap()
    }

    /// Returns a valid Redditor with no submissions nor comments that can
    /// be used for testing purposes.
    pub async fn test_empty() -> Redditor {
        Redditor::new_with_service(
            String::from("testuserpleaseignore"),
            TestService::new("empty"),
        )
        .await
        .unwrap()
    }

    /// Returns a non-existent Redditor.
    pub async fn test_none() -> Option<Redditor> {
        Redditor::new_with_service(String::from("doesnotexist"), TestService::new("404"))
            .await
            .ok()
    }
}
