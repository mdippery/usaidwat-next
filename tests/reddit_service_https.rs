use usaidwat::service::{RedditService, Service};
use uuid::Uuid;

// Not really the most interesting tests, but these are testing live HTTPS
// integration and there's not really a consistent way to determine what we
// get back, so merely checking that we're not getting an empty string will
// suffice until I can come up with a better way.
// I mostly just want to make sure that the types and everything are correct.

#[test]
fn it_retrieves_profiles() {
    let service = RedditService::new();
    let resp = service.get_resource("mipadi", "about").unwrap();
    assert_ne!(resp, "");
}

#[test]
fn it_retrieves_comments() {
    let service = RedditService::new();
    let resp = service.get_resource("mipadi", "comments").unwrap();
    assert_ne!(resp, "");
}

#[test]
fn it_retrieves_posts() {
    let service = RedditService::new();
    let resp = service.get_resource("mipadi", "submitted").unwrap();
    assert_ne!(resp, "");
}

#[test]
fn it_returns_none_for_invalid_users() {
    let service = RedditService::new();
    let user = Uuid::new_v4().to_string();
    let resp = service.get_resource(&user, "about");
    assert!(resp.is_none(), "response was {resp:?}");
}
