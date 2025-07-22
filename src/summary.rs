//! AI summarization.

use crate::markdown;
use crate::reddit::Redditor;
use itertools::Itertools;

/// Summarizes a Redditor's comments and provides a sentiment analysis using AI.
#[derive(Debug)]
pub struct Summarizer<'a> {
    user: &'a Redditor,
}

impl<'a> Summarizer<'a> {
    /// Summarizes content from the given `user`.
    pub fn for_user(user: &'a Redditor) -> Self {
        Self { user }
    }

    /// Raw content that will be sent to an LLM for summarization.
    pub fn context(&self) -> String {
        self.user
            .comments()
            .map(|c| markdown::summarize(c.markdown_body()))
            .join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use crate::reddit::Redditor;
    use crate::summary::Summarizer;
    use crate::test_utils::load_output;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn it_provides_context_for_an_llm() {
        let redditor = Redditor::test().await;
        let expected = load_output("summary_raw");
        let actual = Summarizer::for_user(&redditor).context();
        assert_eq!(actual, expected);
    }
}
