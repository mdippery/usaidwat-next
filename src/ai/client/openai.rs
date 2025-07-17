//! OpenAI API client.

use crate::ai::client::{HasCost, ModelCost};
use std::fmt;

/// Available OpenAI GPT models.
#[derive(Debug, Default)]
pub enum Model {
    /// The model currently used by ChatGPT.
    ChatGpt4o,

    /// Versatile, high-intelligence flagship model.
    ///
    /// This is the best model to use for most tasks.
    #[default]
    Gpt4o,

    /// A fast, affordable model for focused tasks.
    Gpt4omini,

    /// The flagship model for complex tasks.
    ///
    /// It is well-suited for problem-solving across domains.
    Gpt4_1,

    /// Provides a balance between intelligence, speed, and cost.
    ///
    /// An attractive model for many use cases.
    Gpt4_1mini,

    /// The fastest, most cost-effective 4.1 model.
    Gpt4_1nano,

    /// Optimized for fast, effective reasoning with exceptionally efficient
    /// performance in coding and visual tasks.
    O4mini,

    /// A well-rounded and powerful reasoning model across domains.
    ///
    /// It sets a new standard for math, science, coding, and visual
    /// reasoning tasks, and excels at technical writing and following
    /// instructions.
    O3,

    /// A mini version of the o3 model, providing high intelligence with
    /// the same cost and latency targets of o1-mini.
    O3mini,

    /// Like the o3 model, but it uses more compute to think even harder.
    O3pro,

    /// A model trained with reinforcement learning that thinks before it
    /// answers and produces a long chain of thought before responding to
    /// the user.
    O1,

    /// A version of the [`o1`][Model::O1] model that thinks even harder
    /// before responding.
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
        match self {
            Model::ChatGpt4o => write!(f, "chatgpt-4o-latest"),
            Model::Gpt4o => write!(f, "gpt-4o"),
            Model::Gpt4omini => write!(f, "gpt-4o-mini"),
            Model::Gpt4_1 => write!(f, "gpt-4.1"),
            Model::Gpt4_1mini => write!(f, "gpt-4.1-mini"),
            Model::Gpt4_1nano => write!(f, "gpt-4.1-nano"),
            Model::O4mini => write!(f, "o4-mini"),
            Model::O3 => write!(f, "o3"),
            Model::O3mini => write!(f, "o3-mini"),
            Model::O3pro => write!(f, "o3-pro"),
            Model::O1 => write!(f, "o1"),
            Model::O1pro => write!(f, "o1-pro"),
        }
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
