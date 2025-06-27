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
pub fn convert_html_entities(text: &str) -> String {
    let text = text.trim();
    entity::decode(text.as_bytes())
        .to_string()
        .unwrap_or(text.to_string())
}

/// A string-like structure with parts that can be matched against a
/// regex and replaced.
pub trait RegexReplaceable {
    /// Search the target for `needle` and replace matches with `repl`,
    /// returning a new version of the target.
    fn replace_all(&self, needle: &str, repl: &str) -> Self;
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
    fn replace_all(&self, needle: &str, repl: &str) -> Self {
        match Regex::new(needle) {
            Ok(re) => re.replace_all(self, repl).to_string(),
            Err(_) => self.replace(needle, repl),
        }
    }
}
