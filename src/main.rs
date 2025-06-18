use clap::Parser;
use std::process;
use usaidwat::cli::{Config, Runner};

fn die(error_code: i32, message: &str) {
    eprintln!("{}", message);
    process::exit(error_code);
}

fn main() {
    let config = Config::parse();
    env_logger::builder()
        .filter_level(config.verbosity().into())
        .init();
    match Runner::new(config) {
        Ok(runner) => runner.run(),
        Err(message) => die(1, &message),
    }
}
