//! OpenAI API client.

use crate::ai::client::{HasCost, ModelCost};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A body for an OpenAI API request.
#[derive(Default, Deserialize, Serialize)]
struct OpenAIRequestBody {
    model: Model,

    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,

    input: String,
}

impl OpenAIRequestBody {
    /// Sets the model used by the OpenAI API request.
    pub fn model(self, model: Model) -> Self {
        Self { model, ..self }
    }

    /// Sets optional instructions for the request.
    pub fn instructions(self, instructions: impl Into<String>) -> Self {
        let instructions = Some(instructions.into());
        Self {
            instructions,
            ..self
        }
    }

    /// Sets the request's input.
    pub fn input(self, input: impl Into<String>) -> Self {
        let input = input.into();
        Self { input, ..self }
    }
}

/// Available OpenAI GPT models.
#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub enum Model {
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

    /// A version of the [`o1`][Model::O1] model that thinks even harder
    /// before responding.
    #[serde(rename = "o1-pro")]
    O1pro,
}

impl Model {
    /// The least expensive available model.
    pub fn cheapest() -> Self {
        Model::Gpt4_1nano
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_string(&self).expect(&format!("could not serialize {:?}", self));
        let s = s.trim_matches('"');
        f.write_fmt(format_args!("{}", s))
    }
}

impl HasCost for Model {
    fn cost(&self) -> ModelCost {
        let (i, c, o) = match self {
            Model::ChatGpt4o => (5.0, None, 15.0),
            Model::Gpt4o => (2.5, Some(1.5), 10.0),
            Model::Gpt4omini => (0.15, Some(0.075), 0.60),
            Model::Gpt4_1 => (2.0, Some(0.5), 8.0),
            Model::Gpt4_1mini => (0.4, Some(0.1), 1.6),
            Model::Gpt4_1nano => (0.1, Some(0.025), 0.4),
            Model::O4mini => (1.1, Some(0.275), 4.4),
            Model::O3 => (2.0, Some(0.5), 8.0),
            Model::O3mini => (1.1, Some(0.55), 4.4),
            Model::O3pro => (20.0, None, 80.0),
            Model::O1 => (15.0, Some(7.5), 60.0),
            Model::O1pro => (150.0, None, 600.0),
        };
        ModelCost::new(i * 100.0, c.map(|cents| cents * 100.0), o * 100.0)
    }
}

#[cfg(test)]
mod test {
    mod request_body {
        use super::super::*;
        use indoc::indoc;

        #[test]
        fn it_serializes() {
            let body = OpenAIRequestBody::default()
                .model(Model::Gpt4omini)
                .instructions("Please treat this as a test.")
                .input("Serialize me, GPT!");
            let expected = indoc! {"{
              \"model\": \"gpt-4o-mini\",
              \"instructions\": \"Please treat this as a test.\",
              \"input\": \"Serialize me, GPT!\"
            }"};
            let actual = serde_json::to_string_pretty(&body).unwrap();
            assert_eq!(
                actual, expected,
                "\n\nleft:\n{actual}\n\nright:\n{expected}\n"
            );
        }

        #[test]
        fn it_serializes_without_instructions() {
            let body = OpenAIRequestBody::default().input("Serialize me, GPT!");
            let expected = indoc! {"{
              \"model\": \"gpt-4o\",
              \"input\": \"Serialize me, GPT!\"
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
                "input": "Deserialize me, GPT!"
            }"#;
            let body: OpenAIRequestBody = serde_json::from_str(data).unwrap();
            assert_eq!(body.model, Model::Gpt4omini);
            assert!(body.instructions.is_some());
            assert_eq!(body.instructions.unwrap(), "Please treat this as a test.");
            assert_eq!(body.input, "Deserialize me, GPT!");
        }

        #[test]
        fn it_deserializes_without_instructions() {
            let data = r#"{
                "model": "gpt-4o",
                "input": "Deserialize me, GPT!"
            }"#;
            let body: OpenAIRequestBody = serde_json::from_str(data).unwrap();
            assert_eq!(body.model, Model::Gpt4o);
            assert!(body.instructions.is_none());
            assert_eq!(body.input, "Deserialize me, GPT!");
        }
    }

    mod model {
        use super::super::*;

        #[test]
        fn it_returns_valid_descriptors() {
            let test_cases = vec![
                (Model::ChatGpt4o, "chatgpt-4o-latest"),
                (Model::Gpt4o, "gpt-4o"),
                (Model::Gpt4omini, "gpt-4o-mini"),
                (Model::Gpt4_1, "gpt-4.1"),
                (Model::Gpt4_1mini, "gpt-4.1-mini"),
                (Model::Gpt4_1nano, "gpt-4.1-nano"),
                (Model::O4mini, "o4-mini"),
                (Model::O3, "o3"),
                (Model::O3mini, "o3-mini"),
                (Model::O3pro, "o3-pro"),
                (Model::O1, "o1"),
                (Model::O1pro, "o1-pro"),
            ];

            for (model, descriptor) in test_cases {
                assert_eq!(model.to_string(), descriptor, "Model::{:?}", model);
            }
        }
    }
}
