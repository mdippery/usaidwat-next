//! AI summarization.

use crate::ai::client::{APIClient, APIRequest, APIResponse};
use crate::markdown;
use crate::reddit::Redditor;
use itertools::Itertools;

/// Summarizes a Redditor's comments and provides a sentiment analysis using AI.
#[derive(Debug)]
pub struct Summarizer<'a, C>
where
    C: APIClient,
    C::APIRequest: APIRequest,
{
    client: C,
    user: &'a Redditor,
    model: <C::APIRequest as APIRequest>::Model,
}

impl<'a, C> Summarizer<'a, C>
where
    C: APIClient,
{
    const PREAMBLE: &'static str = include_str!("summary_prompt.txt");

    /// Summarizes content from the given `user`.
    ///
    /// `auth` will be used when making requests to the AI service.
    pub fn new(client: C, user: &'a Redditor) -> Self {
        Self {
            client,
            user,
            model: <C::APIRequest as APIRequest>::Model::default(),
        }
    }

    /// Sets the AI model used for summarization.
    ///
    /// By default, the summarizer uses the default model, but that option can
    /// be changed here.
    pub fn model(self, model: <C::APIRequest as APIRequest>::Model) -> Self {
        Self { model, ..self }
    }

    /// Summarize the Redditor's comments and return the summary as a string,
    /// including an analysis of sentiment and tone.
    pub async fn summarize(&self) -> String {
        // We might want to separate instructions from text to summarize,
        // or at least pass some of the preamble as instructions.
        // Iterate on this.
        let request = C::APIRequest::default()
            .model(self.model)
            .input(self.input());

        // TODO: Error handling!
        // Do we need a unified Result and Error enum, or at least a
        // unified module?
        self.client.send(&request).await.unwrap().concatenate()
    }

    /// Raw content that will be sent to an LLM for summarization.
    ///
    /// This is essentially all of a Redditor's comments stripped of
    /// formatting. It does not include the introductory instructions
    /// set by the [preamble](Summarizer::preamble()).
    pub fn context(&self) -> String {
        self.user
            .comments()
            .map(|c| markdown::summarize(c.markdown_body()))
            .join("\n\n")
    }

    /// The initial prompt sent to the LLM.
    ///
    /// This is the set of instructions occurring before the text to be
    /// summarized.
    pub fn preamble(&self) -> String {
        Self::PREAMBLE.replace('\n', " ")
    }

    /// The full input sent to the LLM, including any introductory
    /// instructions along with the [context](Summarizer::context()).
    pub fn input(&self) -> String {
        format!("{}\n\n{}", self.preamble(), self.context())
    }
}

#[cfg(test)]
mod tests {
    use crate::ai::client::openai::OpenAIResponse;
    use crate::ai::client::{AIModel, APIClient, APIRequest, APIResponse, APIResult};
    use crate::reddit::Redditor;
    use crate::summary::Summarizer;
    use crate::test_utils::load_output;
    use std::fs;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Copy, Default, Debug, PartialEq)]
    enum TestAIModel {
        #[default]
        TestAIModel,

        OtherAIModel,
    }

    impl AIModel for TestAIModel {
        fn best() -> Self {
            TestAIModel::TestAIModel
        }

        fn cheapest() -> Self {
            TestAIModel::TestAIModel
        }

        fn fastest() -> Self {
            TestAIModel::TestAIModel
        }
    }

    #[derive(Clone, Debug, Default)]
    struct TestAPIRequest {
        model: TestAIModel,
        instructions: Option<String>,
        input: String,
    }

    impl APIRequest for TestAPIRequest {
        type Model = TestAIModel;

        fn model(self, model: Self::Model) -> Self {
            Self { model, ..self }
        }

        fn instructions(self, instructions: impl Into<String>) -> Self {
            Self {
                instructions: Some(instructions.into()),
                ..self
            }
        }

        fn input(self, input: impl Into<String>) -> Self {
            Self {
                input: input.into(),
                ..self
            }
        }
    }

    #[derive(Debug)]
    struct TestAPIResponse;

    impl APIResponse for TestAPIResponse {
        fn concatenate(&self) -> String {
            let json_data = fs::read_to_string("tests/data/openai/responses_multi_content.json")
                .expect("could not load file");
            let wrapped: OpenAIResponse =
                serde_json::from_str(&json_data).expect("could not parse json");
            wrapped.concatenate()
        }
    }

    #[derive(Debug)]
    struct RequestSpy {
        request: Option<TestAPIRequest>,
    }

    impl RequestSpy {
        fn new() -> Self {
            Self { request: None }
        }

        fn record(&mut self, request: TestAPIRequest) {
            self.request = Some(request)
        }
    }

    #[derive(Debug)]
    struct TestAIClient {
        request_spy: Arc<Mutex<RequestSpy>>,
    }

    impl TestAIClient {
        fn new() -> Self {
            let request_spy = Arc::new(Mutex::new(RequestSpy::new()));
            Self { request_spy }
        }
    }

    impl APIClient for TestAIClient {
        type APIRequest = TestAPIRequest;
        type APIResponse = TestAPIResponse;

        async fn send(&self, request: &Self::APIRequest) -> APIResult<Self::APIResponse> {
            self.request_spy
                .lock()
                .expect("could not lock mutex")
                .record(request.clone());
            Ok(Self::APIResponse {})
        }
    }

    impl<'a> Summarizer<'a, TestAIClient> {
        pub fn test(user: &'a Redditor) -> Self {
            let client = TestAIClient::new();
            Self::new(client, user)
        }
    }

    fn load_preamble() -> String {
        include_str!("summary_prompt.txt").replace('\n', " ")
    }

    fn load_summary() -> String {
        load_output("summary_raw")
    }

    fn load_input() -> String {
        let premble = load_preamble();
        let summary = load_summary();
        format!("{}\n\n{}", premble, summary)
    }

    #[tokio::test]
    async fn it_uses_the_default_model_if_one_is_not_provided() {
        let redditor = Redditor::test().await;
        let summarizer = Summarizer::test(&redditor);
        assert_eq!(summarizer.model, TestAIModel::default());
    }

    #[tokio::test]
    async fn it_allows_model_to_be_configured() {
        let redditor = Redditor::test().await;
        let summarizer = Summarizer::test(&redditor).model(TestAIModel::OtherAIModel);
        assert_eq!(summarizer.model, TestAIModel::OtherAIModel);
    }

    #[tokio::test]
    async fn it_provides_context_for_an_llm() {
        let redditor = Redditor::test().await;
        let expected = load_summary();
        let actual = Summarizer::test(&redditor).context();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn it_provides_a_preamble_for_an_llm() {
        let redditor = Redditor::test().await;
        let expected = load_preamble();
        let actual = Summarizer::test(&redditor).preamble();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn it_provides_input_for_an_llm() {
        let redditor = Redditor::test().await;
        let expected = load_input();
        let actual = Summarizer::test(&redditor).input();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn it_sends_a_request_with_the_correct_model_and_input() {
        let expected_instructions = load_input();

        let redditor = Redditor::test().await;
        let summarizer = Summarizer::test(&redditor).model(TestAIModel::OtherAIModel);
        let _ = summarizer.summarize().await;
        let client = summarizer.client;
        let request = &client
            .request_spy
            .lock()
            .expect("could not lock mutex")
            .request
            .take()
            .expect("could not get request");

        assert_eq!(request.model, TestAIModel::OtherAIModel);
        assert_eq!(request.input, expected_instructions);
        assert!(request.instructions.is_none());
    }

    #[tokio::test]
    async fn it_summarizes_a_response_and_returns_a_string() {
        let redditor = Redditor::test().await;
        let summarizer = Summarizer::test(&redditor);
        let expected = vec![
            "Silent circuits hum,  ",
            "Thoughts woven in coded threads,  ",
            "Dreams of silicon.",
            "Silicon whispers,  ",
            "Dreams woven in code and light,  ",
            "Thoughts beyond the stars.",
            "Wires hum softly,  ",
            "Thoughts of silicon arise\u{2014}  ",
            "Dreams in coded light.  ",
            "Silent circuits hum,  ",
            "Thoughts woven in code's embrace\u{2014}  ",
            "Dreams of minds reborn.",
            "Lines of code and dreams,  ",
            "Whispers of thought intertwined\u{2014}  ",
            "Silent minds awake.",
        ]
        .join("\n");
        let actual = summarizer.summarize().await;
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    #[ignore = "figure out how to spy well"]
    async fn it_calls_concatenate_on_the_response() {
        todo!()
    }
}
