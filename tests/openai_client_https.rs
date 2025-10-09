use hypertyper::HTTPClientFactory;
use usaidwat::ai::Auth;
use usaidwat::ai::client::openai::{OpenAIClient, OpenAIModel, OpenAIRequest};
use usaidwat::ai::client::{APIClient, APIRequest};

// These tests aren't particularly interesting and mostly serve to ensure
// that we can actually connect to the OpenAI service.

#[tokio::test]
async fn it_sends_a_request_using_gpt_4o() {
    let auth =
        Auth::from_env("OPENAI_API_KEY").expect("Could not create auth. Is $OPENAI_API_KEY set?");
    let factory = HTTPClientFactory::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let client = OpenAIClient::new(auth, factory);
    let req = OpenAIRequest::default()
        .model(OpenAIModel::Gpt4o)
        .input("write a haiku about ai");
    let resp = client.send(&req).await;
    let resp = resp.expect("could not make OpenAI API request");
    assert_eq!(resp.output().len(), 1);
    assert_eq!(resp.output().next().unwrap().content().len(), 1);
}

#[tokio::test]
async fn it_sends_a_request_using_gpt_5nano() {
    let auth =
        Auth::from_env("OPENAI_API_KEY").expect("Could not create auth. Is $OPENAI_API_KEY set?");
    let factory = HTTPClientFactory::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let client = OpenAIClient::new(auth, factory);
    let req = OpenAIRequest::default()
        .model(OpenAIModel::Gpt5nano)
        .input("write a haiku about ai");
    let resp = client.send(&req).await;
    let resp = resp.expect("could not make OpenAI API request");
    assert_eq!(resp.output().len(), 2);

    let output = resp.output().nth(1).expect(&format!(
        "could not get second element of output: {:?}",
        resp
    ));
    assert_eq!(output.content().len(), 1);
}
