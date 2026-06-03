// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025-2026 Michael Dippery <michael@monkey-robot.com>

//! Helpful utilities for working with text.

pub use discount::text::convert_html_entities;
use regex::Regex;

/// A string-like structure with parts that can be matched against a
/// regex and replaced.
pub trait RegexReplaceable {
    /// Search the target for `needle` and replace matches with `repl`,
    /// returning a new version of the target.
    fn replace_all<S: AsRef<str>>(&self, needle: S, repl: S) -> Self;
}

impl RegexReplaceable for String {
    /// Search the string for `needle` and replace matches with `repl`,
    /// returning a new string with the replaced text.
    ///
    /// # Examples
    ///
    /// `replace_all()` matches using a regex and returns a string with
    /// replacements:
    ///
    /// ```
    /// use usaidwat::text::RegexReplaceable;
    /// let before = String::from("I HAAAAAAAATE REGEXES!");
    /// let after = before.replace_all("(?i)ha+te", "LOVE");
    /// assert_eq!(after, "I LOVE REGEXES!");
    /// ```
    ///
    /// It works even if `needle` is not a valid regex:
    ///
    /// ```
    /// use usaidwat::text::RegexReplaceable;
    /// let before = String::from("I HAAAAAAAATE REGEXES!");
    /// let after = before.replace_all("(?i)ha{?}te", "LOVE");
    /// assert_eq!(after, before);
    /// ```
    ///
    /// It can replace fixed strings, too:
    ///
    /// ```
    /// use usaidwat::text::RegexReplaceable;
    /// let before = String::from("I HAAAAAAAATE REGEXES!");
    /// let after = before.replace_all("HAAAAAAAATE", "LOVE");
    /// assert_eq!(after, "I LOVE REGEXES!");
    /// ```
    fn replace_all<S: AsRef<str>>(&self, needle: S, repl: S) -> Self {
        match Regex::new(needle.as_ref()) {
            Ok(re) => re.replace_all(self, repl.as_ref()).to_string(),
            Err(_) => self.replace(needle.as_ref(), repl.as_ref()),
        }
    }
}
