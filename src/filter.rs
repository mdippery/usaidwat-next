//! General-purpose search utilities.

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
            StringSetKind::Negative(set) => !set.contains(needle),
            StringSetKind::Positive(set) => set.contains(needle),
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
                .map(|s| s.trim_start_matches('-').to_owned()),
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
