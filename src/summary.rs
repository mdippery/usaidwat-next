//! AI summarization.

use crate::client::Redditor;
use crate::markdown;
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

    /// Dumps the raw content that will be sent to an LLM for summarization.
    pub fn dump(&self) -> String {
        self.user
            .comments()
            .map(|c| markdown::summarize(c.markdown_body()))
            .join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use crate::client::Redditor;
    use crate::summary::Summarizer;
    use crate::test_utils::load_output;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_dumps_redditor_comments() {
        let redditor = Redditor::test();
        let expected = load_output("summary_raw");
        let actual = Summarizer::for_user(&redditor).dump();

        assert_eq!(actual, expected);
    }
}
