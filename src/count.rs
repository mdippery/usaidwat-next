// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025-2026 Michael Dippery <michael@monkey-robot.com>

//! General-purpose counting capabilities.

use crate::reddit::thing::HasSubreddit;
use counter::Counter;
use itertools::Itertools;
use std::cmp::Ordering;

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
///
/// Normally you create a `SubredditCounter` by calling `collect()` on
/// an iterator of `Comments` or `Submissions`, as described in
/// [`SubredditCounter::from_iter()`].
#[derive(Debug)]
pub struct SubredditCounter {
    counts: Counter<String>,
}

impl<A: HasSubreddit> FromIterator<A> for SubredditCounter {
    /// Groups and counts comments and submissions.
    ///
    /// `iter` is an iterator of `Comments` or `Submissions`, or anything
    /// that has a subreddit attached to it.
    ///
    /// You can easily create a `SubredditCounter` from these iterators
    /// using `collect()`:
    ///
    /// ```
    /// # use usaidwat::count::{SortAlgorithm, SubredditCount, SubredditCounter};
    /// # use usaidwat::reddit::thing::Comment;
    /// # fn get_comments_somehow() -> Vec<Comment> {
    /// #     vec![]
    /// # }
    /// let comments: Vec<Comment> = get_comments_somehow();
    /// let counter: Vec<SubredditCount> = comments
    ///     .into_iter()
    ///     .collect::<SubredditCounter>()
    ///     .sort_by(&SortAlgorithm::Numerically);
    /// ```
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let counts = SubredditCounter::count(iter);
        SubredditCounter { counts }
    }
}

impl SubredditCounter {
    fn count<T: HasSubreddit>(iter: impl IntoIterator<Item = T>) -> Counter<String> {
        iter.into_iter()
            .map(|item| String::from(item.subreddit()))
            .collect::<Counter<_>>()
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

    fn sort_numerically(&self) -> Vec<SubredditCount> {
        self.counts
            .most_common_tiebreaker(|lhs, rhs| sort_string(lhs, rhs))
    }

    fn sort_lexicographically(&self) -> Vec<SubredditCount> {
        self.counts
            .keys()
            .sorted_by(|lhs, rhs| sort_string(lhs, rhs))
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

#[inline]
fn sort_string(lhs: &str, rhs: &str) -> Ordering {
    Ord::cmp(&lhs.to_lowercase(), &rhs.to_lowercase())
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
        let actual: Vec<SubredditCount> = redditor
            .comments()
            .collect::<SubredditCounter>()
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
        let actual: Vec<SubredditCount> = redditor
            .comments()
            .collect::<SubredditCounter>()
            .sort_by(&SortAlgorithm::Numerically);
        assert_eq!(actual, expected);
    }
}
