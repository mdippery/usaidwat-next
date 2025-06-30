//! General-purpose search utilities.

use crate::thing::HasSubreddit;
use regex::Regex;
use std::collections::HashSet;

/// A thing that can be searched.
pub trait Searchable {
    /// The haystack that can be searched for a needle.
    fn search_text(&self) -> String;

    /// True if the search pattern can be found in the [`Searchable::search_text()`].
    ///
    /// The search is case-insensitive.
    ///
    /// `pattern` can be a regular expression.
    fn matches(&self, pattern: &str) -> bool {
        match Regex::new(&format!("(?i){pattern}")) {
            Ok(re) => re.is_match(&self.search_text()),
            Err(_) => self.search_text().find(pattern).is_some(),
        }
    }
}

/// A container for filtering Reddit things.
pub struct RedditFilter<I>
where
    I: Iterator,
    I::Item: Searchable + HasSubreddit,
{
    things: I,
}

impl<I> RedditFilter<I>
where
    I: Iterator,
    I::Item: Searchable + HasSubreddit,
{
    /// Creates a new `RedditFilter` that wraps the given iterator.
    pub fn new(things: I) -> Self {
        Self { things }
    }

    /// Returns all items with searchable text that matches the given needle.
    ///
    /// If `needle` is `None`, all items are returned.
    pub fn grep(self, needle: &Option<String>) -> RedditFilter<impl Iterator<Item = I::Item>> {
        let things = match needle {
            None => self.things.collect::<Vec<_>>(),
            Some(needle) => self
                .things
                .filter(|thing| thing.matches(needle))
                .collect::<Vec<_>>(),
        };
        let things = things.into_iter();
        RedditFilter { things }
    }

    /// Returns all items with subreddits matching the given set of subreddits.
    ///
    /// If `subreddits` is empty, all items are returned.
    pub fn filter(self, subreddits: &StringSet) -> RedditFilter<impl Iterator<Item = I::Item>> {
        let things: Vec<I::Item> = if subreddits.is_empty() {
            self.things.collect()
        } else {
            self.things
                .filter(|item| subreddits.contains(item.subreddit()))
                .collect()
        };
        let things = things.into_iter();

        RedditFilter { things }
    }

    /// Collects all items into a vector.
    pub fn collect(self) -> Vec<I::Item> {
        self.things.collect()
    }
}

/// A set of strings.
///
/// This set can function like a normal set, but it can also store _negated_
/// strings, meaning that [`StringSet::contains()`] will return `true` if the test
/// string is _not_ contained in the set.
#[derive(Debug)]
pub struct StringSet {
    kind: StringSetKind,
}

impl StringSet {
    /// Converts a list of strings into a `StringSet`.
    ///
    /// Strings can be negated by prefixing them with a `-`; for example,
    /// `-string` will match any needles that are _not_ `"string"`.
    ///
    /// All strings passed in must either be negated or not negated.
    /// If strings are mixed, `None` is returned.
    pub fn from(strings: &Vec<&str>) -> Option<Self> {
        let validator = StringSetValidator::from(strings);

        if !validator.is_valid() {
            None
        } else {
            let all_positive = validator.all_positive();
            let set = validator.into_set();
            let kind = if all_positive {
                StringSetKind::Positive(set)
            } else {
                StringSetKind::Negative(set)
            };
            Some(Self { kind })
        }
    }

    pub fn contains(&self, needle: &str) -> bool {
        match &self.kind {
            StringSetKind::Negative(set) => !set.contains(&needle.to_lowercase()),
            StringSetKind::Positive(set) => set.contains(&needle.to_lowercase()),
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.kind {
            StringSetKind::Negative(set) | StringSetKind::Positive(set) => set.is_empty(),
        }
    }

    pub fn is_negated(&self) -> bool {
        self.kind.is_negated()
    }
}

/// Indicates whether a set is negated or not.
#[derive(Debug)]
enum StringSetKind {
    Positive(HashSet<String>),
    Negative(HashSet<String>),
}

impl StringSetKind {
    pub fn is_negated(&self) -> bool {
        matches!(self, StringSetKind::Negative(_))
    }
}

/// Processes a vector of strings into a flattened vector and checks
/// it for validity.
struct StringSetValidator {
    strings: Vec<String>,
}

impl StringSetValidator {
    /// Flattens a vector of strings and returns a new validator.
    ///
    /// Some or all of the elements of `strings` may be comma-separated;
    /// `new()` will flatten them into a single vector.
    pub fn from(strings: &Vec<&str>) -> Self {
        let strings = strings
            .into_iter()
            .flat_map(|s| {
                s.replace(" ", "")
                    .split(',')
                    .map(str::to_owned)
                    .collect::<Vec<String>>()
            })
            .collect();
        Self { strings }
    }

    /// Returns true if either:
    ///
    /// - All strings are positive (not prefixed with `-`)
    /// - All strings are negative (prefixed with `-`)
    pub fn is_valid(&self) -> bool {
        self.all_positive() || self.all_negative()
    }

    /// True if every string is prefixed with `-`.
    pub fn all_negative(&self) -> bool {
        self.strings.iter().all(|s| s.starts_with('-'))
    }

    /// True if none of the strings are prefixed with `-`.
    pub fn all_positive(&self) -> bool {
        self.strings.iter().all(|s| !s.starts_with('-'))
    }

    /// Converts the internally stored strings into a hash set, consuming
    /// the validator in the process.
    pub fn into_set(self) -> HashSet<String> {
        HashSet::from_iter(
            self.strings
                .into_iter()
                .map(|s| s.trim_start_matches('-').to_lowercase()),
        )
    }
}

#[cfg(test)]
mod tests {
    mod searchable {
        use super::super::*;

        #[derive(Default, Debug)]
        struct TestSearchable;

        impl Searchable for TestSearchable {
            fn search_text(&self) -> String {
                String::from("peter piper picked a peck of pickled peppers")
            }
        }

        #[test]
        fn it_returns_true_if_there_is_a_match() {
            let t = TestSearchable::default();
            assert!(t.matches("peppers"));
        }

        #[test]
        fn it_returns_true_if_there_are_multiple_matches() {
            let t = TestSearchable::default();
            assert!(t.matches("pick"));
        }

        #[test]
        fn it_matches_substrings() {
            let t = TestSearchable::default();
            assert!(t.matches("pip"));
        }

        #[test]
        fn it_matches_needles_with_spaces() {
            let t = TestSearchable::default();
            assert!(t.matches("picked a peck"));
        }

        #[test]
        fn it_returns_false_if_there_are_no_matches() {
            let t = TestSearchable::default();
            assert!(!t.matches("usaidwait"));
        }

        #[test]
        fn it_matches_regexes() {
            let t = TestSearchable::default();
            assert!(t.matches("pep{2,}ers"));
        }

        #[test]
        fn it_matches_regexes_case_insensitively() {
            let t = TestSearchable::default();
            assert!(t.matches("Piper"));
        }

        #[test]
        fn it_treats_invalid_regexes_as_a_fixed_string() {
            let t = TestSearchable::default();
            assert!(!t.matches("pic{?}kl**ed"));
        }
    }

    mod reddit_filter {
        use super::super::*;

        #[derive(Debug)]
        struct TestSearchable {
            string: String,
            subreddit: String,
        }

        impl TestSearchable {
            pub fn new(string: &str, subreddit: &str) -> Self {
                TestSearchable {
                    string: String::from(string),
                    subreddit: String::from(subreddit),
                }
            }
        }

        impl Searchable for TestSearchable {
            fn search_text(&self) -> String {
                self.string.clone()
            }
        }

        impl HasSubreddit for TestSearchable {
            // Doesn't matter, not tested but required to meet trait constraints
            fn subreddit(&self) -> &str {
                self.subreddit.as_str()
            }
        }

        fn load_test() -> Vec<TestSearchable> {
            let strings = vec![
                (
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
                    "subreddit",
                ),
                ("In sodales urna et libero commodo varius.", "subreddit"),
                ("Morbi vitae varius orci.", "other"),
                ("Sed luctus turpis ac fringilla maximus.", "another"),
                (
                    "In libero nisl, condimentum in gravida eget, bibendum id lectus.",
                    "words",
                ),
                ("Nunc sit amet odio dolor.", "poetry"),
                ("Nunc quis urna vel sem iaculis dapibus.", "subreddit"),
                (
                    "Donec justo metus, vulputate a purus at, tincidunt porttitor erat.",
                    "blah",
                ),
                (
                    "Quisque in metus molestie, dictum metus nec, malesuada tortor.",
                    "foo",
                ),
                ("Nam sed turpis eu tortor semper rhoncus.", "bar"),
                (
                    "Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas.",
                    "baz",
                ),
            ];
            strings
                .iter()
                .map(|(s, sr)| TestSearchable::new(s, sr))
                .collect()
        }

        #[test]
        fn it_finds_items_matching_a_string() {
            let texts = load_test();
            let grep = Some(String::from("\\bnunc\\b"));
            let matches = RedditFilter::new(texts.into_iter()).grep(&grep);
            assert_eq!(matches.collect().len(), 2);
        }

        #[test]
        fn it_returns_everything_if_there_is_no_needle() {
            let texts = load_test();
            let n = texts.len();
            let grep = None;
            let matches = RedditFilter::new(texts.into_iter()).grep(&grep);
            assert_eq!(matches.collect().len(), n);
        }

        #[test]
        fn it_returns_nothing_if_there_are_no_matches() {
            let texts = load_test();
            let grep = Some(String::from("some text"));
            let matches = RedditFilter::new(texts.into_iter()).grep(&grep);
            assert_eq!(matches.collect().len(), 0);
        }

        #[test]
        fn it_returns_everything_if_subreddit_filter_is_empty() {
            let texts = load_test();
            let n = texts.len();
            let filter =
                StringSet::from(&vec![]).expect("should create string set from empty vector");
            let filtered = RedditFilter::new(texts.into_iter()).filter(&filter);
            assert_eq!(filtered.collect().len(), n);
        }

        #[test]
        fn it_returns_a_subset_if_subreddit_filter_is_positive() {
            let texts = load_test();
            let n = texts
                .iter()
                .filter(|t| t.subreddit() == "subreddit")
                .count();
            let filter = StringSet::from(&vec!["subreddit", "doesnotexist"])
                .expect("should create string set from empty vector");
            let filtered = RedditFilter::new(texts.into_iter()).filter(&filter);
            assert_eq!(filtered.collect().len(), n);
        }

        #[test]
        fn it_returns_everything_if_subreddit_filter_is_negative() {
            let texts = load_test();
            let n = texts.len();
            let x = texts
                .iter()
                .filter(|t| t.subreddit() == "subreddit")
                .count();
            let filter = StringSet::from(&vec!["-subreddit", "-doesnotexist"])
                .expect("should create string set from empty vector");
            let filtered = RedditFilter::new(texts.into_iter()).filter(&filter);
            assert_eq!(filtered.collect().len(), n - x);
        }
    }

    mod string_set {
        use super::super::*;

        #[test]
        fn it_accepts_all_positive_strings() {
            let strings = vec!["alpha", "beta", "charlie", "delta"];
            let set = StringSet::from(&strings);
            assert!(set.is_some());
        }

        #[test]
        fn it_accepts_all_positive_comma_separated_strings() {
            let strings = vec!["alpha,beta,charlie,delta"];
            let set = StringSet::from(&strings);
            assert!(set.is_some());
        }

        #[test]
        fn it_accepts_all_positive_nested_strings() {
            let strings = vec!["alpha,beta", "charlie", "delta,echo,foxtrot", "golf"];
            let set = StringSet::from(&strings);
            assert!(set.is_some());
        }

        #[test]
        fn it_accepts_all_negative_strings() {
            let strings = vec!["-alpha", "-beta", "-charlie", "-delta"];
            let set = StringSet::from(&strings);
            assert!(set.is_some());
        }

        #[test]
        fn it_accepts_all_negative_comma_separated_strings() {
            let strings = vec!["-alpha,-beta,-charlie,-delta"];
            let set = StringSet::from(&strings);
            assert!(set.is_some());
        }

        #[test]
        fn it_accepts_all_negative_nested_strings() {
            let strings = vec!["-alpha,-beta", "-charlie", "-delta,-echo,-foxtrot", "-golf"];
            let set = StringSet::from(&strings);
            assert!(set.is_some());
        }

        #[test]
        fn it_rejects_mixed_strings() {
            let strings = vec!["alpha", "-beta", "-charlie", "delta"];
            let set = StringSet::from(&strings);
            assert!(set.is_none());
        }

        #[test]
        fn it_rejects_mixed_comma_separated_strings() {
            let strings = vec!["-alpha,beta,-charlie,delta"];
            let set = StringSet::from(&strings);
            assert!(set.is_none());
        }

        #[test]
        fn it_rejects_mixed_nested_strings() {
            let strings = vec!["-alpha,-beta", "charlie", "delta,-echo,foxtrot", "-golf"];
            let set = StringSet::from(&strings);
            assert!(set.is_none());
        }

        #[test]
        fn it_returns_true_if_it_contains_negated_strings() {
            let strings = vec!["-alpha", "-beta", "-charlie", "-delta"];
            let set =
                StringSet::from(&strings).expect(&format!("should build set from {strings:?}"));
            assert!(set.is_negated());
        }

        #[test]
        fn it_returns_false_if_it_contains_positive_strings() {
            let strings = vec!["alpha", "beta", "charlie", "delta"];
            let set =
                StringSet::from(&strings).expect(&format!("should build set from {strings:?}"));
            assert!(!set.is_negated());
        }

        #[test]
        fn it_is_empty_if_it_contains_no_items() {
            let set = StringSet::from(&vec![]).expect("should build set from empty vector");
            assert!(set.is_empty());
        }

        #[test]
        fn it_is_not_empty_if_it_contains_items() {
            let strings = vec!["alpha", "beta", "charlie", "delta"];
            let set =
                StringSet::from(&strings).expect(&format!("should build set from {strings:?}"));
            assert!(!set.is_empty());
        }

        mod when_positive {
            use super::super::super::*;

            #[test]
            fn it_accepts_a_string_in_the_set() {
                let strings = vec!["alpha,beta", "charlie", "delta,echo,foxtrot", "golf"];
                let set =
                    StringSet::from(&strings).expect(&format!("should build set from {strings:?}"));
                assert!(set.contains("echo"));
            }

            #[test]
            fn it_accepts_a_string_in_the_set_case_insensitively() {
                let strings = vec!["Alpha,Beta", "Charlie", "Delta,Echo,Foxtrot", "golf"];
                let set =
                    StringSet::from(&strings).expect(&format!("should build set from {strings:?}"));
                assert!(
                    set.contains("echo"),
                    "'echo' should be in {set:?}, but is not"
                );
            }

            #[test]
            fn it_rejects_a_string_not_in_the_set() {
                let strings = vec!["alpha,beta", "charlie", "delta,echo,foxtrot", "golf"];
                let set =
                    StringSet::from(&strings).expect(&format!("should build set from {strings:?}"));
                assert!(!set.contains("romeo"));
            }
        }

        mod when_negative {
            use super::super::super::*;

            #[test]
            fn it_accepts_a_string_not_in_the_set() {
                let strings = vec!["-alpha,-beta", "-charlie", "-delta,-echo,-foxtrot", "-golf"];
                let set =
                    StringSet::from(&strings).expect(&format!("should build set from {strings:?}"));
                assert!(set.contains("romeo"));
            }

            #[test]
            fn it_rejects_a_string_in_the_set() {
                let strings = vec!["-alpha,-beta", "-charlie", "-delta,-echo,-foxtrot", "-golf"];
                let set =
                    StringSet::from(&strings).expect(&format!("should build set from {strings:?}"));
                assert!(
                    !set.contains("echo"),
                    "'echo' should not be in {set:?}, but is"
                );
            }

            #[test]
            fn it_rejects_a_string_in_the_set_case_insensitively() {
                let strings = vec!["-Alpha,-Beta", "-Charlie", "-Delta,-Echo,-Foxtrot", "-golf"];
                let set =
                    StringSet::from(&strings).expect(&format!("should build set from {strings:?}"));
                assert!(
                    !set.contains("echo"),
                    "'echo' should not be in {set:?}, but is"
                );
            }
        }
    }

    mod string_set_validator {
        use super::super::*;

        #[test]
        fn it_accepts_all_positive_strings() {
            let strings = vec!["alpha", "beta", "charlie", "delta"];
            let validator = StringSetValidator::from(&strings);
            assert!(validator.is_valid());
            assert!(validator.all_positive());
            assert!(!validator.all_negative());
        }

        #[test]
        fn it_accepts_all_positive_comma_separated_strings() {
            let strings = vec!["alpha,beta,charlie,delta"];
            let validator = StringSetValidator::from(&strings);
            assert!(validator.is_valid());
            assert!(validator.all_positive());
            assert!(!validator.all_negative());
        }

        #[test]
        fn it_accepts_all_positive_nested_strings() {
            let strings = vec!["alpha,beta", "charlie", "delta,echo,foxtrot", "golf"];
            let validator = StringSetValidator::from(&strings);
            assert!(validator.is_valid());
            assert!(validator.all_positive());
            assert!(!validator.all_negative());
        }

        #[test]
        fn it_accepts_all_negative_strings() {
            let strings = vec!["-alpha", "-beta", "-charlie", "-delta"];
            let validator = StringSetValidator::from(&strings);
            assert!(validator.is_valid());
            assert!(!validator.all_positive());
            assert!(validator.all_negative());
        }

        #[test]
        fn it_accepts_all_negative_comma_separated_strings() {
            let strings = vec!["-alpha,-beta,-charlie,-delta"];
            let validator = StringSetValidator::from(&strings);
            assert!(validator.is_valid());
            assert!(!validator.all_positive());
            assert!(validator.all_negative());
        }

        #[test]
        fn it_accepts_all_negative_nested_strings() {
            let strings = vec!["-alpha,-beta", "-charlie", "-delta,-echo,-foxtrot", "-golf"];
            let validator = StringSetValidator::from(&strings);
            assert!(validator.is_valid());
            assert!(!validator.all_positive());
            assert!(validator.all_negative());
        }

        #[test]
        fn it_rejects_mixed_strings() {
            let strings = vec!["alpha", "-beta", "-charlie", "delta"];
            let validator = StringSetValidator::from(&strings);
            assert!(!validator.is_valid());
            assert!(!validator.all_positive());
            assert!(!validator.all_negative());
        }

        #[test]
        fn it_rejects_mixed_comma_separated_strings() {
            let strings = vec!["-alpha,beta,-charlie,delta"];
            let validator = StringSetValidator::from(&strings);
            assert!(!validator.is_valid());
            assert!(!validator.all_positive());
            assert!(!validator.all_negative());
        }

        #[test]
        fn it_rejects_mixed_nested_strings() {
            let strings = vec!["-alpha,-beta", "charlie", "delta,-echo,foxtrot", "-golf"];
            let validator = StringSetValidator::from(&strings);
            assert!(!validator.is_valid());
            assert!(!validator.all_positive());
            assert!(!validator.all_negative());
        }
    }
}
