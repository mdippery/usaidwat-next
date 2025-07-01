//! Drives the command-line program.

pub use crate::client::Error;
use crate::client::Redditor;
use crate::clock::SystemClock;
use crate::conf;
use crate::count::{SortAlgorithm, SubredditCounter};
use crate::filter::{RedditFilter, StringSet};
use crate::service::RedditService;
use crate::view::{ViewOptions, Viewable};
use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::Verbosity;
use pager::Pager;
use std::fmt::Formatter;

/// Result of running a command.
pub type CliResult = Result<(), String>;

/// Program configuration.
#[derive(Debug, Parser)]
#[command(version)]
#[command(about = "Answers the age-old question, \"Where does a Redditor comment the most?\"", long_about = None
)]
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
    #[clap(alias = "l")]
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
    Summary {
        /// Reddit username
        username: String,
    },

    /// Tally a user's comments by subreddit
    #[clap(alias = "t")]
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
            Command::Summary { username } => username,
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

impl std::fmt::Display for DateFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DateFormat::Absolute => write!(f, "absolute"),
            DateFormat::Relative => write!(f, "relative"),
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
    pub fn new(config: Config) -> Result<Runner, Error> {
        let username = config.command.username();
        let user = Redditor::new(username.to_string(), RedditService::new())?;
        Ok(Self { config, user })
    }

    fn user(&self) -> &Redditor {
        &self.user
    }

    /// Run the command-line program using its stored configuration options.
    pub fn run(&self) -> CliResult {
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
            } => self.run_log(subreddits, date, grep, limit, oneline, raw),
            Command::Posts(subconfig) => self.run_posts(subconfig),
            Command::Summary { .. } => self.run_summary(),
            Command::Tally(config) => self.run_tally(&config.sort_algorithm()),
            Command::Timeline { .. } => self.run_timeline(),
        }
    }

    fn run_info(&self) -> CliResult {
        println!(
            "{}",
            self.user()
                .view(&ViewOptions::default(), &SystemClock::default())
        );
        Ok(())
    }

    fn run_log(
        &self,
        subreddits: &Vec<String>,
        date_format: &DateFormat,
        grep: &Option<String>,
        limit: &Option<u32>,
        oneline: &bool,
        raw: &bool,
    ) -> CliResult {
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

        Pager::new().pager_envs(conf::pager_env(oneline)).setup();
        println!("{}", output);
        Ok(())
    }

    fn run_posts(&self, config: &PostCommandConfig) -> CliResult {
        match &config.command {
            PostSubcommand::Log {
                subreddits,
                date,
                oneline,
                ..
            } => self.run_posts_log(subreddits, date, &oneline),
            PostSubcommand::Tally(config) => self.run_posts_tally(&config.sort_algorithm()),
        }
    }

    fn run_posts_log(
        &self,
        subreddits: &Vec<String>,
        date_format: &DateFormat,
        oneline: &bool,
    ) -> CliResult {
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

        Pager::new().pager_envs(conf::pager_env(oneline)).setup();
        println!("{}", output);
        Ok(())
    }

    fn run_posts_tally(&self, sort_algorithm: &SortAlgorithm) -> CliResult {
        // TODO: Need to test this conditional logic

        if self.user().has_submissions() {
            let posts = self.user().submissions();
            let tallies = SubredditCounter::from_iter(posts).sort_by(sort_algorithm);
            println!(
                "{}",
                tallies
                    .collect::<Vec<_>>()
                    .view(&ViewOptions::default(), &SystemClock::default())
            );
            Ok(())
        } else {
            println!("{} has no posts.", self.user().username());
            Ok(())
        }
    }

    fn run_summary(&self) -> CliResult {
        todo!("summary");
    }

    fn run_tally(&self, sort_algorithm: &SortAlgorithm) -> CliResult {
        // TODO: Need to test this conditional logic

        if self.user.has_comments() {
            let comments = self.user().comments();
            let tallies = SubredditCounter::from_iter(comments).sort_by(sort_algorithm);
            println!(
                "{}",
                tallies
                    .collect::<Vec<_>>()
                    .view(&ViewOptions::default(), &SystemClock::default())
            );
            Ok(())
        } else {
            println!("{} has no comments.", self.user().username());
            Ok(())
        }
    }

    fn run_timeline(&self) -> CliResult {
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
