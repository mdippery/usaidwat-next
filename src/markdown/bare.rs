// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! A bare Markdown parser.
//!
//! This parser strips all formatting from Markdown source code, returning
//! only the text of the markup. It completely removes link URLs, quoted text,
//! headers, image links, and other text accoutrement.
//!
//! This parser is primarily useful for returning stripped text suitable for
//! passing to an LLM. Thus it is not a completely accurate summary, as
//! some text is completely removed, but it should be "good enough" for an
//! LLM.

use crate::markdown::{TextAppendable, Visitable, Visitor};
use crate::text;
use log::trace;
use markdown::mdast::{Code, InlineCode, Node, Text};
use markdown::{Constructs, ParseOptions};

/// Converts Markdown markup into a string that has been stripped of nearly
/// all markup, including quoted text, headers, image links, and link URLs.
///
/// The result is suitable for passing to an LLM for further processing.
pub fn parse(markup: impl AsRef<str>) -> String {
    MarkdownParser::new().parse(text::convert_html_entities(markup))
}

/// A reusable Markdown parser for stripping formatting and producing
/// bare output suitable for passing to an LLM.
#[derive(Debug)]
struct MarkdownParser;

impl MarkdownParser {
    /// Creates a new Markdown parser.
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, markup: impl AsRef<str>) -> String {
        let tree = markdown::to_mdast(markup.as_ref(), &self.parse_options()).unwrap();
        let mut visitor = MarkdownVisitor::new();
        tree.accept(&mut visitor);
        visitor.text()
    }

    fn parse_options(&self) -> ParseOptions {
        ParseOptions {
            constructs: Constructs {
                gfm_strikethrough: true,
                ..Constructs::default()
            },
            ..ParseOptions::default()
        }
    }
}

#[derive(Debug)]
struct MarkdownVisitor {
    text: String,
}

impl MarkdownVisitor {
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    fn ignore(&self, node: &Node) {
        trace!("completely ignoring a node and its children: {node:?}");
    }

    fn visit_paragraph(&mut self, node: &Node) {
        self.push_text("\n");
        node.accept_children(self)
    }

    fn visit_text(&mut self, text: &Text) {
        self.push_text(&text.value.replace(">!", "").replace("!<", ""))
    }

    fn visit_break(&mut self) {
        self.push_text(" ");
    }

    fn visit_any_emphasis(&mut self, node: &Node) {
        trace!("emphasis: {node:?}");
        node.accept_children(self);
    }

    fn visit_link(&mut self, node: &Node) {
        trace!("link: {node:#?}");
        node.accept_children(self);
    }

    fn visit_inline_code(&mut self, code: &str, node: &Node) {
        self.push_text(&format!("`{code}`"));
        node.accept_children(self);
    }

    fn visit_code(&mut self, code: &str) {
        self.push_text("\n\n");
        self.push_text(code);
        self.push_text("\n");
    }
}

impl Visitor for MarkdownVisitor {
    fn text(&self) -> String {
        String::from(self.text.trim_matches('\n'))
    }

    fn visit(&mut self, node: &Node) {
        match node {
            Node::Root(_) => self.swallow(node),
            Node::Paragraph(_) => self.visit_paragraph(node),
            Node::Text(text) => self.visit_text(text),
            Node::Break(_) => self.visit_break(),
            Node::Emphasis(_) | Node::Strong(_) => self.visit_any_emphasis(node),
            Node::Heading(_) => self.ignore(node),
            Node::Link(_) => self.visit_link(node),
            Node::Image(_) => self.ignore(node),
            Node::InlineCode(InlineCode { value, .. }) => self.visit_inline_code(value, node),
            Node::List(_) => self.swallow(node),
            Node::Code(Code { value, .. }) => self.visit_code(value),
            Node::Blockquote(_) => self.ignore(node),
            _ => self.swallow(node),
        }
    }
}

impl TextAppendable for MarkdownVisitor {
    fn push_text(&mut self, text: &str) {
        trace!("appending text to {:?}: {text:?}", self.text);
        self.text += text;
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::markdown::test_utils::load_markdown;
    use crate::test_utils::do_logging;
    use crate::{header_tests, parse_assert_eq};
    use pretty_assertions::assert_eq;

    fn load_output(file: &str) -> String {
        let bare_file = format!("{file}.bare");
        crate::markdown::test_utils::load_output(&bare_file)
    }

    fn load_test(file: &str) -> (String, String) {
        (load_markdown(file), load_output(&file))
    }

    #[test]
    fn it_does_not_touch_normal_text() {
        let text = "Lorem ipsum dolor sit amet";
        parse_assert_eq!(text, text);
    }

    #[test]
    fn it_separates_paragraphs_by_a_single_newline() {
        let (text, expected) = load_test("paragraphs");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_spoiler_markers() {
        let text = "This is >!Reddit spoiler text!< and is hidden.";
        let expected = "This is Reddit spoiler text and is hidden.";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_emphasis_asterisks() {
        let text = "this text is *emphasized*";
        let expected = "this text is emphasized";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_emphasis_underscores() {
        let text = "this text is _emphasized_";
        let expected = "this text is emphasized";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_emphasis_double_underscores() {
        let text = "this text is __really emphasized__";
        let expected = "this text is really emphasized";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_emphasis_triple_asterisks() {
        let text = "this text is ***really really emphasized***";
        let expected = "this text is really really emphasized";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_emphasis_underscores_and_double_asterisks() {
        let text = "this text is **_really really emphasized_**";
        let expected = "this text is really really emphasized";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_emphasis_underscores_and_double_asterisks_reversed() {
        let text = "this text is _**really really emphasized**_";
        let expected = "this text is really really emphasized";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_quadruple_underscores() {
        let text = "this text is ____really really emphasized____";
        let expected = "this text is really really emphasized";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_quadruple_asterisks() {
        let text = "this text is ****really really emphasized****";
        let expected = "this text is really really emphasized";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_nested_emphasis_beginning_with_an_underscore() {
        let text = "this _text **is really emphasized** text_";
        let expected = "this text is really emphasized text";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_nested_emphasis_beginning_with_asterisks() {
        let text = "this **text _is really emphasized_ text**";
        let expected = "this text is really emphasized text";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_nested_emphasis_beginning_with_a_single_asterisk() {
        let text = "this text is really emphasized text";
        let expected = "this text is really emphasized text";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_nested_emphasis_beginning_with_double_asterisks() {
        let text = "this **text *is really emphasized* text**";
        let expected = "this text is really emphasized text";
        parse_assert_eq!(text, expected);
    }

    header_tests!("");

    #[test]
    fn it_removes_link_urls() {
        do_logging();
        let text = "[usaidwat source](https://github.com/mdippery/usaidwat-next)";
        let expected = "usaidwat source";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_retains_inline_links() {
        let text = "<https://github.com/mdippery/usaidwat-next>";
        let expected = "https://github.com/mdippery/usaidwat-next";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_emphasis_in_links() {
        let text = "[_usaidwat source_](https://github.com/mdippery/usaidwat-next)";
        let expected = "usaidwat source";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_strong_emphasis_in_links() {
        let text = "[**usaidwat source**](https://github.com/mdippery/usaidwat-next)";
        let expected = "usaidwat source";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_retains_inline_code_in_links() {
        let text = "[`usaidwat source`](https://github.com/mdippery/usaidwat-next)";
        let expected = "`usaidwat source`";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_images() {
        let text = "![A beautiful picture](https://www.example.com/a-beautiful-picture.jpg)";
        let expected = "";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_does_not_touch_inline_code() {
        let text = "Here is a Rust struct: `struct SomeStruct;`";
        parse_assert_eq!(text, text);
    }

    #[test]
    fn it_removes_markers_around_strikethrough_text_with_single_tildes() {
        let text = "this text is ~gone~";
        let expected = "this text is gone";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_markers_around_strikethrough_text_with_double_tildes() {
        do_logging();
        let text = "this text is ~~gone~~";
        let expected = "this text is gone";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_prepends_a_caret_to_superscripts() {
        let text = "x^y";
        parse_assert_eq!(text, text);
    }

    #[test]
    fn it_wraps_long_superscript_text_in_parentheses() {
        let text = "x^(y + z)";
        parse_assert_eq!(text, text);
    }

    #[test]
    fn it_removes_indentation_from_unordered_lists() {
        let (text, expected) = load_test("unordered_list");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_retains_unordered_lists_as_is() {
        let text = load_markdown("unordered_list_unindented");
        let expected = load_output("unordered_list");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_handles_markdown_in_unordered_list_items() {
        let (text, expected) = load_test("unordered_list_embedded");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_parses_nested_unordered_lists() {
        let (text, expected) = load_test("nested_unordered_list");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_indentation_from_ordered_lists() {
        let (text, expected) = load_test("ordered_list");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_retains_ordered_lists_as_is() {
        let text = load_markdown("ordered_list_unindented");
        let expected = load_output("ordered_list");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_handles_markdown_in_ordered_list_items() {
        let (text, expected) = load_test("ordered_list_embedded");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_parses_nested_ordered_lists() {
        let (text, expected) = load_test("nested_ordered_list");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_doesnt_worry_about_nonsequential_ordering_markers() {
        let text = load_markdown("ordered_list_nonsequential");
        let expected = load_output("ordered_list");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_retains_code_blocks() {
        let (text, expected) = load_test("code");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_removes_fenced_code_block_markers() {
        let text = load_markdown("code_fenced");
        let expected = load_output("code");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_block_quotes() {
        let (text, expected) = load_test("blockquote");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_adjacent_block_quotes() {
        let (text, expected) = load_test("blockquote_single");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_inline_html() {
        let text = "this HTML has <span>inline text</span>";
        let expected = "this HTML has inline text";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_processes_inline_html_containing_markdown() {
        let text = "this HTML has <span>_emphasized_ inline text</span>";
        let expected = "this HTML has emphasized inline text";
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_block_html() {
        let (text, expected) = load_test("block_html");
        parse_assert_eq!(text, expected);
    }

    #[test]
    fn it_removes_horizontal_rules() {
        let text = "---\n";
        parse_assert_eq!(text, "");
    }

    #[test]
    fn it_processes_html_entities() {
        let text = "&lt;this &amp; &quot;that&quot;&gt;";
        parse_assert_eq!(text, "<this & \"that\">");
    }

    #[test]
    #[ignore = "double-check output but this test is not vital"]
    fn it_parses_complex_markdown() {
        let (text, expected) = load_test("markdown");
        parse_assert_eq!(text, expected);
    }
}
