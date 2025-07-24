//! OpenAI API client.
//!
//! When you create a client, you will have to select a [model](OpenAIModel) to use. By default,
//! the [cheapest](OpenAIModel::cheapest) model will be selected. Read the
//! [OpenAI model documentation](https://platform.openai.com/docs/models) for more information
//! on the various models offered by the OpenAI API.
//!
//! # Access
//!
//! You will need to set up an [OpenAI API account](https://platform.openai.com/docs/overview)
//! and generate your own authentication key to use OpenAI's API. Your key should be stored
//! under the `$OPENAI_API_KEY` environment variable for use with [`Auth`].
//!
//! **Note that you are solely responsible for paying the costs of OpenAI API access.** The
//! usaidwat developers are not responsible for costs you incur while making use of the usaidwat
//! summarization service or other AI services. Read on for details about OpenAI's API pricing.
//!
//! # Cost
//!
//! There's no such thing as a free lunch, and there's no such thing as free OpenAI access,
//! even if OpenAI is a "non-profit" that is building its technology for the betterment of
//! humanity (and not Sam Altman's bank account). When you create an OpenAI API client,
//! you will need to select an [`OpenAIModel`]. Models are billed on a per-token basis, where
//! a token is the smallest unit of text that the model reads and processes. There are three
//! types of tokens: input tokens, cached input tokens, and output tokens.
//!
//! - **Input tokens** are the token used in any _requests_ made to the OpenAPI AI. This is
//!   the "prompt" that usaidwat sends to OpenAI for summarization.
//! - **Cached input tokens** are input tokens that have been reused by GPT. Input tokens are
//!   reused by prompts that have a common prefix, as described
//!   [here](https://openai.com/index/api-prompt-caching/).
//! - **Output tokens** are tokens generated in the output that is sent back to a client in
//!   response to a request.
//!
//! Prices are expressed in US dollars per $1 million tokens. As of 17 July 2025, the prices
//! for each model are as follows.
//!
//! For the latest pricing, see OpenAI's [pricing](https://platform.openai.com/docs/pricing)
//! docs.
//!
//! | Model      | Descriptor        | Input    | Cached Input | Output  |
//! |------------|-------------------|----------|--------------|---------|
//! | Gpt4_1nano | gpt-4.1-nano      | $0.10   | $0.025        | $0.40   |
//! | Gpt4omini  | gpt-4o-mini       | $0.15   | $0.075        | $0.60   |
//! | Gpt4_1mini | gpt-4.1-mini      | $0.40   | $0.10         | $1.60   |
//! | O4mini     | o4-mini           | $1.10   | $0.275        | $4.40   |
//! | O3mini     | o3-mini           | $1.10   | $0.55         | $4.40   |
//! | Gpt4_1     | gpt-4.1           | $2.00   | $0.50         | $8.00   |
//! | O3         | o3                | $2.00   | $0.50         | $8.00   |
//! | Gpt4o      | gpt-4o            | $2.50   | $1.25         | $10.00  |
//! | ChatGpt4o  | chatgpt-4o-latest | $5.00   | -             | $15.00  |
//! | O1         | o1                | $15.00  | $7.50         | $60.00  |
//! | O3pro      | o3-pro            | $20.00  | -             | $80.00  |
//! | 01pro      | o1-pro            | $150.00 | -             | $600.00 |
//!
//! # See Also
//!
//! - [OpenAI model documentation](https://platform.openai.com/docs/models)

use crate::ai::Auth;
use crate::ai::client::{AIModel, APIClient, APIRequest, APIResponse, APIResult};
use crate::ai::service::{APIService, HTTPService};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::slice::Iter;

/// An OpenAI API client.
#[derive(Debug)]
pub struct OpenAIClient<T: APIService + Sync> {
    auth: Auth,
    service: T,
}

impl<T: APIService + Sync> APIClient for OpenAIClient<T> {
    type APIRequest = OpenAIRequest;
    type APIResponse = OpenAIResponse;

    // TODO: Test with a dummy service
    async fn send(&self, request: &Self::APIRequest) -> APIResult<Self::APIResponse> {
        self.service.post(Self::BASE_URI, &self.auth, request).await
    }
}

impl<T: APIService + Sync> OpenAIClient<T> {
    /// The base URI for OpenAI API requests.
    const BASE_URI: &'static str = "https://api.openai.com/v1/responses";

    fn new_with_service(auth: Auth, service: T) -> Self {
        Self { auth, service }
    }
}

impl OpenAIClient<HTTPService> {
    /// Create a new OpenAI client using the given authentication data.
    pub fn new(auth: Auth) -> Self {
        let service = HTTPService::new();
        Self::new_with_service(auth, service)
    }
}

/// A body for an OpenAI API request.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OpenAIRequest {
    model: OpenAIModel,

    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,

    input: String,

    store: bool,
}

impl APIRequest for OpenAIRequest {
    /// This request uses OpenAI GPT-specific [models](OpenAIModel).
    type Model = OpenAIModel;

    /// Sets the model used by the OpenAI API request.
    ///
    /// If not specified, the [default](OpenAIModel::default) model, gpt-4o,
    /// will be used. [According to OpenAI][1], gpt-4.1 also "offers a
    /// solid combination of intelligence, speed, and cost effectiveness".
    /// If you are on a budget, you can also try using the
    /// [least expensive](OpenAIModel::cheapest), too.
    ///
    /// [1]: https://platform.openai.com/docs/guides/text?api-mode=responses#choosing-a-model
    fn model(self, model: OpenAIModel) -> Self {
        Self { model, ..self }
    }

    /// Sets optional instructions for the request.
    ///
    /// Instructions provide high-level instructions on how a GPT model should
    /// behave while generating a response, including tone, goals, and examples
    /// of correct responses. Instructions take precedence over the prompt
    /// provided by the [`input`](OpenAIRequest::input) parameter.
    /// Instructions are not necessary if you do not wish to customize the
    /// response or provide guidance.
    fn instructions(self, instructions: impl Into<String>) -> Self {
        let instructions = Some(instructions.into());
        Self {
            instructions,
            ..self
        }
    }

    /// Sets the request's input.
    ///
    /// This is sometimes referred to as a "prompt" and represents a request
    /// made to GPT for which one or more responses are expected.
    ///
    /// If [instructions](OpenAIRequest::instructions) are provided,
    /// the instructions take precedence over this input.
    fn input(self, input: impl Into<String>) -> Self {
        let input = input.into();
        Self { input, ..self }
    }
}

/// Available OpenAI GPT models.
///
/// For more information on the differences between each model, see the
/// [OpenAI model documentation](https://platform.openai.com/docs/models).
///
/// The [default](OpenAIModel::default) is [gpt-4o](OpenAIModel::Gpt4o),
/// which OpenAI describes as "the best model to use for most tasks".
/// [According to its docs][1], [gpt-4.1](OpenAIModel::Gpt4_1) "offers a solid
/// combination of intelligence, speed, and cost effectiveness". If you are on
/// a budget, consider using [gpt-4.1-nano](OpenAIModel::Gpt4_1nano), the
/// [least expensive](OpenAIModel::cheapest) model.
///
/// # Cost
///
/// OpenAI API usage has a cost, and the cost of each model differs;
/// naturally more powerful models cost more to use.
///
/// See the [cost breakdown](self#Cost) in the `openai` module documentation
/// for more details,  or visit OpenAI's
/// [pricing](https://platform.openai.com/docs/pricing) docs for the last prices.
///
/// [1]: https://platform.openai.com/docs/guides/text?api-mode=responses#choosing-a-model
#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub enum OpenAIModel {
    /// The model currently used by ChatGPT.
    #[serde(rename = "chatgpt-4o-latest")]
    ChatGpt4o,

    /// Versatile, high-intelligence flagship model.
    ///
    /// This is the best model to use for most tasks.
    #[default]
    #[serde(rename = "gpt-4o")]
    Gpt4o,

    /// A fast, affordable model for focused tasks.
    #[serde(rename = "gpt-4o-mini")]
    Gpt4omini,

    /// The flagship model for complex tasks.
    ///
    /// It is well-suited for problem-solving across domains.
    #[serde(rename = "gpt-4.1")]
    Gpt4_1,

    /// Provides a balance between intelligence, speed, and cost.
    ///
    /// An attractive model for many use cases.
    #[serde(rename = "gpt-4.1-mini")]
    Gpt4_1mini,

    /// The fastest, most cost-effective 4.1 model.
    #[serde(rename = "gpt-4.1-nano")]
    Gpt4_1nano,

    /// Optimized for fast, effective reasoning with exceptionally efficient
    /// performance in coding and visual tasks.
    #[serde(rename = "o4-mini")]
    O4mini,

    /// A well-rounded and powerful reasoning model across domains.
    ///
    /// It sets a new standard for math, science, coding, and visual
    /// reasoning tasks, and excels at technical writing and following
    /// instructions.
    #[serde(rename = "o3")]
    O3,

    /// A mini version of the o3 model, providing high intelligence with
    /// the same cost and latency targets of o1-mini.
    #[serde(rename = "o3-mini")]
    O3mini,

    /// Like the o3 model, but it uses more compute to think even harder.
    #[serde(rename = "o3-pro")]
    O3pro,

    /// A model trained with reinforcement learning that thinks before it
    /// answers and produces a long chain of thought before responding to
    /// the user.
    #[serde(rename = "o1")]
    O1,

    /// A version of the [`o1`][OpenAIModel::O1] model that thinks even harder
    /// before responding.
    #[serde(rename = "o1-pro")]
    O1pro,
}

impl AIModel for OpenAIModel {
    /// The "best" GPT model as defined by OpenAI.
    fn best() -> Self {
        OpenAIModel::default()
    }

    fn cheapest() -> Self {
        OpenAIModel::Gpt4_1nano
    }

    fn fastest() -> Self {
        OpenAIModel::Gpt4_1nano
    }
}

impl fmt::Display for OpenAIModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_string(&self).expect(&format!("could not serialize {:?}", self));
        let s = s.trim_matches('"');
        f.write_fmt(format_args!("{}", s))
    }
}

/// A response from the OpenAI API.
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAIResponse {
    output: Vec<OpenAIOutput>,
}

impl APIResponse for OpenAIResponse {}

impl OpenAIResponse {
    /// GPT response output.
    ///
    /// There should be at least item in the output, but there could be
    /// multiple output objects.
    pub fn output(&self) -> Iter<OpenAIOutput> {
        self.output.iter()
    }
}

/// Generated GPT output.
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAIOutput {
    content: Vec<OpenAIContent>,
}

impl OpenAIOutput {
    /// Contents of the GPT API response.
    ///
    /// There should be at least one piece of content in the output,
    /// but there could be multiple content objects.
    pub fn content(&self) -> Iter<OpenAIContent> {
        self.content.iter()
    }
}

/// Content of GPT output.
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAIContent {
    text: String,
}

impl OpenAIContent {
    /// Generated GPT text.
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[cfg(test)]
mod test {
    mod client {
        use super::super::{APIClient, APIRequest, APIService};
        use super::super::{OpenAIClient, OpenAIRequest};
        use crate::ai::Auth;
        use crate::http::{HTTPError, HTTPResult, HTTPService};
        use reqwest::IntoUrl;
        use serde::Serialize;
        use serde::de::DeserializeOwned;
        use std::fs;

        struct TestAPIService {}

        impl HTTPService for TestAPIService {}

        impl APIService for TestAPIService {
            async fn post<U, D, R>(&self, _uri: U, _auth: &Auth, _data: &D) -> HTTPResult<R>
            where
                U: IntoUrl + Send,
                D: Serialize + Sync,
                R: DeserializeOwned,
            {
                let data = self.load_data();
                serde_json::from_str(&data).map_err(HTTPError::Serialization)
            }
        }

        impl TestAPIService {
            pub fn new() -> Self {
                Self {}
            }

            fn load_data(&self) -> String {
                fs::read_to_string(format!("tests/data/openai/responses.json"))
                    .expect("could not find test data")
            }
        }

        impl OpenAIClient<TestAPIService> {
            fn test() -> Self {
                let auth = Auth::new("some-api-key");
                OpenAIClient::new_with_service(auth, TestAPIService::new())
            }
        }

        #[tokio::test]
        async fn it_sends_a_request_and_returns_a_response() {
            let client = OpenAIClient::test();
            let request = OpenAIRequest::default().input("write a haiku about ai");
            let response = client.send(&request).await;
            assert!(response.is_ok());

            let response = response.unwrap();
            assert_eq!(response.output().count(), 1);
            assert_eq!(response.output().next().unwrap().content().count(), 1);
        }
    }

    mod request {
        use super::super::*;
        use indoc::indoc;

        #[test]
        fn it_serializes() {
            let body = OpenAIRequest::default()
                .model(OpenAIModel::Gpt4omini)
                .instructions("Please treat this as a test.")
                .input("Serialize me, GPT!");
            let expected = indoc! {"{
              \"model\": \"gpt-4o-mini\",
              \"instructions\": \"Please treat this as a test.\",
              \"input\": \"Serialize me, GPT!\",
              \"store\": false
            }"};
            let actual = serde_json::to_string_pretty(&body).unwrap();
            assert_eq!(
                actual, expected,
                "\n\nleft:\n{actual}\n\nright:\n{expected}\n"
            );
        }

        #[test]
        fn it_serializes_without_instructions() {
            let body = OpenAIRequest::default().input("Serialize me, GPT!");
            let expected = indoc! {"{
              \"model\": \"gpt-4o\",
              \"input\": \"Serialize me, GPT!\",
              \"store\": false
            }"};
            let actual = serde_json::to_string_pretty(&body).unwrap();
            assert_eq!(
                actual, expected,
                "\n\nleft:\n{actual}\n\nright:\n{expected}\n"
            );
        }

        #[test]
        fn it_deserializes() {
            let data = r#"{
                "model": "gpt-4o-mini",
                "instructions": "Please treat this as a test.",
                "input": "Deserialize me, GPT!",
                "store": false
            }"#;
            let body: OpenAIRequest = serde_json::from_str(data).unwrap();
            assert_eq!(body.model, OpenAIModel::Gpt4omini);
            assert!(body.instructions.is_some());
            assert_eq!(body.instructions.unwrap(), "Please treat this as a test.");
            assert_eq!(body.input, "Deserialize me, GPT!");
        }

        #[test]
        fn it_deserializes_without_instructions() {
            let data = r#"{
                "model": "gpt-4o",
                "input": "Deserialize me, GPT!",
                "store": false
            }"#;
            let body: OpenAIRequest = serde_json::from_str(data).unwrap();
            assert_eq!(body.model, OpenAIModel::Gpt4o);
            assert!(body.instructions.is_none());
            assert_eq!(body.input, "Deserialize me, GPT!");
        }
    }

    mod model {
        use super::super::*;

        #[test]
        fn it_returns_valid_descriptors() {
            let test_cases = vec![
                (OpenAIModel::ChatGpt4o, "chatgpt-4o-latest"),
                (OpenAIModel::Gpt4o, "gpt-4o"),
                (OpenAIModel::Gpt4omini, "gpt-4o-mini"),
                (OpenAIModel::Gpt4_1, "gpt-4.1"),
                (OpenAIModel::Gpt4_1mini, "gpt-4.1-mini"),
                (OpenAIModel::Gpt4_1nano, "gpt-4.1-nano"),
                (OpenAIModel::O4mini, "o4-mini"),
                (OpenAIModel::O3, "o3"),
                (OpenAIModel::O3mini, "o3-mini"),
                (OpenAIModel::O3pro, "o3-pro"),
                (OpenAIModel::O1, "o1"),
                (OpenAIModel::O1pro, "o1-pro"),
            ];

            for (model, descriptor) in test_cases {
                assert_eq!(model.to_string(), descriptor, "Model::{:?}", model);
            }
        }
    }
}
