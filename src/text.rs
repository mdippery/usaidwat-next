// usaidwat
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Helpful utilities for working with text.

use htmlentity::entity::{self, ICodedDataTrait};
use regex::Regex;

/// Converts HTML entities into their single-character equivalents.
///
/// For example, Reddit returns "&" as "&amp;", ">" as "&gt;",
/// and "<" as "&lt;"; this function will convert those HTML
/// entities into single, human-readable characters.
///
/// Leading and trailing whitespace will also be trimmed from the string.
///
/// # Examples
///
/// `convert_html_entities` will convert HTML entities into their respective
/// single-character equivalents:
///
/// ```
/// use usaidwat::text::convert_html_entities;
/// let raw = "&lt;This &amp; That&gt;";
/// let converted = convert_html_entities(raw);
/// assert_eq!(converted, "<This & That>");
/// ```
///
/// It will also remove leading and trailing whitespace:
///
/// ```
/// use usaidwat::text::convert_html_entities;
/// let raw = "  &lt;This &amp; That&gt;  ";
/// let converted = convert_html_entities(raw);
/// assert_eq!(converted, "<This & That>");
/// ```
///
/// It won't change characters that are not HTML entities:
///
/// ```
/// use usaidwat::text::convert_html_entities;
/// let raw = "A Plaintext Post";
/// let converted = convert_html_entities(raw);
/// assert_eq!(converted, raw);
/// ```
///
/// It will even handle empty strings:
///
/// ```
/// use usaidwat::text::convert_html_entities;
/// let raw = "";
/// let converted = convert_html_entities(raw);
/// assert_eq!(converted, raw);
/// ```
pub fn convert_html_entities(text: impl AsRef<str>) -> String {
    let text = text.as_ref().trim();
    entity::decode(text.as_bytes())
        .to_string()
        .unwrap_or(text.to_string())
}

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
