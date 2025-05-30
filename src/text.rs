//! Helpful utilities for working with text.

use htmlentity::entity::{self, ICodedDataTrait};

/// Indicates a piece of text that has HTML markup, such as HTML character
/// entities, that can be converted to "raw" text.
pub trait HtmlConvertible {
    /// Converts HTML entities into their single-character equivalents.
    ///
    /// For example, Reddit returns "&" as "&amp;amp;", ">" as "&amp;gt;",
    /// and "<" as "&amp;lt;"; this function will convert those HTML
    /// entities into single, human-readable characters.
    ///
    /// Leading and trailing whitespace will also be trimmed from the string.
    ///
    /// # Examples
    ///
    /// ```
    /// use usaidwat::text::HtmlConvertible;
    /// let raw = String::from("&lt;This &amp; That&gt;");
    /// let converted = raw.convert_html_entities();
    /// assert_eq!(converted, "<This & That>");
    /// ```
    ///
    /// ```
    /// use usaidwat::text::HtmlConvertible;
    /// let raw = String::from("  &lt;This &amp; That&gt;  ");
    /// let converted = raw.convert_html_entities();
    /// assert_eq!(converted, "<This & That>");
    /// ```
    ///
    /// ```
    /// use usaidwat::text::HtmlConvertible;
    /// let raw = String::from("A Plaintext Post");
    /// let converted = raw.convert_html_entities();
    /// assert_eq!(converted, raw);
    /// ```
    ///
    /// ```
    /// use usaidwat::text::HtmlConvertible;
    /// let raw = String::new();
    /// let converted = raw.convert_html_entities();
    /// assert_eq!(converted, raw);
    /// ```
    fn convert_html_entities(&self) -> String;
}

impl HtmlConvertible for String {
    fn convert_html_entities(&self) -> String {
        let text = self.trim();
        entity::decode(text.as_bytes())
            .to_string()
            .unwrap_or(text.to_string())
    }
}
