//! General-purpose search utilities.

/// A thing that can be searched.
pub trait Searchable {
    /// The haystack that can be searched for a needle.
    fn search_text(&self) -> &str;

    /// True if the search pattern can be found in the [`Searchable::search_text()`].
    ///
    /// The search is case-insensitive.
    ///
    /// `pattern` is a fixed string; regular expression matches are not
    /// yet supported.
    fn matches(&self, pattern: &str) -> bool {
        // TODO: pattern should be a case-insensitive regex
        //       (or rather, that's how it is in the current Ruby tool,
        //       but I'm actually not convinced that we should search
        //       case-insensitively with a regex)
        self.search_text()
            .to_lowercase()
            .matches(&pattern.to_lowercase())
            .count()
            > 0
    }
}

mod tests {
    use super::*;

    #[derive(Default, Debug)]
    struct TestSearchable;

    impl Searchable for TestSearchable {
        fn search_text(&self) -> &str {
            "peter piper picked a peck of pickled peppers"
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
}
