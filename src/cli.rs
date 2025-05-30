//! Drives the command-line program.

use crate::client::Redditor;
use crate::clock::SystemClock;
use crate::count::{SortAlgorithm, SubredditCounter};
use crate::filter::Searchable;
use crate::service::RedditService;
use crate::thing::Comment;
use crate::view::{ViewOptions, Viewable};
use clap::{Args, Parser, Subcommand, ValueEnum};
use pager::Pager;
use std::env;
use std::ffi::OsString;

/// Program configuration.
#[derive(Debug, Parser)]
#[command(version)]
#[command(about = "Answers the age-old question, \"Where does a Redditor comment the most?\"", long_about = None)]
pub struct Config {
    #[command(subcommand)]
    command: Command,
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

#[derive(Clone, Debug, Default, PartialEq, ValueEnum)]
pub enum DateFormat {
    Absolute,
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
    /// # Panics
    ///
    /// If the user specified in `config.command` does not exist.
    pub fn new(config: Config) -> Runner {
        let username = config.command.username();
        let user = Redditor::new(username.to_string(), RedditService::new())
            // TODO: Just show message and exit non-zero instead of panicking
            .expect(&format!("no such user: {username}"));
        Runner { config, user }
    }

    fn username(&self) -> String {
        self.user.username()
    }

    fn user(&self) -> &Redditor {
        &self.user
    }

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
    // TODO: Refactor this into testable function.
    fn pager_env(&self, oneline: &bool) -> impl IntoIterator<Item = impl Into<OsString>> {
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
        // to right if lines are too long.
        let less = if *oneline && !less.contains("S") {
            less + "S"
        } else {
            less
        };

        vec![format!("LESS={less}")]
    }

    /// Run the command-line program using its stored configuration options.
    pub fn run(&self) {
        match &self.config.command {
            Command::Info { .. } => self.run_info(),
            Command::Log {
                date,
                grep,
                limit,
                oneline,
                raw,
                ..
            } => {
                let date_format = date.as_ref().unwrap_or(&DateFormat::Absolute);
                self.run_log(date_format, grep, limit, oneline, raw);
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
        date_format: &DateFormat,
        grep: &Option<String>,
        limit: &Option<u32>,
        oneline: &bool,
        raw: &bool,
    ) {
        // TODO: Filter by subreddit

        let opts = ViewOptions::build()
            .oneline(*oneline)
            .raw(*raw)
            .date_format(date_format.clone())
            .build();
        let n = limit
            .and_then(|n| Some(n as usize))
            .unwrap_or_else(|| self.user().comments().count());
        let comments = self.user().comments().take(n);

        // TODO: Really need to extract this filtering out into a module I
        //       can test easily, along with filtering by subreddit.
        let comments: Box<dyn Iterator<Item = Comment>> = match grep {
            Some(grep) => Box::new(comments.filter(move |comment| comment.matches(grep))),
            None => Box::new(comments),
        };

        let joiner = if *oneline { "\n" } else { "\n\n\n" };
        let output = comments
            .map(|comment| comment.view(&opts, &SystemClock::default()))
            .collect::<Vec<_>>()
            .join(joiner);

        Pager::new().pager_envs(self.pager_env(oneline)).setup();
        // TODO: Only output color if hooked up to tty
        // TODO: Highlight matches in output, if grep is specified
        println!("{}", output);
    }

    fn run_posts(&self, config: &PostCommandConfig) {
        match &config.command {
            PostSubcommand::Log { oneline, .. } => self.run_posts_log(&oneline),
            PostSubcommand::Tally(config) => self.run_posts_tally(&config.sort_algorithm()),
        }
    }

    fn run_posts_log(&self, oneline: &bool) {
        // TODO: Support absolute dates
        //       Never did in the Ruby tool but it would be nice to do here.
        // TODO: Filter by subreddit

        let opts = ViewOptions::build().oneline(*oneline).build();
        let posts = self.user().submissions();

        let joiner = if *oneline { "\n" } else { "\n\n\n" };
        let output = posts
            .map(|post| post.view(&opts, &SystemClock::default()))
            .collect::<Vec<_>>()
            .join(joiner);

        Pager::new().pager_envs(self.pager_env(oneline)).setup();
        // TODO: Only output color if hooked up to tty
        println!("{}", output);
    }

    fn run_posts_tally(&self, sort_algorithm: &SortAlgorithm) {
        // TODO: Need to test this conditional logic

        if self.user().has_submissions() {
            let posts = self.user().submissions();
            let tallies = SubredditCounter::from_iter(posts).sort_by(sort_algorithm);
            println!(
                "{}",
                tallies.view(&ViewOptions::default(), &SystemClock::default())
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
                tallies.view(&ViewOptions::default(), &SystemClock::default())
            );
        } else {
            println!("{} has no comments.", self.user().username());
        }
    }

    fn run_timeline(&self) {
        // TODO: This is hard to test -- should move the conditional check
        //       into testable method, maybe Timeline::view(), although I'm
        //       not sure the logic is appropriate there, either.

        // TODO: Print in color with intensity proportional to number of comments

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
