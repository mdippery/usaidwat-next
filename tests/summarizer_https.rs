use usaidwat::ai::Auth;
use usaidwat::ai::client::openai::OpenAIClient;
use usaidwat::reddit::Redditor;
use usaidwat::summary::Summarizer;

#[tokio::test]
#[ignore = "long test"]
async fn it_summarizes_a_redditors_comments() {
    let auth = Auth::from_env("OPENAI_API_KEY").expect("$OPENAI_API_KEY is not defined");
    let client = OpenAIClient::new(auth);
    let user = Redditor::new("mipadi")
        .await
        .expect("could not create redditor");
    let summarizer = Summarizer::new(client, &user);
    let response = summarizer.summarize().await;
    assert!(response.is_ok());

    let response = response.unwrap();
    assert!(response.len() > 0);
}
