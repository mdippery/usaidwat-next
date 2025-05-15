use clap::Parser;
use usaidwat::cli::{Config, Runner};

fn main() {
    let config = Config::parse();
    Runner::new(config).run()
}
