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

//! Asynchronous, Tokio-friendly pager implementation.
//!
//! Unfortunately, Rust's [pager] crate does not play nicely with Tokio.
//! It leaves threads open after the Tokio runtime exits, resulting in a
//! nasty I/O error after a CLI program using both pager and a Tokio runtime
//! exits. This may be due to the fact that the pager crate actually runs the
//! pager in the _parent_ process, meaning that the Tokio runtime, in the
//! child process, exits before the pager, leaving dangling file descriptors
//! and the aforementioned I/O error from Tokio.
//!
//! Unfortunately, there isn't a great away to customize the behavior of
//! the pager crate, so this module implements a [`Pager`] struct that
//! allows the use of a pager subprocess in a way that plays nicely with
//! Tokio.
//!
//! [pager]: https://crates.io/crates/pager

use atty::Stream;
use std::process::{ExitStatus, Stdio};
use std::{env, io, result};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// An environmental variable consisting of a name-value pair.
pub type EnvVar = (String, String);

/// A tokio-friendly asynchronous pager.
#[derive(Debug)]
pub struct Pager {
    pager_env: PagerEnv,
}

impl Pager {
    /// Creates a new pager from the pager env.
    pub fn new(pager_env: PagerEnv) -> Self {
        Self { pager_env }
    }

    /// The program used for pagination.
    ///
    /// # Examples
    ///
    /// ```
    /// use usaidwat::pager::{Pager, PagerEnv};
    /// # use temp_env::with_var_unset;
    /// # with_var_unset("LESS", || {
    /// let command = Pager::new(PagerEnv::default()).command();
    /// assert_eq!(command, "/usr/bin/less");
    /// # });
    /// ```
    pub fn command(&self) -> String {
        self.pager_env.pager()
    }

    /// Name and value of the environment variable used to control aspects
    /// of the pager's behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use usaidwat::pager::{Pager, PagerEnv};
    /// # use temp_env::with_var_unset;
    /// # with_var_unset("LESS", || {
    /// let (name, value) = Pager::new(PagerEnv::default()).env().unwrap();
    /// assert_eq!(name, "LESS");
    /// assert_eq!(value, "FSRX");
    /// # });
    /// ```
    pub fn env(&self) -> Option<EnvVar> {
        self.pager_env.pager_env()
    }

    /// True if the pager is `cat`.
    ///
    /// # Examples
    ///
    /// Returns true if `$PAGER` is `cat`:
    ///
    /// ```
    /// # use usaidwat::pager::{Pager, PagerEnv};
    /// # use temp_env::with_var;
    /// # with_var("PAGER", Some("cat"), || {
    /// // $PAGER == "cat"
    /// let is_cat = Pager::new(PagerEnv::default()).is_cat();
    /// assert!(is_cat);
    /// # });
    /// ```
    ///
    /// Or if `$PAGER` is `/usr/bin/cat`, or anything ending in `cat`:
    ///
    /// ```
    /// # use usaidwat::pager::{Pager, PagerEnv};
    /// # use temp_env::with_var;
    /// # with_var("PAGER", Some("/usr/bin/cat"), || {
    /// // $PAGER == "/usr/bin/cat"
    /// let is_cat = Pager::new(PagerEnv::default()).is_cat();
    /// assert!(is_cat);
    /// # });
    ///
    ///
    /// # with_var("PAGER", Some("/bin/cat"), || {
    /// // $PAGER == "/bin/cat"
    /// let is_cat = Pager::new(PagerEnv::default()).is_cat();
    /// assert!(is_cat);
    /// # });
    /// ```
    ///
    /// But it returns false if `$PAGER` is something else:
    ///
    /// ```
    /// # use usaidwat::pager::{Pager, PagerEnv};
    /// # use temp_env::with_var;
    /// # with_var("PAGER", Some("less"), || {
    /// // $PAGER == "less"
    /// let is_cat = Pager::new(PagerEnv::default()).is_cat();
    /// assert!(!is_cat);
    /// # });
    /// ```
    pub fn is_cat(&self) -> bool {
        self.command() == "cat" || self.command().ends_with("/cat")
    }

    /// True if stdout is a tty.
    pub fn is_tty(&self) -> bool {
        atty::is(Stream::Stdout)
    }

    /// Pages the output to the pager.
    ///
    /// Returns the exit status of the child pager process.
    pub async fn page_to_pager_with_error(
        &self,
        output: impl AsRef<str>,
    ) -> io::Result<ExitStatus> {
        // TODO: Skip paging if pager == "cat"
        // TODO: Skip paging it not outputting to a tty

        let mut command = Command::new(&self.command());

        if let Some((key, value)) = &self.env() {
            command.env(key, value);
        }

        let mut command = command.stdin(Stdio::piped()).spawn().map_err(|e| {
            let message = format!("failed to spawn pager: {e}");
            io::Error::new(io::ErrorKind::Other, message)
        })?;

        if let Some(mut stdin) = command.stdin.take() {
            stdin.write_all(output.as_ref().as_bytes()).await?;
        }

        command.wait().await
    }

    /// Pages output to stdout instead of a separate pager process.
    pub async fn page_to_stdout_with_error(
        &self,
        output: impl AsRef<str>,
    ) -> io::Result<ExitStatus> {
        let output = output.as_ref();
        println!("{}", output);
        Ok(ExitStatus::default())
    }

    /// Pages the output and returns an I/O result.
    ///
    /// The output will be sent to a separate pager process as defined by
    /// `$PAGER`, unless `$PAGER` is `cat`, in which case the output will
    /// simply be sent to stdout.
    pub async fn page_with_error(&self, output: impl AsRef<str>) -> io::Result<ExitStatus> {
        if self.is_cat() || !self.is_tty() {
            self.page_to_stdout_with_error(output).await
        } else {
            self.page_to_pager_with_error(output).await
        }
    }

    /// Pages the output to the pager.
    ///
    /// If the `$PAGER` is `cat` or any variant like `/usr/bin/cat`, the output
    /// will be sent directly to stdout instead of to a separate paging process.
    ///
    /// If no errors occur, `()` is returned; otherwise, a string describing
    /// the error is returned.
    pub async fn page(&self, output: impl AsRef<str>) -> Result {
        let status = self
            .page_with_error(output)
            .await
            .map_err(|e| format!("I/O error: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            let message = format!("pager {} exited with status {status}", self.command());
            Err(message)
        }
    }
}

/// A result from the pager subprocess.
pub type Result = result::Result<(), String>;

// Pager Environment
// --------------------------------------------------------------------------

// I'm not sure all of this logic really makes sense -- some of it may be
// specific to my own personal preferences -- but let's use this until
// someone complains.
//
// In the Ruby tool, I do, in fact, force "RS" if --oneline is selected,
// similarly to what I do here, so perhaps the logic following the
// retrieval of $LESS should simply be
//
//     let less = if *oneline { "RS" } else { less };
//
// However, since I send ANSI color codes whenever we are hooked up to a
// tty, I definitely want "R" to be included, so if I instead respect
// the user's possible absence of "R", I should make sure I only send
// ANSI color codes when "R" is included in $LESS.
//
// Specifically, the Ruby tool includes this code (spread around the
// codebase, but listed here contiguously for clarity):
//
//    ENV['LESS'] = 'RS' if options[:oneline]
//    ENV['LESS'] = 'FSRX' unless ENV['LESS']
//
// Oy vey.
//
// Also, I should test this with various values of $LESS. For example,
// my $LESS is simply set to "R", but I should test output when the
// default option of "FSRX is used.

/// Retrieves the pager and pager configuration from the environment.
#[derive(Debug, Default)]
pub struct PagerEnv {
    oneline: bool,
}

impl PagerEnv {
    /// The default pager.
    pub const DEFAULT_PAGER: &'static str = "/usr/bin/less";

    /// Controls whether the pager is outputting single-line output or not,
    /// which may alter aspects of its configuration or behavior.
    ///
    /// Pass true if the `--oneline` option was specified on the command line.
    pub fn oneline(self, oneline: bool) -> Self {
        Self { oneline }
    }

    /// Returns a path to the program that should be used for paginating output.
    ///
    /// If a program is not specified in the environment, the
    /// [default pager](PagerEnv::DEFAULT_PAGER) is used.
    pub fn pager(&self) -> String {
        env::var("PAGER").unwrap_or(String::from(PagerEnv::DEFAULT_PAGER))
    }

    /// Returns an appropriate 2-tuple of (environment variable name, value)
    /// to pass to the pager.
    ///
    /// By default, this is `FSRX`, unless the user has defined `$LESS` in the
    /// environment. However, because text is printed in color, `R` is always
    /// included regardless of the value of `$LESS` (it is appended to `$LESS` if
    /// not already present), and when output is printed to oneline (via the
    /// `--oneline` option), `S` is appended to `$LESS` if not already present.
    ///
    /// This ensures that output is pleasant for the user, regardless of the
    /// definition of `$LESS`.
    ///
    /// # Examples
    ///
    /// `pager_env` will return a default value if `$LESS` is not set:
    ///
    /// ```
    /// use usaidwat::pager::PagerEnv;
    /// # use temp_env::with_var_unset;
    /// # with_var_unset("LESS", || {
    /// let (key, value) = PagerEnv::default().pager_env().unwrap();
    /// assert_eq!(key, "LESS");
    /// assert_eq!(value, "FSRX");
    /// # });
    /// ```
    ///
    /// It will include `S` if `oneline` is `true`:
    ///
    /// ```
    /// use usaidwat::pager::PagerEnv;
    /// # use temp_env::with_var_unset;
    /// # with_var_unset("LESS", || {
    /// let (_, value) = PagerEnv::default().oneline(true).pager_env().unwrap();
    /// assert_eq!(value, "FSRX");
    /// # });
    /// ```
    ///
    /// In this example, `$LESS` was set to `SX`, but `R` will be appended anyway:
    ///
    /// ```
    /// use usaidwat::pager::PagerEnv;
    /// # use temp_env::with_var;
    /// # with_var("LESS", Some("SX"), || {
    /// let (_, value) = PagerEnv::default().pager_env().unwrap();
    /// assert_eq!(value, "SXR");
    /// # });
    /// ```
    ///
    /// In this example, `$LESS` was set to `RSX`. Note that `R` is still included,
    /// but `$LESS` was not altered since `R` was already in it:
    ///
    /// ```
    /// use usaidwat::pager::PagerEnv;
    /// # use temp_env::with_var;
    /// # with_var("LESS", Some("RSX"), || {
    /// let (_, value) = PagerEnv::default().pager_env().unwrap();
    /// assert_eq!(value, "RSX");
    /// # });
    /// ```
    ///
    /// In this example, `$LESS` was set to `R`. Because the `oneline` option is
    /// `true`, `S` is also appended:
    ///
    /// ```
    /// use usaidwat::pager::PagerEnv;
    /// # use temp_env::with_var;
    /// # with_var("LESS", Some("R"), || {
    /// let (_, value) = PagerEnv::default().oneline(true).pager_env().unwrap();
    /// assert_eq!(value, "RS");
    /// # });
    /// ```
    ///
    /// In this example, `$LESS` was set to `SR`. Because the `oneline` option is
    /// `true`, `S` is still included, but because it is already present, the
    /// value of `$LESS` does not change:
    ///
    /// ```
    /// use usaidwat::pager::PagerEnv;
    /// # use temp_env::with_var;
    /// # with_var("LESS", Some("SR"), || {
    /// let (_, value) = PagerEnv::default().oneline(true).pager_env().unwrap();
    /// assert_eq!(value, "SR");
    /// # });
    /// ```
    pub fn pager_env(&self) -> Option<EnvVar> {
        // TODO: Generalize for non-LESS pagers

        // Get the value of $LESS, defaulting to "FSRX" if $LESS is unset.
        let less = env::var_os("LESS").unwrap_or(
            "FSRX"
                .parse()
                .expect("could not parse 'FSRX' into OsString"),
        );
        let less = less.to_string_lossy();

        // Always interpret ANSI color escape sequences.
        let less = if !less.contains("R") {
            less + "R"
        } else {
            less
        };

        // When printing to one line, really print to one line, and force scrolling
        // to the right if lines are too long.
        let less = if self.oneline && !less.contains("S") {
            less + "S"
        } else {
            less
        };

        Some((String::from("LESS"), less.to_string()))
    }
}
