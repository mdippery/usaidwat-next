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
//!
//! # License
//!
//! usaidwat is licensed under the terms of the [Apache License 2.0]. Please
//! see the LICENSE file accompanying this source code for visit the previous
//! link for more information on licensing.
//!
//! [Apache License 2.0]: https://www.apache.org/licenses/LICENSE-2.0

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
