use usaidwat::reddit::Redditor;
use usaidwat::summary::Summarizer;

#[tokio::test]
#[ignore = "long test"]
async fn it_summarizes_a_redditors_comments() {
    let user = Redditor::new("mipadi")
        .await
        .expect("could not create redditor");
    let summarizer = Summarizer::for_user(&user);
    let response = summarizer.summarize().await;

    // TODO: Test results
    eprintln!("Summarization:\n{:?}", response);
    eprintln!(
        "JSON:\n{}",
        serde_json::to_string_pretty(&response).unwrap()
    );
}
