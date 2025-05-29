//! General-purpose counting capabilities.

use counter::Counter;

/// A thing that is attached to a subreddit.
pub trait HasSubreddit {
    /// The subreddit the thing appears in.
    fn subreddit(&self) -> &str;
}

/// Groups Reddit comments and submissions by subreddit and provides a
/// count of the number of items in each subreddit.
#[derive(Debug)]
pub struct SubredditCounter;

impl SubredditCounter {
    /// Groups and counts comments and submissions.
    ///
    /// `iter` is an iterator of `Comments` or `Submissions`, or anything
    /// that has a subreddit attached to it.
    pub fn from_iter<T: HasSubreddit>(iter: impl Iterator<Item = T>) -> Self {
        let counts = SubredditCounter::count(iter);
        SubredditCounter {}
    }

    fn count<T: HasSubreddit>(iter: impl Iterator<Item = T>) -> Counter<String> {
        iter.map(|item| String::from(item.subreddit()))
            .collect::<Counter<_>>()
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
}
