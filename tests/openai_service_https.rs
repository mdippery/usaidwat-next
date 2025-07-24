// These tests aren't particularly interesting and mostly serve to ensure
// that we can actually connect to the OpenAI service. Somewhat redundant
// with openai_client_https, but it ensures we are testing the integration
// of each individual component.

use usaidwat::ai::Auth;
use usaidwat::ai::client::openai::{OpenAIModel, OpenAIRequest, OpenAIResponse};
use usaidwat::ai::client::{AIModel, APIRequest};
use usaidwat::ai::service::{APIService, HTTPService};
use usaidwat::http::HTTPResult;

#[tokio::test]
async fn it_sends_a_post_request() {
    let auth =
        Auth::from_env("OPENAI_API_KEY").expect("Could not create auth. Is $OPENAI_API_KEY set?");
    let request = OpenAIRequest::default()
        .model(OpenAIModel::cheapest())
        .input("write a haiku about ai");
    let service = HTTPService::new();
    let response: HTTPResult<OpenAIResponse> = service
        .post("https://api.openai.com/v1/responses", &auth, &request)
        .await;
    let resp = response.expect("could not make OpenAI API request");
    assert_eq!(resp.output().count(), 1);
    assert_eq!(resp.output().next().unwrap().content().count(), 1);
}
