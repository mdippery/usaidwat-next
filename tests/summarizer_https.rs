use cogito::service::Auth;
use cogito_openai::client::OpenAIClient;
use hypertyper::HttpClientFactory;
use usaidwat::reddit::Redditor;
use usaidwat::summary::Summarizer;

#[tokio::test]
#[ignore = "long test"]
async fn it_summarizes_a_redditors_comments() {
    let auth = Auth::from_env("OPENAI_API_KEY").expect("$OPENAI_API_KEY is not defined");
    let factory = HttpClientFactory::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let client = OpenAIClient::new(auth, factory);
    let user = Redditor::new("mipadi")
        .await
        .expect("could not create redditor");
    let summarizer = Summarizer::new(client, &user);
    let response = summarizer.summarize(false).await;
    assert!(response.is_ok());

    let response = response.unwrap();
    assert!(response.len() > 0);
}
