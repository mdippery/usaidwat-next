//! A "thing" in the Reddit sense.
//!
//! Historically in the Reddit API and its old source code, a "Thing" was
//! any element of the Reddit system: users, posts, comments, etc. This
//! module encapsulates that idea and provides an easy way to more or less
//! work with JSON data from the Reddit API.

use crate::clock::{DateTime, HasAge, Local, Utc};
use crate::count::HasSubreddit;
use crate::filter::Searchable;
use crate::{markdown, text};
use log::error;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

/// A Reddit user account.
#[derive(Debug)]
pub struct User {
    about: About,
    comments: Vec<Comment>,
    submissions: Vec<Submission>,
}

/// Reddit user account data.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct About {
    name: String,
    id: String,
    #[serde(deserialize_with = "from_timestamp_f64")]
    created_utc: DateTime<Utc>,
    link_karma: i64,
    comment_karma: i64,
}

/// A Reddit comment.
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct Comment {
    id: String,
    name: String,
    subreddit_id: String,
    subreddit: String,
    link_title: String,
    link_id: String,
    #[serde(deserialize_with = "from_timestamp_f64")]
    created_utc: DateTime<Utc>,
    body: String,
    ups: i64,
    downs: i64,
    score: i64,
}

/// A Reddit Post.
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct Submission {
    id: String,
    name: String,
    permalink: String,
    author: String,
    domain: String,
    subreddit_id: String,
    subreddit: String,
    url: String,
    title: String,
    selftext: String,
    #[serde(deserialize_with = "from_timestamp_f64")]
    created_utc: DateTime<Utc>,
    num_comments: u64,
    ups: i64,
    downs: i64,
    score: i64,
}

impl User {
    /// Parses text responses from the Reddit API into the associated
    /// data structures.
    ///
    /// `user_data` is the result of a call to `/users/<user>/about.json`
    /// and contains account medata, `comment_data` is the result of a call
    /// to `/users/<user>/comments.json`, and `post_data` is the result of
    /// a call to `/users/<user>/submitted.json`.
    ///
    /// Obviously parsing can fail so this method returns an `Option`.
    pub fn parse(user_data: &str, comment_data: &str, post_data: &str) -> Option<Self> {
        let about = About::parse(user_data)?;
        let comments = Comment::parse(comment_data)?;
        let submissions = Submission::parse(post_data)?;
        Some(User {
            about,
            comments,
            submissions,
        })
    }

    /// Returns account data for the user.
    pub fn about(&self) -> &About {
        &self.about
    }

    /// User's comments.
    pub fn comments(&self) -> impl Iterator<Item = Comment> {
        self.comments.clone().into_iter()
    }

    /// User's submissions.
    pub fn submissions(&self) -> impl Iterator<Item = Submission> {
        self.submissions.clone().into_iter()
    }
}

impl About {
    /// Parses a text response from the Reddit API into user data.
    ///
    /// Specifically, `user_data` is the result of a call to
    /// `/users/<user>/about.json`.
    ///
    /// This method is generally invoked by `User`, not directly.
    pub fn parse(user_data: &str) -> Option<Self> {
        serde_json::from_str(user_data)
            .inspect_err(|err| error!("failed to parse user data: {err:?}"))
            .ok()
            .map(|wrapper: AboutResponse| wrapper.data)
    }

    /// The date on which the account was created.
    pub fn created_utc(&self) -> DateTime<Utc> {
        self.created_utc
    }

    /// User's current karma for submissions.
    pub fn link_karma(&self) -> i64 {
        self.link_karma
    }

    /// User's current karma for comments.
    pub fn comment_karma(&self) -> i64 {
        self.comment_karma
    }
}

impl Comment {
    /// Parses a text response from the Reddit API into a list of comments.
    ///
    /// Specifically, `comment_data` is the result of a call to
    /// `/users/<user>/comments.json`.
    ///
    /// This method is generally invoked by `User`, not directly.
    pub fn parse(comment_data: &str) -> Option<Vec<Self>> {
        serde_json::from_str(comment_data)
            .inspect_err(|err| error!("failed to parse comment data: {err:?}"))
            .ok()
            .map(|comment_listing: ListingResponse<CommentResponse>| {
                comment_listing
                    .data
                    .children
                    .into_iter()
                    .map(|comment_wrapper| comment_wrapper.data)
                    .collect()
            })
    }

    /// The time the comment was created, in local time.
    pub fn created_local(&self) -> DateTime<Local> {
        self.created_utc().with_timezone(&Local)
    }

    /// The full URL at which the comment can be retrieved.
    pub fn permalink(&self) -> String {
        self.link_id.split("_").last().and_then(|link_id| {
            // Reddit itself uses the submission title here, but in practice,
            // it can be anything. Since a Comment doesn't have a link back
            // to its submission, we'll just use a placeholder instead of
            // the submission title.
            let placeholder = String::from("z");

            let subreddit = self.subreddit();
            let comment_id = &self.id;
            let uri = format!("https://www.reddit.com/r/{subreddit}/comments/{link_id}/{placeholder}/{comment_id}");
            Some(uri)
        }).unwrap_or(String::from("?"))
    }

    /// The title of the link for which the comment was posted.
    ///
    /// HTML entities in the title will be converted.
    pub fn link_title(&self) -> String {
        text::convert_html_entities(&self.link_title)
    }

    /// The comment's total score.
    pub fn score(&self) -> i64 {
        self.score
    }

    /// The comment body, as formatted Markdown text.
    ///
    /// The formatted text converts Markdown markup into terminal escape
    /// codes for elegant display in a terminal.
    pub fn body(&self) -> String {
        markdown::parse(&self.body, textwrap::termwidth())
    }

    /// The comment body, as raw Markdown text, with HTML entities converted
    /// to their respective characters.
    ///
    /// Reddit converts some raw Markdown characters to HTML entities;
    /// for example, `>` in raw Markdown markup will be returned from
    /// Reddit as `&gt;` to be compatible with HTML and XML. This method
    /// will convert HTML entities like `&gt;` into their corresponding
    /// characters, but it will not do any additional parsing of Markdown
    /// text. In other words, the text returned by this method is suitable
    /// for passing into a Markdown parser.
    pub fn raw_body(&self) -> String {
        text::convert_html_entities(&self.body)
    }
}

impl HasAge for Comment {
    /// The time the comment was created, in UTC.
    fn created_utc(&self) -> DateTime<Utc> {
        self.created_utc
    }
}

impl HasSubreddit for Comment {
    /// The subreddit the comment was posted in.
    fn subreddit(&self) -> &str {
        &self.subreddit.trim()
    }
}

impl Searchable for Comment {
    fn search_text(&self) -> String {
        self.body()
    }
}

impl Submission {
    /// Parses a text response from the Reddit API into a list of
    /// submissions (posts).
    ///
    /// Specifically, `post_data` is the result of a call to
    /// `/users/<user>/submitted.json`.
    ///
    /// This method is generally invoked by `User`, not directly.
    pub fn parse(post_data: &str) -> Option<Vec<Self>> {
        serde_json::from_str(post_data)
            .inspect_err(|err| error!("failed to parse post data: {err:?}"))
            .ok()
            .map(|comment_listing: ListingResponse<SubmissionResponse>| {
                comment_listing
                    .data
                    .children
                    .into_iter()
                    .map(|comment_wrapper| comment_wrapper.data)
                    .collect()
            })
    }

    /// True if the submission is a self post.
    pub fn is_self(&self) -> bool {
        self.domain.starts_with("self.")
    }

    /// The submission's permalink.
    pub fn permalink(&self) -> String {
        let path = &self.permalink;
        format!("https://www.reddit.com{path}")
    }

    /// The submission's title.
    pub fn title(&self) -> String {
        text::convert_html_entities(&self.title)
    }

    /// The URL to which the submission points.
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl HasAge for Submission {
    fn created_utc(&self) -> DateTime<Utc> {
        self.created_utc
    }
}

impl HasSubreddit for Submission {
    /// The subreddit in which the submission was posted.
    fn subreddit(&self) -> &str {
        &self.subreddit
    }
}

// Deserializers
// --------------------------------------------------------------------------

fn from_timestamp_f64<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let ts_f64 = f64::deserialize(deserializer)?;
    let ts = f64_to_i64(ts_f64)
        .ok_or_else(|| Error::custom(format!("Invalid Unix timestamp: {ts_f64}")))?;
    DateTime::from_timestamp(ts, 0)
        .ok_or_else(|| Error::custom(format!("Invalid Unix timestamp: {ts}")))
}

fn f64_to_i64(n: f64) -> Option<i64> {
    if n.is_finite() && n <= i64::MAX as f64 {
        Some(n.trunc() as i64)
    } else {
        None
    }
}

// Response wrappers
// --------------------------------------------------------------------------
// These are necessary because the Reddit API returns data wrapped in "data"
// and "children" keys, so serde_json has to first parse these parent keys
// that we don't really care about to get to the "real" data.

#[derive(Debug, Deserialize)]
struct AboutResponse {
    data: About,
}

#[derive(Debug, Deserialize)]
struct ListingResponse<T> {
    data: ChildrenResponse<T>,
}

#[derive(Debug, Deserialize)]
struct ChildrenResponse<T> {
    children: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct CommentResponse {
    data: Comment,
}

#[derive(Debug, Deserialize)]
struct SubmissionResponse {
    data: Submission,
}

// Unit tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    mod about {
        use super::super::*;
        use crate::test_utils::load_data;

        #[test]
        fn it_cannot_parse_invalid_data() {
            let about = About::parse(&load_data("about_404"));
            assert!(about.is_none(), "should be None, was {about:?}");
        }

        #[test]
        fn it_can_parse_valid_data() {
            let about = About::parse(&load_data("about_mipadi"));
            assert!(about.is_some());
        }

        #[test]
        fn it_parses_fields() {
            let about = About::parse(&load_data("about_mipadi")).unwrap();
            let expected_created_at = DateTime::from_timestamp(1207004126, 0).unwrap();
            assert_eq!(about.created_utc(), expected_created_at);
            assert_eq!(
                about.created_utc().to_rfc2822(),
                "Mon, 31 Mar 2008 22:55:26 +0000"
            );
            assert_eq!(
                about.created_utc().to_rfc3339(),
                "2008-03-31T22:55:26+00:00"
            );
            assert_eq!(about.link_karma(), 11729);
            assert_eq!(about.comment_karma(), 121995);
        }
    }

    mod comments {
        use super::super::*;
        use crate::test_utils::{load_data, load_output};
        use pretty_assertions::assert_eq;

        #[test]
        fn it_cannot_parse_invalid_data() {
            let comments = Comment::parse(&load_data("comments_404"));
            assert!(comments.is_none(), "should be None, was {comments:?}");
        }

        #[test]
        fn it_can_parse_valid_data() {
            let comments = Comment::parse(&load_data("comments_mipadi"));
            assert!(comments.is_some());
        }

        #[test]
        fn it_can_parse_empty_data() {
            let comments = Comment::parse(&load_data("comments_empty"));
            assert!(comments.is_some());
        }

        #[test]
        fn it_parses_fields() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            assert_eq!(comments.len(), 100);

            let expected_link_title = "I dont want to play and we didn't even start";
            let expected_body = "Honestly, min/maxing and system mastery is a big part of the \
                Pathfinder community. It's a fairly crunchy system that draws in the sort of \
                players who really like finding ways to exploit the rules. Supposedly some groups \
                are more focused on roleplaying, but I have yet to meet a PF2 player in real life \
                who gives a shit about pesky, whimsical things like _story_. If that's not your \
                thing, you probably won't see eye to eye with the Pathfinder players you meet.\
                \n\nI'm in a slightly similar boat right now: I don't care that much about \
                min/maxing, but I put up with my Pathfinder friends because I really like our \
                group and I like them as people well enough.";
            let expected_created_utc = DateTime::from_timestamp(1743054429, 0).unwrap();

            // Parse comment 9 because it has negative ups and I want to test for that
            let comment = &comments[9];
            assert_eq!(comment.id, "mjyuqdz");
            assert_eq!(comment.name, "t1_mjyuqdz");
            assert_eq!(comment.subreddit_id, "t5_2qh2s");
            assert_eq!(comment.subreddit, "rpg");
            assert_eq!(comment.link_title, expected_link_title);
            assert_eq!(comment.link_id, "t3_1jktw0c");
            assert_eq!(comment.created_utc, expected_created_utc);
            assert_eq!(
                comment.created_utc.to_rfc2822(),
                "Thu, 27 Mar 2025 05:47:09 +0000"
            );
            assert_eq!(
                comment.created_utc.to_rfc3339(),
                "2025-03-27T05:47:09+00:00"
            );
            assert_eq!(comment.body, expected_body);
            assert_eq!(comment.ups, -3);
            assert_eq!(comment.downs, 0);
            assert_eq!(comment.score, -3);
        }

        #[test]
        fn it_returns_its_score() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[9];
            assert_eq!(comment.score(), -3);
        }

        #[test]
        fn it_returns_its_subreddit() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[0];
            assert_eq!(comment.subreddit(), "cyphersystem");
        }

        #[test]
        #[ignore]
        fn it_trims_whitespace_from_its_subreddit() {
            todo!("figure out how to test this");
        }

        #[test]
        fn it_returns_its_permalink() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[0];
            let expected = "https://www.reddit.com/r/cyphersystem/comments/1k1iixf/z/mnpd3zh";
            let actual = comment.permalink();
            assert_eq!(actual, expected);
        }

        #[test]
        fn it_returns_its_link_title() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            // Get the 0th comment because it has an ampersand and I want to test the conversion
            let comment = &comments[0];
            assert_eq!(comment.link_title(), "Cypher System & ChatGPT");
        }

        #[test]
        #[ignore]
        fn it_trims_whitespace_from_link_titles() {
            todo!("figure out how to test this");
        }

        #[test]
        fn it_returns_its_body() {
            let expected = load_output("comments_body");
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[9];
            let actual = comment.body();
            assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
        }

        #[test]
        fn it_converts_html_entities_in_its_body() {
            let expected = load_output("comments_html_entities");
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[3];
            let actual = comment.body();
            assert_eq!(actual, expected, "\nleft:\n{actual}\n\nright:\n{expected}");
        }

        #[test]
        #[ignore]
        fn it_trims_whitespace_from_its_body() {
            todo!("figure out how to test this")
        }

        #[test]
        fn it_matches_a_fixed_string() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[9];
            let result = comment.matches("min/maxing");
            assert!(result, "{result} != true");
        }

        #[test]
        fn it_matches_a_fixed_string_case_insensitively() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[9];

            let result = comment.matches("Pathfinder");
            assert!(result, "'Pathfinder' not found in text");

            let result = comment.matches("pathfinder");
            assert!(result, "'pathfinder' not found in text");
        }

        #[test]
        fn it_matches_a_fixed_string_with_a_space() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[9];
            let result = comment.matches("see eye to eye");
            assert!(result, "{result} != true");
        }

        #[test]
        fn it_does_not_match_a_fixed_string() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[9];
            let result = comment.matches("D&D");
            assert!(!result, "{result} != false");
        }

        #[test]
        fn it_returns_its_creation_time() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[9];
            let datetime = DateTime::parse_from_rfc3339("2025-03-27T05:47:09+00:00")
                .unwrap()
                .with_timezone(&Utc);
            assert_eq!(comment.created_utc(), datetime);
        }

        #[test]
        fn it_returns_its_creation_time_in_local_time() {
            let comments = Comment::parse(&load_data("comments_mipadi")).unwrap();
            let comment = &comments[9];
            let datetime = DateTime::parse_from_rfc3339("2025-03-27T05:47:09+00:00")
                .unwrap()
                .with_timezone(&Local);
            assert_eq!(comment.created_local(), datetime);
        }

        #[test]
        fn it_returns_an_empty_collection() {
            let comments = Comment::parse(&load_data("comments_empty")).unwrap();
            assert!(comments.is_empty());
        }
    }

    mod submissions {
        use super::super::*;
        use crate::test_utils::load_data;

        #[test]
        fn it_cannot_parse_invalid_data() {
            let submissions = Submission::parse(&load_data("submitted_404"));
            assert!(submissions.is_none(), "should be None, was {submissions:?}");
        }

        #[test]
        fn it_can_parse_valid_data() {
            let submissions = Submission::parse(&load_data("submitted_mipadi"));
            assert!(submissions.is_some());
        }

        #[test]
        fn it_can_parse_empty_data() {
            let submissions = Submission::parse(&load_data("submitted_empty"));
            assert!(submissions.is_some());
        }

        #[test]
        fn it_parses_fields() {
            let submissions = Submission::parse(&load_data("submitted_mipadi")).unwrap();
            assert_eq!(submissions.len(), 100);

            let submission = &submissions[0];
            let expected_created_utc = DateTime::from_timestamp(1736196841, 0).unwrap();
            assert_eq!(submission.id, "1hv9k9l");
            assert_eq!(submission.name, "t3_1hv9k9l");
            assert_eq!(
                submission.permalink,
                "/r/rpg/comments/1hv9k9l/collections_coinage_and_the_tyranny_of_fantasy/"
            );
            assert_eq!(submission.author, "mipadi");
            assert_eq!(submission.domain, "acoup.blog");
            assert_eq!(submission.subreddit_id, "t5_2qh2s");
            assert_eq!(submission.subreddit, "rpg");
            assert_eq!(
                submission.url,
                "https://acoup.blog/2025/01/03/collections-coinage-and-the-tyranny-of-fantasy-gold/"
            );
            assert_eq!(
                submission.title,
                "Collections: Coinage and the Tyranny of Fantasy \"Gold\""
            );
            assert_eq!(submission.selftext, "");
            assert_eq!(submission.created_utc, expected_created_utc);
            assert_eq!(
                submission.created_utc.to_rfc2822(),
                "Mon, 6 Jan 2025 20:54:01 +0000"
            );
            assert_eq!(
                submission.created_utc.to_rfc3339(),
                "2025-01-06T20:54:01+00:00"
            );
            assert_eq!(submission.num_comments, 22);
            assert_eq!(submission.ups, 60);
            assert_eq!(submission.downs, 0);
            assert_eq!(submission.score, 60);
        }

        #[test]
        fn it_parses_fields_of_self_posts() {
            let submissions = Submission::parse(&load_data("submitted_mipadi")).unwrap();
            assert_eq!(submissions.len(), 100);

            let expected_selftext = "I have two types of technology upgrades available for my \
                exosuit: items listed as _protection units_, and items listed as _protection \
                upgrades_. The ones listed as upgrades have text that generally says something \
                like \"an almost total rework of the &lt;damage type&gt; Protection, this upgrade \
                brings unparalleled improvements to &lt;damage type&gt; Shielding and &lt;damage \
                type&gt; Protection\", whereas the upgrade units give a percentage of resistance.\
                \n\nShould I install both, or do I just need to install one or the other? For \
                example:\n\n- I have a \"High-Energy Bio-Integrity Unit\" which is a _protection \
                upgrade_, and I can build a \"Radiation Reflector\" which is a _protection unit_. \
                Should I install both?\n- I have a \"Specialist De-Toxifier\" and I can build a \
                \"Toxin Suppressor\". Should I install both?\n- I have a \"Carbon Sublimation \
                Pump\" and I can build a \"Coolant Network\". Should I install both?\n- I have a \
                \"Nitroged-Based Thermal Stabilizer\" and I can build a \"Thermic Layer\". Should \
                I install both?\n\nAnd then for something similar but a little different:\n\n- I \
                have a \"Deep Water Depth Protection\" which says it is an \"almost total rework \
                of the Aeration Membrance\", and I can also build an Aeration Membrane. Will \
                crafting and installing an Aeration Membrane bring any extra benefits?";
            let expected_created_utc = DateTime::from_timestamp(1721503204, 0).unwrap();

            let submission = &submissions[3];
            assert_eq!(submission.id, "1e83c2w");
            assert_eq!(submission.name, "t3_1e83c2w");
            assert_eq!(
                submission.permalink,
                "/r/NoMansSkyTheGame/comments/1e83c2w/should_i_install_both_protection_upgrades_and/"
            );
            assert_eq!(submission.author, "mipadi");
            assert_eq!(submission.domain, "self.NoMansSkyTheGame");
            assert_eq!(submission.subreddit_id, "t5_325lr");
            assert_eq!(submission.subreddit, "NoMansSkyTheGame");
            assert_eq!(
                submission.url,
                "https://www.reddit.com/r/NoMansSkyTheGame/comments/1e83c2w/should_i_install_both_protection_upgrades_and/"
            );
            assert_eq!(
                submission.title,
                "Should I install both protection upgrades and protection units in an exosuit?"
            );
            assert_eq!(submission.selftext, expected_selftext);
            assert_eq!(submission.created_utc, expected_created_utc);
            assert_eq!(
                submission.created_utc.to_rfc2822(),
                "Sat, 20 Jul 2024 19:20:04 +0000"
            );
            assert_eq!(
                submission.created_utc.to_rfc3339(),
                "2024-07-20T19:20:04+00:00"
            );
            assert_eq!(submission.num_comments, 7);
            assert_eq!(submission.ups, 1);
            assert_eq!(submission.downs, 0);
            assert_eq!(submission.score, 1);
        }

        #[test]
        fn it_returns_its_subreddit() {
            let submissions = Submission::parse(&load_data("submitted_mipadi")).unwrap();
            let submission = &submissions[0];
            assert_eq!(submission.subreddit(), "rpg");
        }

        #[test]
        fn it_returns_its_permalink() {
            let submissions = Submission::parse(&load_data("submitted_mipadi")).unwrap();
            let submission = &submissions[0];
            let expected = "https://www.reddit.com/r/rpg/comments/1hv9k9l/collections_coinage_and_the_tyranny_of_fantasy/";
            assert_eq!(submission.permalink(), expected);
        }

        #[test]
        fn it_returns_its_title() {
            let submissions = Submission::parse(&load_data("submitted_mipadi")).unwrap();
            let submission = &submissions[0];
            let expected = "Collections: Coinage and the Tyranny of Fantasy \"Gold\"";
            assert_eq!(submission.title(), expected);
        }

        #[test]
        #[ignore]
        fn it_converts_html_entities_in_its_title() {
            todo!("find or create post data with &gt; or &amp; in title");
        }

        #[test]
        fn it_returns_its_url() {
            let submissions = Submission::parse(&load_data("submitted_mipadi")).unwrap();
            let submission = &submissions[0];
            let expected = "https://acoup.blog/2025/01/03/collections-coinage-and-the-tyranny-of-fantasy-gold/";
            assert_eq!(submission.url(), expected);
        }

        #[test]
        fn it_returns_true_if_it_is_a_self_post() {
            let submissions = Submission::parse(&load_data("submitted_mipadi")).unwrap();
            let submission = &submissions[3];
            assert!(submission.is_self());
        }

        #[test]
        fn it_returns_false_if_it_is_a_self_post() {
            let submissions = Submission::parse(&load_data("submitted_mipadi")).unwrap();
            let submission = &submissions[0];
            assert!(!submission.is_self());
        }

        #[test]
        fn it_returns_its_creation_time() {
            let submissions = Submission::parse(&load_data("submitted_mipadi")).unwrap();
            let submission = &submissions[0];
            let expected = DateTime::parse_from_rfc3339("2025-01-06T20:54:01+00:00")
                .expect("could not parse datetime string");
            assert_eq!(submission.created_utc(), expected);
        }

        #[test]
        fn it_returns_an_empty_collection() {
            let submissions = Submission::parse(&load_data("submitted_empty")).unwrap();
            assert!(submissions.is_empty());
        }
    }
}
