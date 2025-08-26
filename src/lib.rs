// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

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
//! Summarize a user's last 100 comments and provide a tone and sentiment analysis
//! using AI:
//!
//! ```bash
//! usaidwat summary reddit_user
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
//!
//! # OpenAI API Setup
//!
//! To use the summarization feature provided by `usaidwat summary` and the
//! [`Summarizer`](summary::Summarizer), you must set up access to OpenAI.
//! To enable access:
//!
//! 1. Set up an [OpenAI API account].
//! 2. Generate an [API key].
//! 3. Copy and paste the generated key.
//! 4. Store the generated key in your shell's `$OPENAI_API_KEY` environment
//!    variable. Follow your shell's procedure for configuring environment
//!    variables, but generally this involves running
//!
//!    ```bash
//!    $ export OPENAI_API_KEY='copied api key'
//!    ```
//!
//!    In your shell session or in your shell's configuration ("rc") file
//!    (e.g., `~/.bashrc` or `~/.zshrc`).
//!
//! **You are solely responsible for the cost of your use of the OpenAI API!**
//! See the [openai module documentation] for more information on the cost of
//! using the OpenAI API.
//!
//! By default, `usaidwat summary` will use the [cheapest model]; see
//! `usaidwat summary -h` for other options.
//!
//! Currently only OpenAI's API is supported by usaidwat, but support for additional
//! providers may be added in the future.
//!
//! # License
//!
//! usaidwat is licensed under the terms of the [Apache License 2.0]. Please
//! see the LICENSE file accompanying this source code or visit the previous
//! link for more information on licensing.
//!
//! [Apache License 2.0]: https://www.apache.org/licenses/LICENSE-2.0
//! [API key]: https://platform.openai.com/settings/organization/api-keys
//! [OpenAI API account]: https://platform.openai.com/docs/overview
//! [cheapest model]: crate::ai::client::AIModel::cheapest()
//! [openai module documentation]: crate::ai::client::openai

pub mod ai;
pub mod cli;
pub mod clock;
pub mod count;
pub mod filter;
pub mod http;
pub mod markdown;
pub mod pager;
pub mod reddit;
pub mod summary;
pub mod text;
pub mod thing;
pub mod view;

#[cfg(test)]
mod test_utils;
