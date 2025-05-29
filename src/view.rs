//! Formats viewable objects for display in a terminal.

use crate::cli::DateFormat;
use crate::client::{Redditor, Timeline};
use crate::clock::{Clock, HasAge, SystemClock};
use crate::thing::{Comment, Submission};
use chrono::Local;
use colored::Colorize;
use indoc::formatdoc;
use textwrap::{self, Options};

/// View renderer options.
#[derive(Debug, Default)]
pub struct ViewOptions {
    date_format: DateFormat,
    oneline: bool,
    raw: bool,
}

impl ViewOptions {
    /// Incrementally builds a new set of view options.
    ///
    /// # Examples
    ///
    /// ```
    /// use usaidwat::view::ViewOptions;
    /// let opts = ViewOptions::build().oneline(true).raw(false).build();
    /// ```
    pub fn build() -> ViewOptionsBuilder {
        ViewOptionsBuilder::default()
    }
}

/// A builder for view options.
///
/// You probably don't want to use this directly; call [`ViewOptions::build()`]
/// and construct it incrementally instead.
#[derive(Debug, Default)]
#[must_use]
pub struct ViewOptionsBuilder {
    date_format: DateFormat,
    oneline: bool,
    raw: bool,
}

impl ViewOptionsBuilder {
    /// Sets the date format option to relative or absolute.
    pub fn date_format(mut self, date_format: DateFormat) -> Self {
        self.date_format = date_format;
        self
    }

    /// Sets the "oneline" option to true or false.
    pub fn oneline(mut self, oneline: bool) -> Self {
        self.oneline = oneline;
        self
    }

    /// Sets the "raw" option to true or false.
    pub fn raw(mut self, raw: bool) -> Self {
        self.raw = raw;
        self
    }

    /// Finalizes the [`ViewOptions`].
    pub fn build(self) -> ViewOptions {
        ViewOptions {
            date_format: self.date_format,
            oneline: self.oneline,
            raw: self.raw,
        }
    }
}

/// Marks an item that can be converted into a string for display on a terminal.
pub trait Viewable {
    /// Converts the item into a string for display on a terminal.
    ///
    /// `opts` are a set of view options that control formatting of the
    /// resulting string. `clock` is a source from which the current
    /// time can be derived for items that print their age or other
    /// time-based parameters.
    fn view<C: Clock>(&self, opts: &ViewOptions, clock: &C) -> String;
}

impl Viewable for Redditor {
    fn view<C: Clock>(&self, _: &ViewOptions, _: &C) -> String {
        formatdoc! {"
            Created: {} ({})
            Link Karma: {}
            Comment Karma: {}",
            self.created_utc().with_timezone(&Local).format("%b %d, %Y %H:%M %p"),
            self.relative_age(&SystemClock::new()),
            self.link_karma(),
            self.comment_karma(),
        }
    }
}

impl Viewable for Comment {
    fn view<C: Clock>(&self, opts: &ViewOptions, clock: &C) -> String {
        if opts.oneline {
            self.view_oneline(opts, clock)
        } else {
            self.view_full(opts, clock)
        }
    }
}

impl Comment {
    fn view_full<C: Clock>(&self, opts: &ViewOptions, clock: &C) -> String {
        // TODO: Disable color if not outputting to tty

        let age = self.format_date(opts, clock);

        // TODO: Formatted as Markdown (wrap after formatting unless raw)
        let body = self.wrapped_body();

        formatdoc! {"
            {}
            {}
            {}
            {} {} {}

            {body}",
            self.subreddit().green(),
            self.permalink().yellow(),
            self.link_title().magenta(),
            age.blue(),
            "\u{2022}".cyan(),
            format!("{:+}", self.score()).blue(),
        }
    }

    fn view_oneline<C: Clock>(&self, _: &ViewOptions, _: &C) -> String {
        format!("{} {}", self.subreddit(), self.link_title())
    }

    fn format_date<C: Clock>(&self, opts: &ViewOptions, clock: &C) -> String {
        match opts.date_format {
            DateFormat::Relative => self.relative_age(clock),
            DateFormat::Absolute => self.format_absolute_date(),
        }
    }

    fn format_absolute_date(&self) -> String {
        let date = self.created_local();
        let date_part = format!("{}", date.format("%a, %-d %b %Y"));
        let time_part = format!("{}", date.format("%l:%M %p"));
        // %l formats a single-digit time as, e.g., " 8",
        // but I want to trim off the leading space.
        let time_part = time_part.trim();
        format!("{date_part}, {time_part}")
    }

    fn wrapped_body(&self) -> String {
        let body = self.body();
        // Subtract 1 from terminal width because it looks nicer if the text
        // doesn't run right up to the edge.
        let opts = Options::new(textwrap::termwidth() - 1);
        textwrap::fill(&body, opts)
    }
}

impl Viewable for Submission {
    fn view<C: Clock>(&self, opts: &ViewOptions, clock: &C) -> String {
        // TODO: Only use color if printing to tty

        if opts.oneline {
            self.view_oneline()
        } else {
            self.view_full(clock)
        }
    }
}

impl Submission {
    fn view_full<C: Clock>(&self, clock: &C) -> String {
        String::from(
            formatdoc! {"
                {}
                {}
                {}
                {}
                {}",
                self.subreddit().green(),
                self.short_permalink().yellow(),
                self.title().magenta(),
                self.relative_age(clock).blue(),
                self.link_uri(),
            }
            // Remove trailing space since the link will be blank for self posts
            .trim_end(),
        )
    }

    fn view_oneline(&self) -> String {
        format!("{} {}", self.subreddit().green(), self.title())
    }

    fn short_permalink(&self) -> String {
        let permalink = self.permalink();
        let parts: Vec<_> = permalink.split("/").collect();
        let len = parts.len().saturating_sub(2);
        parts[..len].join("/")
    }

    fn link_uri(&self) -> &str {
        if self.is_self() { "" } else { self.url() }
    }
}

impl Viewable for Timeline {
    fn view<C: Clock>(&self, _: &ViewOptions, _: &C) -> String {
        let mut s = String::from(" ");
        s += (0..24)
            .map(|i| format!("{i:>3}"))
            .collect::<Vec<_>>()
            .join("")
            .as_ref();
        s += "\n";
        s += self
            .days()
            .map(|(wday, day)| {
                let wday = format!("{wday}")
                    .chars()
                    .nth(0)
                    .expect(&format!("could not get first character of {wday}"));
                let days = day
                    // Remove this line to print the number of comments instead of a *
                    .map(|d| if d > 0 { '*' } else { ' ' })
                    .map(|ch| format!("{ch:>3}"))
                    .join("");
                format!("{wday}{days}")
            })
            .collect::<Vec<_>>()
            .join("\n")
            .as_ref();
        s
    }
}

#[cfg(test)]
mod tests {
    fn load_output(filename: &str) -> String {
        let filename = format!("tests/output/{filename}.out");
        String::from(
            std::fs::read_to_string(&filename)
                .expect(&format!("could not load test data from {filename}"))
                .trim_end(),
        )
    }

    mod view_options {
        use super::super::*;

        #[test]
        fn it_returns_default_options() {
            let opts = ViewOptions::default();
            assert_eq!(opts.date_format, DateFormat::default());
            assert!(!opts.oneline);
            assert!(!opts.raw);
        }

        #[test]
        fn it_returns_custom_options() {
            let opts = ViewOptions::build()
                .oneline(true)
                .raw(true)
                .date_format(DateFormat::Absolute);
            assert_eq!(opts.date_format, DateFormat::Absolute);
            assert!(opts.oneline);
            assert!(opts.raw);
        }

        #[test]
        fn it_returns_custom_options_with_only_oneline() {
            let opts = ViewOptions::build().oneline(true);
            assert_eq!(opts.date_format, DateFormat::default());
            assert!(opts.oneline);
            assert!(!opts.raw);
        }

        #[test]
        fn it_returns_custom_options_with_only_raw() {
            let opts = ViewOptions::build().raw(true);
            assert_eq!(opts.date_format, DateFormat::default());
            assert!(!opts.oneline);
            assert!(opts.raw);
        }
    }

    mod format_info {
        use super::super::*;
        use super::load_output;
        use crate::client::Redditor;
        use crate::test_utils::FrozenClock;

        #[test]
        fn it_formats_a_user() {
            let user = Redditor::test();
            let actual = user.view(&ViewOptions::default(), &FrozenClock::default());
            let expected = load_output("about_mipadi");
            assert_eq!(actual, expected);
        }
    }

    mod format_comment {
        use super::super::*;
        use super::load_output;
        use crate::client::Redditor;
        use crate::test_utils::FrozenClock;
        use pretty_assertions::assert_eq;

        // TODO: Test with and without color when possible

        fn get_comment(n: usize) -> Comment {
            Redditor::test()
                .comments()
                .nth(n)
                .expect("no comment found")
        }

        #[test]
        fn it_formats_an_absolute_date() {
            let actual = get_comment(0).format_absolute_date();
            let expected = "Thu, 17 Apr 2025, 8:44 PM";
            assert_eq!(actual, expected);
        }

        #[test]
        #[ignore]
        fn it_wraps_raw_text() {
            todo!("need to test");
        }

        #[test]
        #[ignore]
        fn it_wraps_formatted_markdown_text() {
            todo!("need to test");
        }

        #[test]
        fn it_formats_a_comment_with_no_markdown_markup() {
            let opts = ViewOptions::default();
            let comment = get_comment(0);
            let actual = comment.view(&opts, &FrozenClock::default());
            let expected = load_output("comments_no_markdown");
            assert_eq!(actual, expected);
        }

        #[test]
        #[ignore]
        fn it_formats_a_comment_with_markdown_markup() {
            todo!("format markdown and test!");
        }

        #[test]
        #[ignore]
        fn it_formats_a_comment_with_a_raw_body() {
            let opts = ViewOptions::build().raw(true).build();
            let actual = get_comment(2).view(&opts, &FrozenClock::default());
            let expected = load_output("comments_raw_body");
            assert_eq!(actual, expected);
        }

        #[test]
        fn it_formats_a_comment_with_relative_dates() {
            let opts = ViewOptions::build()
                .date_format(DateFormat::Relative)
                .build();
            let actual = get_comment(0).view(&opts, &FrozenClock::default());
            let expected = load_output("comments_relative_dates");
            assert_eq!(actual, expected);
        }

        #[test]
        fn it_formats_a_comment_with_absolute_dates() {
            let opts = ViewOptions::build()
                .date_format(DateFormat::Absolute)
                .build();
            let actual = get_comment(0).view(&opts, &FrozenClock::default());
            let expected = load_output("comments_absolute_dates");
            assert_eq!(actual, expected);
        }

        #[test]
        fn it_formats_a_comment_on_oneline() {
            let opts = ViewOptions::build().oneline(true).build();
            let actual = get_comment(0).view(&opts, &FrozenClock::default());
            let expected = "cyphersystem Cypher System & ChatGPT";
            assert_eq!(actual, expected);
        }
    }

    mod format_submission {
        use super::super::*;
        use super::load_output;
        use crate::client::Redditor;
        use crate::test_utils::FrozenClock;
        use crate::thing::Submission;
        use pretty_assertions::assert_eq;

        fn get_post(n: usize) -> Submission {
            Redditor::test()
                .submissions()
                .nth(n)
                .expect("no comment found")
        }

        #[test]
        fn it_returns_the_permalink_without_the_full_title() {
            let post = get_post(0);
            let expected = "https://www.reddit.com/r/rpg/comments/1hv9k9l";
            assert_eq!(post.short_permalink(), expected);
        }

        #[test]
        fn it_returns_the_link_uri() {
            let post = get_post(0);
            let expected = "https://acoup.blog/2025/01/03/collections-coinage-and-the-tyranny-of-fantasy-gold/";
            assert_eq!(post.link_uri(), expected);
        }

        #[test]
        fn it_returns_an_empty_link_uri_for_self_posts() {
            let post = get_post(3);
            assert_eq!(post.link_uri(), "")
        }

        #[test]
        fn it_formats_a_post() {
            let post = get_post(0);
            let expected = load_output("posts");
            let actual = post.view(&ViewOptions::default(), &FrozenClock::default());
            assert_eq!(actual, expected);
        }

        #[test]
        fn it_formats_a_self_post() {
            let post = get_post(3);
            let expected = load_output("posts_self");
            let actual = post.view(&ViewOptions::default(), &FrozenClock::default());
            assert_eq!(actual, expected);
        }

        #[test]
        fn it_formats_a_post_on_oneline() {
            let opts = ViewOptions::build().oneline(true).build();
            let post = get_post(0);
            let expected = load_output("posts_oneline");
            let actual = post.view(&opts, &FrozenClock::default());
            assert_eq!(actual, expected);
        }

        #[test]
        fn it_wraps_long_post_titles_for_oneline_posts() {
            let opts = ViewOptions::build().oneline(true).build();
            let post = get_post(4);
            let expected = load_output("posts_oneline_long");
            let actual = post.view(&opts, &FrozenClock::default());
            assert_eq!(actual, expected);
        }
    }

    mod format_timeline {
        use super::super::*;
        use super::load_output;
        use crate::client::Redditor;
        use crate::test_utils::FrozenClock;

        #[test]
        fn it_formats_a_timeline() {
            let user = Redditor::test();
            let actual = user
                .timeline()
                .view(&ViewOptions::default(), &FrozenClock::default());
            let expected = load_output("timeline_mipadi");
            assert_eq!(actual, expected);
        }
    }
}
