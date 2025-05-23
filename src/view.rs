//! Draws viewable objects into a terminal window.

use crate::client::{Redditor, Timeline};
use crate::clock::Clock;
use chrono::Local;
use indoc::formatdoc;
use std::ops::Index;

/// View renderer options.
#[derive(Debug, Default)]
pub struct ViewOptions {
    oneline: bool,
    raw: bool,
}

impl ViewOptions {
    pub fn from(oneline: bool, raw: bool) -> Self {
        Self { oneline, raw }
    }
}

/// Marks an item that can be converted into a string for display on a terminal.
pub trait Viewable {
    /// Converts the item into a string for display on a terminal.
    fn view(&self, opts: &ViewOptions) -> String;
}

impl<C: Clock> Viewable for Redditor<C> {
    fn view(&self, _: &ViewOptions) -> String {
        formatdoc! {"
            Created: {} ({})
            Link Karma: {}
            Comment Karma: {}",
            self.created_at().with_timezone(&Local).format("%b %d, %Y %H:%M %p"),
            self.relative_age(),
            self.link_karma(),
            self.comment_karma(),
        }
    }
}

impl Viewable for Timeline {
    fn view(&self, _: &ViewOptions) -> String {
        // TODO: Print in color with intensity proportional to number of comments
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
        let filename = format!("tests/data/{filename}.out");
        std::fs::read_to_string(&filename)
            .expect(&format!("could not load test data from {filename}"))
    }

    mod view_options {
        use super::super::*;

        fn it_returns_default_options() {
            let opts = ViewOptions::default();
            assert!(!opts.oneline);
            assert!(!opts.raw);
        }

        fn it_returns_custom_options() {
            let opts = ViewOptions::from(true, true);
            assert!(opts.oneline);
            assert!(opts.raw);
        }
    }

    mod format_info {
        use super::super::*;
        use super::load_output;
        use crate::client::Redditor;

        #[test]
        fn it_formats_a_user() {
            let user = Redditor::test();
            let actual = user.view(&ViewOptions::default());
            let output = load_output("about_mipadi");
            let expected = output.trim();
            assert_eq!(actual, expected);
        }
    }

    mod format_timeline {
        use super::super::*;
        use super::load_output;
        use crate::client::Redditor;

        #[test]
        fn it_formats_a_timeline() {
            let user = Redditor::test();
            let actual = user.timeline().view(&ViewOptions::default());
            let output = load_output("timeline_mipadi");
            let expected = output.trim_end();
            assert_eq!(actual, expected);
        }
    }
}
