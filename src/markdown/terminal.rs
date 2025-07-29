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

//! Markdown parsing engine for terminals.
//!
//! This is my Markdown parser. There are many like it, but this one is mine.
//!
//! In fact, there aren't many like it. This parser is not a general Markdown
//! parser; rather, it is specifically designed to format Markdown source
//! code for display on a terminal. Thus the parser not only styles text
//! according to some basic rules, but it also wraps text to the width of
//! the terminal, as specified when calling [`parse()`].
//!
//! It is also designed to format text to the most basic of terminals, so it
//! does not fancy styling beyond underlined and bold ("bright") text.
//!
//! # See also
//!
//! - [ANSI Escape Sequences](https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797>), for
//!   the full rundown of ANSI terminal escape codes.

use crate::markdown::{TextAppendable, Visitable, Visitor};
use crate::text;
use itertools::Itertools;
use log::trace;
use markdown::mdast::{Code, Html, Image, InlineCode, Link, List, Node, Text};
use markdown::{Constructs, ParseOptions};
use textwrap::Options;

/// Converts Markdown markup into a formatted string suitable for output on
/// a terminal.
///
/// `markup` is the raw Markdown source code. `textwidth` is the column at
/// which text should be wrapped for display on a terminal; normally
/// this is [`textwrap::termwidth()`], but any value may be specified.
pub fn parse(markup: &str, textwidth: usize) -> String {
    MarkdownParser::new(textwidth).parse(&text::convert_html_entities(markup))
}

/// A reusable Markdown parser for formatting text suitable for output in a terminal.
#[derive(Debug)]
struct MarkdownParser {
    textwidth: usize,
}

impl MarkdownParser {
    /// Creates a new Markdown parser that will format text to the given width.
    pub fn new(textwidth: usize) -> Self {
        MarkdownParser { textwidth }
    }

    /// Converts Markdown markup into a formatted string.
    pub fn parse(&self, markup: &str) -> String {
        let tree = markdown::to_mdast(markup, &self.parse_options()).unwrap();
        let mut visitor = MarkdownVisitor::new(self.textwidth);
        tree.accept(&mut visitor);
        visitor.text()
    }

    fn parse_options(&self) -> ParseOptions {
        ParseOptions {
            constructs: Constructs {
                autolink: false,
                ..Constructs::default()
            },
            ..ParseOptions::default()
        }
    }
}

/// Provides default implementations of common Markdown parsing operations.
trait DefaultMarkdownVisitor: TextAppendable {
    fn textwidth(&self) -> usize;

    fn visit_emphasis(&mut self, node: &Node) {
        let mut subvisitor = MarkdownEmphasisVisitor::new();
        node.accept_children(&mut subvisitor);
        self.push_text(&subvisitor.text());
    }

    fn visit_strong(&mut self, node: &Node) {
        let mut subvisitor = MarkdownStrongEmphasisVisitor::new();
        node.accept_children(&mut subvisitor);
        self.push_text(&subvisitor.text());
    }

    fn visit_link(&mut self, url: &str, node: &Node) {
        let mut subvisitor = MarkdownLinkVisitor::new();
        node.accept_children(&mut subvisitor);
        let link_text = subvisitor.text();
        self.push_text(&format!("\u{1b}[4m{link_text}\u{1b}[24m <{url}>"));
    }

    fn visit_image(&mut self, url: &str) {
        self.push_text(&format!("\u{1b}[4m{url}\u{1b}[24m"));
    }

    fn visit_inline_code(&mut self, code: &str, node: &Node)
    where
        Self: Visitor + Sized,
    {
        self.push_text(&format!("`{code}`"));
        node.accept_children(self);
    }

    fn visit_html(&mut self, html: &str, node: &Node)
    where
        Self: Visitor + Sized,
    {
        // This is a hack to handle block HTML. Our parser does not differentiate
        // between inline and block HTML, so let's assume that if the HTML
        // contains a new line, it is block HTML. Not perfect in all situations
        // but probably the best we can do for now.
        if html.contains("\n") {
            self.push_text("\n\n");
        }

        self.push_text(html);
        node.accept_children(self);
    }

    fn visit_list(&mut self, ordered: &bool, start: &Option<u32>, node: &Node) {
        trace!("processing list node: {node:?}");

        let list_type = if *ordered {
            ListType::Ordered(start.unwrap_or(1))
        } else {
            ListType::Unordered
        };
        let mut subvisitor = MarkdownListVisitor::new(list_type, &self.textwidth());
        node.accept_children(&mut subvisitor);
        self.push_text("\n\n");
        self.push_text(&subvisitor.text());
    }
}

/// A Markdown code generator.
#[derive(Debug)]
struct MarkdownVisitor {
    text: String,
    textwidth: usize,
}

impl MarkdownVisitor {
    pub fn new(textwidth: usize) -> Self {
        Self {
            text: String::new(),
            textwidth,
        }
    }

    fn visit_paragraph(&mut self, node: &Node) {
        self.push_text("\n\n");
        node.accept_children(self);
    }

    fn visit_text(&mut self, text: &Text) {
        self.push_text(&text.value);
    }

    fn visit_break(&mut self) {
        self.push_text("\n");
    }

    fn visit_heading(&mut self, node: &Node) {
        self.push_text("\u{1b}[1m");
        node.accept_children(self);
        self.push_text("\u{1b}[22m");
    }

    fn visit_thematic_break(&mut self) {
        self.push_text(&"-".repeat(self.textwidth));
    }

    fn visit_code(&mut self, code: &str) {
        // TODO: Long lines of code should not be wrapped.
        // If a line of code is longer than 80 characters when indented,
        // it will be wrapped by the general mechanism that wraps all
        // the text (in MarkdownVisitor::text()). I would prefer lines
        // of code not be wrapped, but I have no idea how to prevent that.
        trace!("got code:\n{code}");
        self.push_text("\n\n");
        self.push_text(
            &code
                .split("\n")
                .map(|line| format!("    {line}"))
                .join("\n"),
        );
    }

    fn visit_blockquote(&mut self, node: &Node) {
        let mut subvisitor = MarkdownBlockquoteVisitor::new(self.textwidth());
        node.accept_children(&mut subvisitor);
        self.push_text("\n\n");
        self.push_text(&subvisitor.text());
    }
}

impl TextAppendable for MarkdownVisitor {
    fn push_text(&mut self, text: &str) {
        self.text += text;
    }
}

impl DefaultMarkdownVisitor for MarkdownVisitor {
    fn textwidth(&self) -> usize {
        self.textwidth
    }
}

impl Visitor for MarkdownVisitor {
    fn text(&self) -> String {
        textwrap::fill(self.text.trim_matches('\n'), self.textwidth)
    }

    fn visit(&mut self, node: &Node) {
        match node {
            Node::Root(_) => self.swallow(node),
            Node::Paragraph(_) => self.visit_paragraph(node),
            Node::Text(text) => self.visit_text(text),
            Node::Break(_) => self.visit_break(),
            Node::Emphasis(_) => self.visit_emphasis(node),
            Node::Strong(_) => self.visit_strong(node),
            Node::Heading(_) => self.visit_heading(node),
            Node::Link(Link { url, .. }) => self.visit_link(url, node),
            Node::Image(Image { url, .. }) => self.visit_image(url),
            Node::InlineCode(InlineCode { value, .. }) => self.visit_inline_code(value, node),
            Node::Html(Html { value, .. }) => self.visit_html(value, node),
            Node::ThematicBreak(_) => self.visit_thematic_break(),
            Node::List(List { ordered, start, .. }) => self.visit_list(ordered, start, node),
            Node::Code(Code { value, .. }) => self.visit_code(value),
            Node::Blockquote(_) => self.visit_blockquote(node),
            _ => self.unknown(node),
        }
    }
}

trait AnyEmphasisVisitor: Visitor {
    fn visit_text(&mut self, text: &Text);
    fn visit_emphasis(&mut self, node: &Node);
    fn visit_strong(&mut self, node: &Node);

    fn dispatch(&mut self, node: &Node) {
        match node {
            Node::Text(text) => self.visit_text(text),
            Node::Emphasis(_) => self.visit_emphasis(node),
            Node::Strong(_) => self.visit_strong(node),
            _ => self.unknown(node),
        }
    }
}

#[derive(Debug)]
struct MarkdownEmphasisVisitor {
    text: String,
    in_strong: bool,
}

impl MarkdownEmphasisVisitor {
    pub fn new() -> Self {
        Self {
            text: String::from(""),
            in_strong: false,
        }
    }
}

impl AnyEmphasisVisitor for MarkdownEmphasisVisitor {
    fn visit_text(&mut self, text: &Text) {
        let text = &text.value;
        self.text += text;
    }

    fn visit_emphasis(&mut self, node: &Node) {
        node.accept_children(self);
    }

    fn visit_strong(&mut self, node: &Node) {
        if self.text.len() == 0 {
            self.in_strong = true;
        } else {
            self.text += "\u{1b}[1m";
        }

        node.accept_children(self);

        if !self.in_strong {
            self.text += "\u{1b}[22m";
        }
    }
}

impl Visitor for MarkdownEmphasisVisitor {
    fn text(&self) -> String {
        let prefix_esc = if self.in_strong { "4;1" } else { "4" };
        let suffix_esc = if self.in_strong { "22;24" } else { "24" };
        let prefix = format!("\u{1b}[{prefix_esc}m");
        let suffix = format!("\u{1b}[{suffix_esc}m");
        format!("{prefix}{}{suffix}", self.text.trim())
    }

    fn visit(&mut self, node: &Node) {
        self.dispatch(node);
    }
}

#[derive(Debug)]
struct MarkdownStrongEmphasisVisitor {
    text: String,
    in_emphasis: bool,
}

impl MarkdownStrongEmphasisVisitor {
    pub fn new() -> Self {
        Self {
            text: String::from(""),
            in_emphasis: false,
        }
    }
}

impl AnyEmphasisVisitor for MarkdownStrongEmphasisVisitor {
    fn visit_text(&mut self, text: &Text) {
        let text = &text.value;
        self.text += text;
    }

    fn visit_emphasis(&mut self, node: &Node) {
        if self.text.len() == 0 {
            self.in_emphasis = true;
        } else {
            self.text += "\u{1b}[4m";
        }

        node.accept_children(self);

        if !self.in_emphasis {
            self.text += "\u{1b}[24m";
        }
    }

    fn visit_strong(&mut self, node: &Node) {
        node.accept_children(self);
    }
}

impl Visitor for MarkdownStrongEmphasisVisitor {
    fn text(&self) -> String {
        let prefix_esc = if self.in_emphasis { "1;4" } else { "1" };
        let suffix_esc = if self.in_emphasis { "24;22" } else { "22" };
        let prefix = format!("\u{1b}[{prefix_esc}m");
        let suffix = format!("\u{1b}[{suffix_esc}m");
        format!("{prefix}{}{suffix}", self.text.trim())
    }

    fn visit(&mut self, node: &Node) {
        self.dispatch(node);
    }
}

#[derive(Debug)]
struct MarkdownLinkVisitor {
    text: String,
}

impl MarkdownLinkVisitor {
    fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    fn visit_text(&mut self, text: &Text) {
        self.text += &text.value;
    }

    fn visit_inline_code(&mut self, code: &str) {
        self.text += code;
    }
}

impl Visitor for MarkdownLinkVisitor {
    fn text(&self) -> String {
        String::from(&self.text)
    }

    fn visit(&mut self, node: &Node) {
        match node {
            Node::Text(text) => self.visit_text(text),
            Node::InlineCode(InlineCode { value, .. }) => self.visit_inline_code(value),
            _ => self.swallow(node),
        }
    }
}

#[derive(Debug)]
enum ListType {
    Unordered,
    Ordered(u32),
}

impl ListType {
    fn mark(&self, list_items: &Vec<String>, textwidth: &usize) -> String {
        match self {
            ListType::Ordered(start) => self.mark_ordered(start, list_items, textwidth),
            ListType::Unordered => self.mark_unordered(list_items, textwidth),
        }
    }

    fn mark_unordered(&self, list_items: &Vec<String>, textwidth: &usize) -> String {
        let opts = Options::new(*textwidth)
            .initial_indent("  * ")
            .subsequent_indent("    ");
        list_items
            .iter()
            .map(|item| textwrap::fill(item, &opts))
            .join("\n")
    }

    fn mark_ordered(&self, start: &u32, list_items: &Vec<String>, textwidth: &usize) -> String {
        let width = list_items.len().to_string().len();
        let mut i = *start;
        list_items
            .iter()
            .map(|item| {
                let indent = format!("  {i:>width$}. ");
                let width = indent.len();
                let subseq_indent = " ".repeat(width);
                let opts = Options::new(*textwidth)
                    .initial_indent(&indent)
                    .subsequent_indent(&subseq_indent);
                let s = textwrap::fill(item, opts);
                i += 1;
                s
            })
            .join("\n")
    }
}

#[derive(Debug)]
struct MarkdownListVisitor {
    list_type: ListType,
    list_item_text: Vec<String>,
    textwidth: usize,
}

impl MarkdownListVisitor {
    pub fn new(list_type: ListType, textwidth: &usize) -> Self {
        Self {
            list_type,
            list_item_text: vec![],
            textwidth: *textwidth,
        }
    }

    fn visit_list_item(&mut self, node: &Node) {
        let mut subvisitor = MarkdownListItemVisitor::new(self.textwidth);
        node.accept_children(&mut subvisitor);
        self.list_item_text.push(subvisitor.text());
    }
}

impl Visitor for MarkdownListVisitor {
    fn text(&self) -> String {
        self.list_type.mark(&self.list_item_text, &self.textwidth)
    }

    fn visit(&mut self, node: &Node) {
        match node {
            Node::ListItem(_) => self.visit_list_item(node),
            _ => self.swallow(node),
        }
    }
}

#[derive(Debug)]
struct MarkdownListItemVisitor {
    text: String,
    textwidth: usize,
}

impl MarkdownListItemVisitor {
    pub fn new(textwidth: usize) -> Self {
        Self {
            text: String::new(),
            textwidth,
        }
    }

    fn visit_text(&mut self, text: &Text) {
        self.text += &text.value;
    }
}

impl TextAppendable for MarkdownListItemVisitor {
    fn push_text(&mut self, text: &str) {
        // Kind of a hack to handle nested lists, but it works.
        self.text += &text.replace("\n\n", "\n");
    }
}

impl DefaultMarkdownVisitor for MarkdownListItemVisitor {
    fn textwidth(&self) -> usize {
        self.textwidth
    }
}

impl Visitor for MarkdownListItemVisitor {
    fn text(&self) -> String {
        String::from(&self.text)
    }

    fn visit(&mut self, node: &Node) {
        match node {
            Node::Text(text) => self.visit_text(text),
            Node::Emphasis(_) => self.visit_emphasis(node),
            Node::Strong(_) => self.visit_strong(node),
            Node::Link(Link { url, .. }) => self.visit_link(url, node),
            Node::Image(Image { url, .. }) => self.visit_image(url),
            Node::InlineCode(InlineCode { value, .. }) => self.visit_inline_code(value, node),
            Node::Html(Html { value, .. }) => self.visit_html(value, node),
            Node::List(List { ordered, start, .. }) => self.visit_list(ordered, start, node),
            _ => self.swallow(node),
        }
    }
}

#[derive(Debug)]
struct MarkdownBlockquoteVisitor {
    text: String,
    textwidth: usize,
}

impl MarkdownBlockquoteVisitor {
    pub fn new(textwidth: usize) -> Self {
        Self {
            text: String::new(),
            textwidth,
        }
    }

    fn visit_paragraph(&mut self, node: &Node) {
        self.text += "\n\n";
        node.accept_children(self);
    }

    fn visit_text(&mut self, text: &Text) {
        self.push_text(&text.value);
    }
}

impl TextAppendable for MarkdownBlockquoteVisitor {
    fn push_text(&mut self, text: &str) {
        self.text += text;
    }
}

impl Visitor for MarkdownBlockquoteVisitor {
    fn text(&self) -> String {
        let indent = "  | ";
        let opts = Options::new(self.textwidth)
            .initial_indent(indent)
            .subsequent_indent(indent);
        textwrap::fill(&self.text.trim(), opts)
    }

    fn visit(&mut self, node: &Node) {
        match node {
            Node::Paragraph(_) => self.visit_paragraph(node),
            Node::Text(text) => self.visit_text(text),
            _ => self.unknown(node),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::markdown::test_utils::{load_markdown, load_output};
    use crate::test_utils::do_logging;
    //use pretty_assertions::assert_eq;

    const TEXTWIDTH: usize = 80;

    pub fn load_test(file: &str) -> (String, String) {
        (load_markdown(file), load_output(file))
    }

    fn parse(markup: &str) -> String {
        super::parse(markup, TEXTWIDTH)
    }

    #[test]
    fn it_does_not_touch_normal_text() {
        let text = "Lorem ipsum dolor sit amet";
        assert_eq!(parse(&text), text);
    }

    #[test]
    fn it_wraps_long_text_to_the_terminal_width() {
        let (text, expected) = load_test("wrap");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_does_not_touch_escaped_characters() {
        let text = "Lorem \\*ipsum\\* dolor sit amet";
        let expected = "Lorem *ipsum* dolor sit amet";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_does_not_touch_badly_formatted_text() {
        let text = "This text is _badly formatted** and has a [broken link]( and ![bad image(http://www.example.com/.";
        let expected = "This text is _badly formatted** and has a [broken link]( and ![bad image(http://\nwww.example.com/.";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_adds_double_empty_lines_after_paragraphs() {
        let (text, expected) = load_test("paragraphs");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    // Markdown parser doesn't really support this so we'd have to implement
    // it ourselves, and that's going to be a real pain. It's not a big deal
    // so just ignore it for now.
    #[ignore = "not possible right now"]
    fn it_collapses_single_linebreaks() {
        let text = "this is one line\nthis is not really a second line";
        let expected = "this is one line this is not really a second line";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_respects_manual_linebreaks() {
        let text = "this is one line  \nthis is actually a second line";
        let expected = "this is one line\nthis is actually a second line";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_emphasizes_text_with_asterisks() {
        let text = "this text is *emphasized*";
        let expected = "this text is \u{1b}[4memphasized\u{1b}[24m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_emphasizes_text_with_underscores() {
        let text = "this text is _emphasized_";
        let expected = "this text is \u{1b}[4memphasized\u{1b}[24m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_really_emphasizes_text_with_double_asterisks() {
        let text = "this text is **really emphasized**";
        let expected = "this text is \u{1b}[1mreally emphasized\u{1b}[22m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_really_emphasizes_text_with_double_underscores() {
        let text = "this text is __really emphasized__";
        let expected = "this text is \u{1b}[1mreally emphasized\u{1b}[22m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_really_really_emphasizes_text_with_triple_asterisks() {
        let text = "this text is ***really really emphasized***";
        let expected = "this text is \u{1b}[4;1mreally really emphasized\u{1b}[22;24m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_really_really_emphasizes_text_with_underscores_and_double_asterisks() {
        let text = "this text is **_really really emphasized_**";
        let expected = "this text is \u{1b}[1;4mreally really emphasized\u{1b}[24;22m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_really_really_emphasizes_text_with_underscores_and_double_asterisks_reversed() {
        let text = "this text is _**really really emphasized**_";
        let expected = "this text is \u{1b}[4;1mreally really emphasized\u{1b}[22;24m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_emphasizes_double_strong_text_with_underscores() {
        let text = "this text is ____really really emphasized____!";
        let expected = "this text is \u{1b}[1mreally really emphasized\u{1b}[22m!";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_really_emphasizes_double_strong_text_with_asterisks() {
        let text = "this text is ****really really emphasized****!";
        let expected = "this text is \u{1b}[1mreally really emphasized\u{1b}[22m!";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_emphasizes_nested_text_beginning_with_an_underscore() {
        let text = "this _text **is really emphasized** text_";
        let expected = "this \u{1b}[4mtext \u{1b}[1mis really emphasized\u{1b}[22m text\u{1b}[24m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_emphasizes_nested_text_beginning_with_asterisks() {
        let text = "this **text _is really emphasized_ text**";
        let expected = "this \u{1b}[1mtext \u{1b}[4mis really emphasized\u{1b}[24m text\u{1b}[22m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_emphasizes_nested_text_beginning_with_a_single_asterisk() {
        let text = "this *text **is really emphasized** text*";
        let expected = "this \u{1b}[4mtext \u{1b}[1mis really emphasized\u{1b}[22m text\u{1b}[24m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_emphasizes_nested_text_beginning_with_double_asterisks() {
        let text = "this **text *is really emphasized* text**";
        let expected = "this \u{1b}[1mtext \u{1b}[4mis really emphasized\u{1b}[24m text\u{1b}[22m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_does_not_style_unclosed_emphasis_with_underscores() {
        let text = "this text _has an unclosed emphasis";
        assert_eq!(parse(&text), text);
    }

    #[test]
    fn it_does_not_style_unclosed_emphasis_with_asterisks() {
        let text = "this text *has an unclosed emphasis";
        assert_eq!(parse(&text), text);
    }

    #[test]
    fn it_does_not_style_unclosed_strong_emphasis_with_double_asterisks() {
        let text = "this text **has an unclosed strong emphasis";
        assert_eq!(parse(&text), text);
    }

    #[test]
    fn it_converts_headers_to_bright_text() {
        for i in 1..=6 {
            let header = (0..i).map(|_| "#").collect::<Vec<_>>().join("");
            let text = format!("{header} Some Text");
            let expected = "\u{1b}[1mSome Text\u{1b}[22m";
            assert_eq!(parse(&text), expected);
        }
    }

    #[test]
    fn it_displays_link_labels_and_urls() {
        let text = "[usaidwat source](https://github.com/mdippery/usaidwat-next)";
        let expected =
            "\u{1b}[4musaidwat source\u{1b}[24m <https://github.com/mdippery/usaidwat-next>";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_displays_inline_links_as_is() {
        let text = "<https://github.com/mdippery/usaidwat-next>";
        assert_eq!(parse(&text), text);
    }

    #[test]
    fn it_does_not_touch_emphasis_in_links() {
        let text = "[_usaidwat source_](https://github.com/mdippery/usaidwat-next)";
        let expected =
            "\u{1b}[4musaidwat source\u{1b}[24m <https://github.com/mdippery/usaidwat-next>";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_does_not_touch_strong_emphasis_in_links() {
        let text = "[**usaidwat source**](https://github.com/mdippery/usaidwat-next)";
        let expected =
            "\u{1b}[4musaidwat source\u{1b}[24m <https://github.com/mdippery/usaidwat-next>";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_does_not_touch_inline_code_in_links() {
        let text = "[`usaidwat source`](https://github.com/mdippery/usaidwat-next)";
        let expected =
            "\u{1b}[4musaidwat source\u{1b}[24m <https://github.com/mdippery/usaidwat-next>";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_underlines_image_urls() {
        let text = "![A beautiful picture](https://www.example.com/a-beautiful-picture.jpg)";
        let expected = "\u{1b}[4mhttps://www.example.com/a-beautiful-picture.jpg\u{1b}[24m";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_does_not_touch_inline_code() {
        let text = "Here is a Rust struct: `struct SomeStruct;`";
        assert_eq!(parse(&text), text);
    }

    #[test]
    fn it_does_not_touch_strikethrough_text_with_single_tildes() {
        let text = "this text is ~gone~";
        assert_eq!(parse(&text), text);
    }

    #[test]
    fn it_does_not_touch_strikethrough_text_with_double_tildes() {
        let text = "this text is ~~gone~~";
        assert_eq!(parse(&text), text);
    }

    #[test]
    fn it_prepends_a_caret_to_superscripts() {
        let text = "x^y";
        assert_eq!(parse(&text), text);
    }

    #[test]
    fn it_wraps_long_superscript_text_in_parentheses() {
        let text = "x^(y + z)";
        assert_eq!(parse(&text), text);
    }

    #[test]
    fn it_indents_unordered_lists_by_two_spaces() {
        let (text, expected) = load_test("unordered_list");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_indents_unordered_lists_when_the_text_is_not_indented() {
        let text = load_markdown("unordered_list_unindented");
        let expected = load_output("unordered_list");
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_wraps_each_item_in_an_unordered_list() {
        let (text, expected) = load_test("unordered_list_long_items");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_parses_markdown_in_unordered_list_items() {
        let (text, expected) = load_test("unordered_list_embedded");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_parses_nested_unordered_lists() {
        let (text, expected) = load_test("nested_unordered_list");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_indents_ordered_lists_by_two_spaces() {
        let (text, expected) = load_test("ordered_list");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_indents_ordered_lists_when_the_text_is_not_indented() {
        let text = load_markdown("ordered_list_unindented");
        let expected = load_output("ordered_list");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_wraps_each_item_in_an_ordered_list() {
        let (text, expected) = load_test("ordered_list_long_items");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_sequentially_numbers_list_even_if_the_markup_isnt() {
        let text = load_markdown("ordered_list_nonsequential");
        let expected = load_output("ordered_list");
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_parses_markdown_in_ordered_list_items() {
        let (text, expected) = load_test("ordered_list_embedded");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_parses_nested_ordered_lists() {
        let (text, expected) = load_test("nested_ordered_list");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    // Our Markdown parser does a weird thing here where it emits three List
    // nodes all at the same level to handle nested lists, so it's going to
    // be a pain to properly format this. It's not absolutely crucial to do
    // this in production, so let's handle this case later.
    #[ignore = "cannot be parsed due to parser limitations"]
    fn it_parses_nested_lists_of_any_type() {
        do_logging();
        let (text, expected) = load_test("nested_mixed_list");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    #[ignore = "long lines of code should not be wrapped, but are"]
    fn it_indents_code_blocks_by_four_spaces_and_wraps_it() {
        do_logging();
        let (text, expected) = load_test("code");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    #[ignore = "long lines of code should not be wrapped, but are"]
    fn it_indents_fenced_code_blocks_by_four_spaces_and_wraps_it() {
        do_logging();
        let text = load_markdown("code_fenced");
        let expected = load_output("code");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_puts_a_pipe_in_front_of_block_quotes_and_wraps_it() {
        let (text, expected) = load_test("blockquote");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_formats_adjacent_blockquotes_as_a_single_quote() {
        let (text, expected) = load_test("blockquote_single");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_does_not_touch_inline_html() {
        let text = "this HTML has <span>inline text</span>";
        assert_eq!(parse(&text), text);
    }

    #[test]
    fn it_processes_inline_html_containing_markdown() {
        let text = "this HTML has <span>_emphasized_ inline text</span>";
        let expected = "this HTML has <span>\u{1b}[4memphasized\u{1b}[24m inline text</span>";
        assert_eq!(parse(&text), expected);
    }

    #[test]
    fn it_does_not_touch_block_html() {
        do_logging();
        let (text, expected) = load_test("block_html");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }

    #[test]
    fn it_converts_horizontal_rules_to_fill_the_screen_width() {
        let text = "---\n";
        let actual_len = parse(&text).trim_end().len();
        assert_eq!(actual_len, TEXTWIDTH, "{actual_len} != {TEXTWIDTH}");
    }

    #[test]
    fn it_processes_html_entities() {
        let text = "&lt;this &amp; &quot;that&quot;&gt;";
        assert_eq!(parse(&text), "<this & \"that\">");
    }

    #[test]
    #[ignore = "should pass but need to expand parser capabilities"]
    fn it_parses_complex_markdown() {
        let (text, expected) = load_test("markdown");
        let actual = parse(&text);
        assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
    }
}
