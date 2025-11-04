// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! Drives the command-line program.

use crate::clock::SystemClock;
use crate::count::{SortAlgorithm, SubredditCounter};
use crate::filter::{RedditFilter, StringSet};
use crate::reddit::Redditor;
pub use crate::reddit::client::Error;
use crate::summary::Summarizer;
use crate::view::{ViewOptions, Viewable};
use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::Verbosity;
use cogito::AIModel;
use cogito::client::{AIClient, AIRequest};
use cogito::service::{Auth, Service};
use cogito_openai::OpenAIModel;
use cogito_openai::client::OpenAIClient;
use hypertyper::HTTPClientFactory;
use indoc::formatdoc;
use log::{debug, info, trace};
use std::time::Instant;
use std::{fmt, result};
use tokio_pager::{Pager, PagerEnv};

/// Result of running a command.
pub type Result = result::Result<(), String>;

const AFTER_HELP: &str = include_str!("help/after.txt");

fn after_summary_help<T>() -> String
where
    T: AIModel + fmt::Display,
{
    let flagship: T = AIModelClass::Flagship.model();
    let best: T = AIModelClass::Best.model();
    let cheapest: T = AIModelClass::Cheapest.model();
    let fastest: T = AIModelClass::Fastest.model();
    formatdoc! {
        "[4mAvailable models:[24m
          flagship    {flagship}
          best        {best}
          cheapest    {cheapest}
        * fastest     {fastest}"
    }
}

fn after_summary_help_long<T>() -> String
where
    T: AIClient,
    <T::AIRequest as AIRequest>::Model: AIModel + fmt::Display,
{
    let short_help = after_summary_help::<<T::AIRequest as AIRequest>::Model>();
    let prompt = textwrap::fill(
        &Summarizer::<T>::default_instructions(),
        textwrap::termwidth(),
    );
    formatdoc! {
        "[4mPrompt:[24m
        {prompt}

        {short_help}"
    }
}

/// Program configuration.
#[derive(Debug, Parser)]
#[command(version)]
#[command(about = "Answers the age-old question, \"Where does a Redditor comment the most?\"", long_about = None)]
#[command(after_help = AFTER_HELP)]
pub struct Config {
    #[command(flatten)]
    verbosity: Verbosity,

    #[command(subcommand)]
    command: Command,
}

impl Config {
    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    pub fn username(&self) -> String {
        String::from(self.command.username())
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Retrieve a user's account information
    Info {
        /// Reddit username
        username: String,
    },

    /// Display a user's comments
    #[clap(visible_alias = "l")]
    Log {
        /// Reddit username
        username: String,

        /// Only show comments from these subreddits
        subreddits: Vec<String>,

        /// Show dates in "absolute" or "relative" format
        #[arg(long, value_name = "FORMAT", default_value_t)]
        date: DateFormat,

        /// Show only comments matching STRING
        #[arg(long, value_name = "STRING")]
        grep: Option<String>,

        /// Only show 'n' comments
        #[arg(short = 'n', long)]
        limit: Option<u32>,

        /// Output log in a more compact form
        #[arg(long, default_value_t = false)]
        oneline: bool,

        /// Print raw comment bodies
        #[arg(long, default_value_t = false)]
        raw: bool,
    },

    /// Display a user's submitted posts
    Posts(PostCommandConfig),

    /// Summarize a user's posting history
    #[clap(visible_alias = "summarize")]
    #[clap(visible_alias = "s")]
    #[clap(
        after_help = after_summary_help::<OpenAIModel>(),
        after_long_help = after_summary_help_long::<OpenAIClient<Service>>(),
    )]
    Summary {
        /// Reddit username
        username: String,

        /// Use this AI model for summarization
        #[arg(short = 'm', long, default_value_t)]
        model: AIModelClass,
    },

    /// Tally a user's comments by subreddit
    #[clap(visible_alias = "t")]
    Tally(TallyConfig),

    /// Display user's activity by day of week and hour
    Timeline {
        /// Reddit username
        username: String,
    },
}

impl Command {
    pub fn username(&self) -> &str {
        match &self {
            Command::Info { username } => username,
            Command::Log { username, .. } => username,
            Command::Posts(subconfig) => subconfig.command.username(),
            Command::Summary { username, .. } => username,
            Command::Tally(TallyConfig { username, .. }) => username,
            Command::Timeline { username } => username,
        }
    }
}

#[derive(Args, Debug)]
struct TallyConfig {
    /// Reddit username
    username: String,

    /// Sort output by number of comments instead of alphabetically by subreddit
    #[arg(short = 'c', long = "count", default_value_t = false)]
    sort_by_count: bool,
}

impl TallyConfig {
    fn sort_algorithm(&self) -> SortAlgorithm {
        if self.sort_by_count {
            SortAlgorithm::Numerically
        } else {
            SortAlgorithm::Lexicographically
        }
    }
}

#[derive(Args, Debug)]
struct PostCommandConfig {
    #[command(subcommand)]
    command: PostSubcommand,
}

#[derive(Debug, Subcommand)]
enum PostSubcommand {
    /// Show a user's submitted posts
    Log {
        /// Reddit username
        username: String,

        // Only show posts from these subreddits
        subreddits: Vec<String>,

        /// Output log in a more compact form
        #[arg(long, default_value_t = false)]
        oneline: bool,

        /// Show dates in "absolute" or "relative" format
        #[arg(long, value_name = "FORMAT", default_value_t)]
        date: DateFormat,
    },

    /// Tally a user's posts by subreddit
    Tally(TallyConfig),
}

impl PostSubcommand {
    pub fn username(&self) -> &str {
        match &self {
            PostSubcommand::Log { username, .. } => username,
            PostSubcommand::Tally(TallyConfig { username, .. }) => username,
        }
    }
}

/// Determines if dates should be displayed as an absolute date ("January 1, 2025")
/// or relative to the current time ("5 months ago").
#[derive(Clone, Debug, Default, PartialEq, ValueEnum)]
pub enum DateFormat {
    /// Display dates as an absolute date ("January 1, 2025").
    Absolute,

    /// Display dates relative to the current time ("5 months ago").
    #[default]
    Relative,
}

impl fmt::Display for DateFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateFormat::Absolute => write!(f, "absolute"),
            DateFormat::Relative => write!(f, "relative"),
        }
    }
}

/// Determines the qualities of the AI model used for summarization.
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum AIModelClass {
    /// Use the AI service's flagship model.
    ///
    /// This is the model that a particular AI service promotes as their
    /// standard or "default" model.
    Flagship,

    /// Use the AI service's "best" model.
    ///
    /// Each AI provider determines what it calls its "best" model, but
    /// generally it is one that provides the best price/performance
    /// ratio.
    Best,

    /// Use the AI service's least expensive model.
    Cheapest,

    /// Use the AI service's fastest model.
    #[default]
    Fastest,
}

impl fmt::Display for AIModelClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AIModelClass::Flagship => write!(f, "flagship"),
            AIModelClass::Best => write!(f, "best"),
            AIModelClass::Cheapest => write!(f, "cheapest"),
            AIModelClass::Fastest => write!(f, "fastest"),
        }
    }
}

impl AIModelClass {
    /// Returns the AI model corresponding to the model selected by the
    /// command-line flags.
    pub fn model<T: AIModel>(&self) -> T {
        match self {
            AIModelClass::Flagship => T::flagship(),
            AIModelClass::Best => T::best(),
            AIModelClass::Cheapest => T::cheapest(),
            AIModelClass::Fastest => T::fastest(),
        }
    }
}

/// Runs the command-line program.
#[derive(Debug)]
pub struct Runner {
    config: Config,
    user: Redditor,
}

impl Runner {
    /// Create a new program runner using the given `config`.
    ///
    /// Returns an error with a helpful message if the user does not exist.
    pub async fn new(config: Config) -> result::Result<Runner, Error> {
        let username = config.command.username();
        let user = Redditor::new(username).await?;
        Ok(Self { config, user })
    }

    fn user(&self) -> &Redditor {
        &self.user
    }

    /// Run the command-line program using its stored configuration options.
    pub async fn run(&self) -> Result {
        match &self.config.command {
            Command::Info { .. } => self.run_info(),
            Command::Log {
                subreddits,
                date,
                grep,
                limit,
                oneline,
                raw,
                ..
            } => {
                self.run_log(subreddits, date, grep, limit, oneline, raw)
                    .await
            }
            Command::Posts(subconfig) => self.run_posts(subconfig).await,
            Command::Summary { model, .. } => self.run_summary(model).await,
            Command::Tally(config) => self.run_tally(&config.sort_algorithm()),
            Command::Timeline { .. } => self.run_timeline(),
        }
    }

    fn run_info(&self) -> Result {
        println!(
            "{}",
            self.user()
                .view(&ViewOptions::default(), &SystemClock::default())
        );
        Ok(())
    }

    async fn run_log(
        &self,
        subreddits: &Vec<String>,
        date_format: &DateFormat,
        grep: &Option<String>,
        limit: &Option<u32>,
        oneline: &bool,
        raw: &bool,
    ) -> Result {
        let opts = ViewOptions::default()
            .oneline(*oneline)
            .raw(*raw)
            .grep(grep.clone())
            .date_format(date_format.clone());

        let filter = StringSet::from(subreddits).ok_or(format!(
            "invalid subreddit filter: {}",
            subreddits.join(" ")
        ))?;

        let comments = RedditFilter::new(self.user().comments())
            .take(limit)
            .grep(grep)
            .filter(&filter)
            .collect();

        let joiner = if *oneline { "\n" } else { "\n\n\n" };
        let output = comments
            .iter()
            .map(|comment| comment.view(&opts, &SystemClock::default()))
            .collect::<Vec<_>>()
            .join(joiner);

        Pager::new(PagerEnv::default().oneline(*oneline))
            .page(&output)
            .await
    }

    async fn run_posts(&self, config: &PostCommandConfig) -> Result {
        match &config.command {
            PostSubcommand::Log {
                subreddits,
                date,
                oneline,
                ..
            } => Ok(self.run_posts_log(subreddits, date, oneline).await?),
            PostSubcommand::Tally(config) => self.run_posts_tally(&config.sort_algorithm()),
        }
    }

    async fn run_posts_log(
        &self,
        subreddits: &Vec<String>,
        date_format: &DateFormat,
        oneline: &bool,
    ) -> Result {
        let opts = ViewOptions::default()
            .oneline(*oneline)
            .date_format(date_format.clone());

        let filter = StringSet::from(subreddits).ok_or(format!(
            "invalid subreddit filter: {}",
            subreddits.join(" ")
        ))?;

        let posts = RedditFilter::new(self.user().submissions())
            .filter(&filter)
            .collect();

        let joiner = if *oneline { "\n" } else { "\n\n\n" };
        let output = posts
            .iter()
            .map(|post| post.view(&opts, &SystemClock::default()))
            .collect::<Vec<_>>()
            .join(joiner);

        Pager::new(PagerEnv::default().oneline(*oneline))
            .page(&output)
            .await
    }

    fn run_posts_tally(&self, sort_algorithm: &SortAlgorithm) -> Result {
        // TODO: Need to test this conditional logic

        if self.user().has_submissions() {
            let posts = self.user().submissions();
            let tallies = SubredditCounter::from_iter(posts).sort_by(sort_algorithm);
            println!(
                "{}",
                tallies.view(&ViewOptions::default(), &SystemClock::default())
            );
            Ok(())
        } else {
            println!("{} has no posts.", self.user().username());
            Ok(())
        }
    }

    async fn run_summary(&self, model: &AIModelClass) -> Result {
        let auth =
            Auth::from_env("OPENAI_API_KEY").map_err(|_| include_str!("help/summary.txt"))?;

        let factory = HTTPClientFactory::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        let client = OpenAIClient::new(auth, factory);

        let summarizer = Summarizer::new(client, self.user());
        info!("Instructions:\n{}", summarizer.instructions());
        debug!("Summarization output:\n{}", summarizer.context());

        let model = model.model();
        debug!("Using model: {:?} - {}", model, model);

        // TODO: Should we return raw JSON here in debug mode?

        // TODO: Track timing in Summarizer and print stats in debug or trace mode
        // I think I can use https://docs.rs/timethis for this.
        // Except I want to return the value of summarize() as well, so maybe I'll
        // have to write my own macro or function returns a 2-tuple (duration, value),
        // or just add it to the Summarize class itself (but more general would be
        // better).

        let now = Instant::now();
        let output = summarizer
            .model(model)
            .summarize()
            .await
            .map_err(|err| format!("Error in API request: {err}"))?;
        let elapsed = now.elapsed();
        trace!("Summarization time: {:.4} secs", elapsed.as_secs_f64());
        let output = textwrap::fill(&output, textwrap::termwidth());
        Pager::new(PagerEnv::default()).page(&output).await
    }

    fn run_tally(&self, sort_algorithm: &SortAlgorithm) -> Result {
        // TODO: Need to test this conditional logic

        if self.user.has_comments() {
            let comments = self.user().comments();
            let tallies = SubredditCounter::from_iter(comments).sort_by(sort_algorithm);
            println!(
                "{}",
                tallies.view(&ViewOptions::default(), &SystemClock::default())
            );
            Ok(())
        } else {
            println!("{} has no comments.", self.user().username());
            Ok(())
        }
    }

    fn run_timeline(&self) -> Result {
        // TODO: This is hard to test -- should move the conditional check
        //       into testable method, maybe Timeline::view(), although I'm
        //       not sure the logic is appropriate there, either.

        if self.user().has_comments() {
            println!(
                "{}",
                self.user()
                    .timeline()
                    .view(&ViewOptions::default(), &SystemClock::default())
            );
            Ok(())
        } else {
            println!("{} has no comments.", self.user().username());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    mod ai_model_class {
        use super::super::AIModelClass;
        use cogito::AIModel;
        use cogito_openai::OpenAIModel;
        use paste::paste;

        macro_rules! model_tests {
            ( $( $name:ident ),* ) => {
                paste! {
                    $(
                        #[test]
                        fn [<it_selects_the_ $name:snake _model>]() {
                            let flag = AIModelClass::$name;
                            let model: OpenAIModel = flag.model();
                            assert_eq!(model, OpenAIModel::[<$name:snake>]());
                        }
                    )*
                }
            }
        }

        model_tests!(Flagship, Best, Cheapest, Fastest);
    }
}
