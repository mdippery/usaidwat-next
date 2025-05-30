//! General-purpose counting capabilities.

use counter::Counter;
use itertools::Itertools;
use std::vec::IntoIter;

/// A thing that is attached to a subreddit.
pub trait HasSubreddit {
    /// The subreddit the thing appears in.
    fn subreddit(&self) -> &str;
}

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
    /// Returns an iterator over the (subreddit name, count) pairs.
    pub fn sort_by(&self, algo: &SortAlgorithm) -> IntoIter<SubredditCount> {
        match algo {
            SortAlgorithm::Numerically => self
                .counts
                .most_common_tiebreaker(|lhs, rhs| {
                    Ord::cmp(&lhs.to_lowercase(), &rhs.to_lowercase())
                })
                .into_iter(),
            SortAlgorithm::Lexicographically => self.sort_lexicographically(),
        }
    }

    fn count<T: HasSubreddit>(iter: impl Iterator<Item = T>) -> Counter<String> {
        iter.map(|item| String::from(item.subreddit()))
            .collect::<Counter<_>>()
    }

    fn sort_lexicographically(&self) -> IntoIter<SubredditCount> {
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
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Redditor;

    #[test]
    fn it_counts_comments_by_subreddit() {
        let redditor = Redditor::test();
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

    #[test]
    fn it_sorts_by_subreddit_name() {
        let redditor = Redditor::test();
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
            .sort_by(&SortAlgorithm::Lexicographically)
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_sorts_by_subreddit_count() {
        let redditor = Redditor::test();
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
        let actual: Vec<SubredditCount> = SubredditCounter::from_iter(redditor.comments())
            .sort_by(&SortAlgorithm::Numerically)
            .collect();
        assert_eq!(actual, expected);
    }
}
