//! General-purpose counting capabilities.

use crate::thing::HasSubreddit;
use counter::Counter;
use itertools::Itertools;

/// Differentiates between the different sorting algorithms used to
/// return subreddit counts.
#[derive(Debug, Default)]
pub enum SortAlgorithm {
    /// Sort counts by name of subreddit.
    #[default]
    Lexicographically,

    /// Sort counts by number of comments or submissions in each subreddit.
    Numerically,
}

/// A pair of subreddit name and count.
pub type SubredditCount = (String, usize);

/// Groups Reddit comments and submissions by subreddit and provides a
/// count of the number of items in each subreddit.
#[derive(Debug)]
pub struct SubredditCounter {
    counts: Counter<String>,
}

impl SubredditCounter {
    /// Groups and counts comments and submissions.
    ///
    /// `iter` is an iterator of `Comments` or `Submissions`, or anything
    /// that has a subreddit attached to it.
    pub fn from_iter<T: HasSubreddit>(iter: impl Iterator<Item = T>) -> Self {
        let counts = SubredditCounter::count(iter);
        SubredditCounter { counts }
    }

    /// Sorts the subreddit counts by subreddit name or the count of items in
    /// the subreddit.
    ///
    /// Returns a vector of (subreddit name, count) pairs.
    pub fn sort_by(&self, algo: &SortAlgorithm) -> Vec<SubredditCount> {
        match algo {
            SortAlgorithm::Numerically => self.sort_numerically(),
            SortAlgorithm::Lexicographically => self.sort_lexicographically(),
        }
    }

    fn count<T: HasSubreddit>(iter: impl Iterator<Item = T>) -> Counter<String> {
        iter.map(|item| String::from(item.subreddit()))
            .collect::<Counter<_>>()
    }

    fn sort_numerically(&self) -> Vec<SubredditCount> {
        self.counts
            .most_common_tiebreaker(|lhs, rhs| Ord::cmp(&lhs.to_lowercase(), &rhs.to_lowercase()))
    }

    fn sort_lexicographically(&self) -> Vec<SubredditCount> {
        self.counts
            .keys()
            .sorted_by(|lhs, rhs| Ord::cmp(&lhs.to_lowercase(), &rhs.to_lowercase()))
            .map(|key| {
                (
                    key.to_owned(),
                    *self
                        .counts
                        .get(key)
                        .expect("somehow the key doesn't actually exist"),
                )
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reddit::Redditor;

    #[tokio::test]
    async fn it_counts_comments_by_subreddit() {
        let redditor = Redditor::test().await;
        let comments = redditor.comments();
        let counter = SubredditCounter::count(comments);

        let expected = vec![
            ("cyphersystem", 1),
            ("DiscoElysium", 31),
            ("French", 1),
            ("MicrobrandWatches", 1),
            ("movies", 2),
            ("nealstephenson", 2),
            ("rpg", 51),
            ("sanfrancisco", 5),
            ("UnresolvedMysteries", 2),
            ("wikipedia", 1),
            ("worldbuilding", 2),
            ("worldnews", 1),
        ];

        for (subreddit, expected) in expected {
            let expect_msg = format!("no '{subreddit}' in counter");
            let actual = *counter.get(subreddit).expect(&expect_msg);
            assert_eq!(
                actual, expected,
                "counter[\"{subreddit}\"] == {actual}, != {expected}"
            );
        }
    }

    #[tokio::test]
    async fn it_sorts_by_subreddit_name() {
        let redditor = Redditor::test().await;
        let expected: Vec<SubredditCount> = vec![
            ("cyphersystem", 1),
            ("DiscoElysium", 31),
            ("French", 1),
            ("MicrobrandWatches", 1),
            ("movies", 2),
            ("nealstephenson", 2),
            ("rpg", 51),
            ("sanfrancisco", 5),
            ("UnresolvedMysteries", 2),
            ("wikipedia", 1),
            ("worldbuilding", 2),
            ("worldnews", 1),
        ]
        .iter()
        .map(|(subreddit, count)| ((*subreddit).to_string(), *count as usize))
        .collect();
        let actual: Vec<SubredditCount> = SubredditCounter::from_iter(redditor.comments())
            .sort_by(&SortAlgorithm::Lexicographically);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn it_sorts_by_subreddit_count() {
        let redditor = Redditor::test().await;
        let expected: Vec<SubredditCount> = vec![
            ("rpg", 51),
            ("DiscoElysium", 31),
            ("sanfrancisco", 5),
            ("movies", 2),
            ("nealstephenson", 2),
            ("UnresolvedMysteries", 2),
            ("worldbuilding", 2),
            ("cyphersystem", 1),
            ("French", 1),
            ("MicrobrandWatches", 1),
            ("wikipedia", 1),
            ("worldnews", 1),
        ]
        .iter()
        .map(|(subreddit, count)| ((*subreddit).to_string(), *count as usize))
        .collect();
        let actual: Vec<SubredditCount> =
            SubredditCounter::from_iter(redditor.comments()).sort_by(&SortAlgorithm::Numerically);
        assert_eq!(actual, expected);
    }
}
