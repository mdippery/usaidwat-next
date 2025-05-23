//! Drives the command-line program.

use crate::client::Redditor;
use crate::clock::SystemClock;
use crate::service::RedditService;
use crate::thing::Comment;
use crate::view::{ViewOptions, Viewable};
use clap::{Args, Parser, Subcommand, ValueEnum};

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
    user: Redditor<SystemClock>,
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

    fn user(&self) -> &Redditor<SystemClock> {
        &self.user
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
            Command::Tally(TallyConfig { sort_by_count, .. }) => self.run_tally(sort_by_count),
            Command::Timeline { .. } => self.run_timeline(),
        }
    }

    fn run_info(&self) {
        println!("{}", self.user().view(&ViewOptions::default()));
    }

    fn run_log(
        &self,
        date_format: &DateFormat,
        grep: &Option<String>,
        limit: &Option<u32>,
        oneline: &bool,
        raw: &bool,
    ) {
        let opts = ViewOptions::build()
            .oneline(*oneline)
            .raw(*raw)
            .date_format(date_format.clone())
            .build();
        let n = limit
            .and_then(|n| Some(n as usize))
            .unwrap_or_else(|| self.user().comments().count());
        let comments = self.user().comments().take(n);

        // TODO: Probably need to move this into a function that I can test easily
        let comments: Box<dyn Iterator<Item = Comment>> = match grep {
            Some(grep) => {
                Box::new(comments.filter(move |comment| comment.body().matches(grep).count() > 0))
            }
            None => Box::new(comments),
        };

        let output = comments
            .map(|comment| comment.view(&opts))
            .collect::<Vec<_>>()
            .join("\n\n");

        println!("{}", output);
    }

    fn run_posts(&self, config: &PostCommandConfig) {
        match config.command {
            PostSubcommand::Log { oneline, .. } => self.run_posts_log(&oneline),
            PostSubcommand::Tally(TallyConfig { sort_by_count, .. }) => {
                self.run_posts_tally(&sort_by_count)
            }
        }
    }

    fn run_posts_log(&self, oneline: &bool) {
        println!(
            "Running posts log for {}, oneline? {oneline}",
            self.username()
        );
    }

    fn run_posts_tally(&self, sort_by_count: &bool) {
        println!(
            "Running posts tally for {}, sort by count? {sort_by_count}",
            self.username()
        );
    }

    fn run_summary(&self) {
        todo!("summary");
    }

    fn run_tally(&self, sort_by_count: &bool) {
        println!(
            "Running comment tally for {}, sort by count? {sort_by_count}",
            self.username()
        );
    }

    fn run_timeline(&self) {
        // TODO: This is hard to test -- should move the conditional check
        //       into testable method, maybe Timeline::view(), although I'm
        //       not sure the logic is appropriate there, either.
        if self.user().has_comments() {
            println!("{}", self.user().timeline().view(&ViewOptions::default()));
        } else {
            println!("{} has no comments.", self.user().username());
        }
    }
}
