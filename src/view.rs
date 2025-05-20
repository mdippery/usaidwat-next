use crate::client::Redditor;
use chrono::Local;
use indoc::{formatdoc, indoc};

pub trait Viewable {
    fn view(&self) -> String;
}

impl Viewable for Redditor {
    fn view(&self) -> String {
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
    mod format_info {
        use super::super::*;
        use crate::client::Redditor;

        #[test]
        fn it_formats_a_user() {
            let user = Redditor::test();
            let actual = user.view();
            // TODO: Eventually the "17 years" part will fail, so I
            //       really should be mocking time, but we'll cross that
            //       bridge when we come to it.
            //       Will also have to mock Local so the tests always use
            //       the same local time zone (PDT).
            let expected = indoc! {"
                Created: Mar 31, 2008 15:55 PM (17 years ago)
                Link Karma: 11729
                Comment Karma: 121995"};
            assert_eq!(actual, expected);
        }
    }
}
