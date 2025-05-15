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

impl Command {
    pub fn username(&self) -> &str {
        match &self {
            Command::Info { username } => username,
            Command::Log { username, ..} => username,
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
            PostSubcommand::Tally(TallyConfig { username, ..}) => username,
        }
    }
}

#[derive(Clone, Debug, Default, ValueEnum)]
enum DateFormat {
    #[default]
    Absolute,
    Relative,
}

fn run_info(username: String) {
    println!("Running info for {username}");
}

fn run_log(
    username: String,
    date_format: DateFormat,
    grep: Option<String>,
    limit: Option<u32>,
    oneline: bool,
    raw: bool,
) {
    println!(
        "Running log for {username}, date_format = {date_format:?}, grep = {grep:?}, limit = {limit:?}, oneline? {oneline}, raw? {raw}"
    );
}

fn run_posts_log(username: String, oneline: bool) {
    println!("Running posts log for {username}, oneline? {oneline}");
}

fn run_posts_tally(username: String, sort_by_count: bool) {
    println!("Running posts tally for {username}, sort by count? {sort_by_count}");
}

fn run_posts(config: PostCommandConfig) {
    match config.command {
        PostSubcommand::Log { username, oneline } => run_posts_log(username, oneline),
        PostSubcommand::Tally(TallyConfig {
            username,
            sort_by_count,
        }) => run_posts_tally(username, sort_by_count),
    }
}

fn run_summary(username: String) {
    println!("Running summary for {username}");
}

fn run_tally(username: String, sort_by_count: bool) {
    println!("Running comment tally for {username}, sort by count? {sort_by_count}");
}

fn run_timeline(username: String) {
    println!("Running timeline for {username}");
}

pub fn run(config: Config) {
    match config.command {
        Command::Info { username } => run_info(username),
        Command::Log {
            username,
            date,
            grep,
            limit,
            oneline,
            raw,
        } => {
            let date_format = date.unwrap_or(DateFormat::Absolute);
            run_log(username, date_format, grep, limit, oneline, raw);
        }
        Command::Posts(subconfig) => run_posts(subconfig),
        Command::Summary { username } => run_summary(username),
        Command::Tally(TallyConfig {
            username,
            sort_by_count,
        }) => run_tally(username, sort_by_count),
        Command::Timeline { username } => run_timeline(username),
    }
}
