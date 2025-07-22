use usaidwat::ai::Auth;
use usaidwat::ai::client::openai::{Model, OpenAIClient, OpenAIRequest};
use usaidwat::ai::client::{APIClient, APIRequest};

// These tests aren't particularly interesting and mostly serve to ensure
// that we can actually connect to the OpenAI service.

#[tokio::test]
async fn it_sends_a_request() {
    let auth =
        Auth::from_env("OPENAI_API_KEY").expect("Could not create auth. Is $OPENAI_API_KEY set?");
    let client = OpenAIClient::new(auth);
    let req = OpenAIRequest::default()
        .model(Model::cheapest())
        .input("write a haiku about ai");
    let resp = client.send(&req).await;
    let resp = resp.expect("could not make OpenAI API request");
    assert_eq!(resp.output().len(), 1);
    assert_eq!(resp.output().next().unwrap().content().len(), 1);
}
