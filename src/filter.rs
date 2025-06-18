//! General-purpose search utilities.

use regex::Regex;

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
}
