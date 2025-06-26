//! Helpful utilities for working with text.

use htmlentity::entity::{self, ICodedDataTrait};

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
