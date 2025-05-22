//! Draws viewable objects into a terminal window.

use crate::client::Redditor;
use chrono::Local;
use indoc::formatdoc;

/// View renderer options.
#[derive(Debug, Default)]
pub struct ViewOptions {
    oneline: bool,
    raw: bool,
}

/// Marks an item that can be converted into a string for display on a terminal.
pub trait Viewable {
    /// Converts the item into a string for display on a terminal.
    fn view(&self, opts: &ViewOptions) -> String;
}

impl Viewable for Redditor {
    fn view(&self, opts: &ViewOptions) -> String {
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

#[cfg(test)]
mod tests {
    fn load_output(filename: &str) -> String {
        std::fs::read_to_string(format!("tests/data/{filename}.out")).unwrap()
    }

    mod format_info {
        use super::super::*;
        use super::load_output;
        use crate::client::Redditor;

        #[test]
        fn it_formats_a_user() {
            let user = Redditor::test();
            let actual = user.view(&ViewOptions::default());
            // TODO: Eventually the "17 years" part will fail, so I
            //       really should be mocking time, but we'll cross that
            //       bridge when we come to it.
            //       Will also have to mock Local so the tests always use
            //       the same local time zone (PDT).
            let output = load_output("about_mipadi");
            let expected = output.trim();
            assert_eq!(actual, expected);
        }
    }
}
