use crate::client::Redditor;
use crate::service::{JsonResponse, RawResponse, Service, Uri};
#[cfg(test)]
use std::fs;

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

    fn get_resource(&self, username: &str, resource: &str) -> Option<JsonResponse> {
        let filename = format!("tests/data/{resource}_{}.json", self.suffix);
        self.get(filename)
    }

    fn user_agent(&self) -> String {
        format!("test-service-please-ignore v{}", env!("CARGO_PKG_VERSION"))
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
