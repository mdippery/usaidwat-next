//! AI summarization.

use crate::ai::Auth;
use crate::ai::client::openai::{OpenAIClient, OpenAIModel, OpenAIRequest, OpenAIResponse};
use crate::ai::client::{APIClient, APIRequest};
use crate::markdown;
use crate::reddit::Redditor;
use itertools::Itertools;

/// Summarizes a Redditor's comments and provides a sentiment analysis using AI.
#[derive(Debug)]
pub struct Summarizer<'a> {
    user: &'a Redditor,
    // TODO: Needs to be generalized (associated type?) to work with AI model
    //       specific to client/request type
    model: OpenAIModel,
}

impl<'a> Summarizer<'a> {
    const PREAMBLE: &'static str = include_str!("summary_prompt.txt");

    /// Summarizes content from the given `user`.
    pub fn for_user(user: &'a Redditor) -> Self {
        Self {
            user,
            model: OpenAIModel::default(),
        }
    }

    /// Sets the AI model used for summarization.
    ///
    /// By default, the summarizer uses [`OpenAIModel::default()`], but
    /// that option can be changed here.
    pub fn model(self, model: OpenAIModel) -> Self {
        Self { model, ..self }
    }

    /// Summarize the Redditor's comments and return the summary as a string,
    /// along with a sentiment analysis at the end.
    pub async fn summarize(&self) -> OpenAIResponse {
        // TODO: TEST THIS!
        // Also, need an easy tool for generating mock input and output
        // that we can save for testing.

        // TODO: Return a string, not an OpenAIResponse!
        //       Or maybe a structure suitable for passing to view(),
        //       or otherwise one that can be wrapped to terminal width.
        // TODO: Let callers specify a client so we can test this easier!

        // TODO: Probably need to result a Result or specify Auth earlier!
        let auth = Auth::from_env("OPENAI_API_KEY").unwrap();
        let client = OpenAIClient::new(auth);

        // TODO: Might want to separate instructions from text to summarize,
        //       or at least pass some of the preamble as instructions.
        //       Iterate on this!
        // TODO: Really should test that request is formed properly, but
        //       that's going to be hard to do without accessors, etc.
        let request = OpenAIRequest::default()
            .model(self.model)
            .input(self.input());

        // TODO: Error handling!
        client.send(&request).await.unwrap()
    }

    /// Raw content that will be sent to an LLM for summarization.
    ///
    /// This is essentially all of a Redditor's comments stripped of
    /// formatting. It does not include the introductory instructions;
    /// see [`Summarizer::preamble()`] for the complete prompt that is
    /// sent to the LLM.
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
    /// instructions along with the [`Summarizer::context()`].
    pub fn input(&self) -> String {
        format!("{}\n\n{}", self.preamble(), self.context())
    }
}

#[cfg(test)]
mod tests {
    use crate::ai::client::openai::OpenAIModel;
    use crate::reddit::Redditor;
    use crate::summary::Summarizer;
    use crate::test_utils::load_output;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn it_uses_the_default_model_if_one_is_not_provided() {
        let redditor = Redditor::test().await;
        let summarizer = Summarizer::for_user(&redditor);
        assert_eq!(summarizer.model, OpenAIModel::default());
    }

    #[tokio::test]
    async fn it_allows_model_to_be_configured() {
        let redditor = Redditor::test().await;
        let summarizer = Summarizer::for_user(&redditor).model(OpenAIModel::O1pro);
        assert_eq!(summarizer.model, OpenAIModel::O1pro);
    }

    #[tokio::test]
    async fn it_provides_context_for_an_llm() {
        let redditor = Redditor::test().await;
        let expected = load_output("summary_raw");
        let actual = Summarizer::for_user(&redditor).context();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn it_provides_a_preamble_for_an_llm() {
        let redditor = Redditor::test().await;
        let expected = include_str!("summary_prompt.txt");
        let expected = expected.replace('\n', " ");
        let actual = Summarizer::for_user(&redditor).preamble();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn it_provides_input_for_an_llm() {
        let redditor = Redditor::test().await;
        let instructions = include_str!("summary_prompt.txt");
        let instructions = instructions.replace('\n', " ");
        let summary = load_output("summary_raw");
        let expected = format!("{}\n\n{}", instructions, summary);
        let actual = Summarizer::for_user(&redditor).input();
        assert_eq!(actual, expected);
    }
}
