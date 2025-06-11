use clap::Parser;
use usaidwat::cli::{Config, Runner};

fn main() {
    let config = Config::parse();
    env_logger::builder()
        .filter_level(config.verbosity().into())
        .init();
    Runner::new(config).run()
}
