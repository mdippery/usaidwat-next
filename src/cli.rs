use clap::{Args, Parser, Subcommand, ValueEnum};

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

#[derive(Args, Debug)]
struct TallyConfig {
    /// Reddit username
    username: String,

    /// Sort output by number of comments
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

#[derive(Clone, Debug, Default, ValueEnum)]
enum DateFormat {
    #[default]
    Absolute,
    Relative,
}

pub fn run(config: Config) {
    println!("{:?}", config);
}
