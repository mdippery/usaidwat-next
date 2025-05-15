use clap::Parser;
use usaidwat::cli::Config;

fn main() {
    let config = Config::parse();
    usaidwat::cli::run(config)
}
