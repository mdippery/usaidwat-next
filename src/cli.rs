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
use std::process;

// TODO: We should probably move this back to main and have Runner.run()
//       return a Result, but we can work on that later.
pub fn die(error_code: i32, message: &str) {
    eprintln!("{}", message);
    process::exit(error_code);
}

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
        #[arg(long)]
        // TODO: The default option to DateFormat should avoid having to
        //       use Option here, but that's not working for some reason.
        date: Option<DateFormat>,

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

        /// Output log in a more compact form
        #[arg(long, default_value_t = false)]
        oneline: bool,
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
    pub fn run(&self) {
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
                let date_format = date.as_ref().unwrap_or(&DateFormat::Relative);
                self.run_log(subreddits, date_format, grep, limit, oneline, raw);
            }
            Command::Posts(subconfig) => self.run_posts(subconfig),
            Command::Summary { .. } => self.run_summary(),
            Command::Tally(config) => self.run_tally(&config.sort_algorithm()),
            Command::Timeline { .. } => self.run_timeline(),
        }
    }

    fn run_info(&self) {
        println!(
            "{}",
            self.user()
                .view(&ViewOptions::default(), &SystemClock::default())
        );
    }

    fn run_log(
        &self,
        subreddits: &Vec<String>,
        date_format: &DateFormat,
        grep: &Option<String>,
        limit: &Option<u32>,
        oneline: &bool,
        raw: &bool,
    ) {
        let opts = ViewOptions::default()
            .oneline(*oneline)
            .raw(*raw)
            .grep(grep.clone())
            .date_format(date_format.clone());

        // TODO: Move this take into the filtering code
        let n = limit
            .and_then(|n| Some(n as usize))
            .unwrap_or_else(|| self.user().comments().count());
        let comments = self.user().comments().take(n);

        // TODO: Wrap this up in a common method for reuse by `posts log`.
        let subreddits: Vec<&str> = subreddits.iter().map(|s| s.as_str()).collect();
        let filter = StringSet::from(&subreddits);
        if filter.is_none() {
            die(
                1,
                &format!("invalid subreddit filter: {}", subreddits.join(" ")),
            );
        }
        // TODO: Might be a better way to do this, but at this point we should know it's Some.
        let filter = filter.unwrap();

        let comments = RedditFilter::new(comments)
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
    }

    fn run_posts(&self, config: &PostCommandConfig) {
        match &config.command {
            PostSubcommand::Log { oneline, .. } => self.run_posts_log(&oneline),
            PostSubcommand::Tally(config) => self.run_posts_tally(&config.sort_algorithm()),
        }
    }

    fn run_posts_log(&self, oneline: &bool) {
        let opts = ViewOptions::default().oneline(*oneline);
        let posts = self.user().submissions();

        let joiner = if *oneline { "\n" } else { "\n\n\n" };
        let output = posts
            .map(|post| post.view(&opts, &SystemClock::default()))
            .collect::<Vec<_>>()
            .join(joiner);

        Pager::new().pager_envs(conf::pager_env(oneline)).setup();
        println!("{}", output);
    }

    fn run_posts_tally(&self, sort_algorithm: &SortAlgorithm) {
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
        } else {
            println!("{} has no posts.", self.user().username());
        }
    }

    fn run_summary(&self) {
        todo!("summary");
    }

    fn run_tally(&self, sort_algorithm: &SortAlgorithm) {
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
        } else {
            println!("{} has no comments.", self.user().username());
        }
    }

    fn run_timeline(&self) {
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
        } else {
            println!("{} has no comments.", self.user().username());
        }
    }
}
