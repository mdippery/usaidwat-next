//! usaidwat is a command-line tool for quickly listing Redditor's comments
//! and posts in the terminal. It finds the last 100 comments or posts a user
//! has made and presents them as pageable text. It can also tally a user's
//! comments or posts, showing a breakdown of the user's last 100 comments or
//! posts by subreddit.
//!
//! # Examples
//!
//! (In all examples, replace `reddit_user` with the actual username of a
//! Redditor.)
//!
//! Display a user's last 100 comments:
//!
//! ```bash
//! usaidwat log reddit_user
//! ```
//!
//! Show a count of the user's last 100 comments by subreddit:
//!
//! ```bash
//! usaidwat tally reddit_user
//! ```
//!
//! Display a user's last 100 submissions:
//!
//! ```bash
//! usaidwat posts log reddit_user
//! ```
//!
//! Show a count of a user's last 100 submissions by subreddit:
//!
//! ```bash
//! usaidwat posts tally reddit_user
//! ```
//!
//! Show information about a Redditor, such as the age of their account and
//! total karma breakdown:
//!
//! ```bash
//! usaidwat info reddit_user
//! ```
//!
//! Show a breakdown of which hours and days of the week a Redditor has
//! commented:
//!
//! ```bash
//! usaidwat timeline reddit_user
//! ```
//!
//! Get usage and help for the tool:
//!
//! ```bash
//! usaidwat --help
//! ```

pub mod cli;
pub mod client;
pub mod clock;
pub mod conf;
pub mod count;
pub mod filter;
pub mod markdown;
pub mod service;
pub mod text;
pub mod thing;
pub mod view;

#[cfg(test)]
mod test_utils;
