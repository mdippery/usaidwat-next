use crate::client::Redditor;
use chrono::Local;
use indoc::{formatdoc, indoc};

pub trait Viewable {
    fn view(&self) -> String;
}

impl Viewable for Redditor {
    fn view(&self) -> String {
        formatdoc! {"
            Created: {} ({})
            Link Karma: {}
            Comment Karma: {}",
            self.created_at().with_timezone(&Local).format("%b %d, %Y %H:%M %p"),
            self.relative_age(),
            self.link_karma(),
            self.comment_karma(),
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: All of these test services should be moved to a global module
    //       so we can use them across tests.
    use crate::client::Redditor;
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

    fn test_service() -> TestService {
        TestService::new("mipadi")
    }

    fn test_client() -> Redditor {
        Redditor::new(String::from("mipadi"), Box::new(test_service())).unwrap()
    }

    mod format_info {
        use super::super::*;
        use super::test_client;

        #[test]
        fn it_formats_a_user() {
            let user = test_client();
            let actual = user.view();
            // TODO: Eventually the "17 years" part will fail, so I
            //       really should be mocking time, but we'll cross that
            //       bridge when we come to it.
            //       Will also have to mock Local so the tests always use
            //       the same local time zone (PDT).
            let expected = indoc! {"
                Created: Mar 31, 2008 15:55 PM (17 years ago)
                Link Karma: 11729
                Comment Karma: 121995"};
            assert_eq!(actual, expected);
        }
    }
}
