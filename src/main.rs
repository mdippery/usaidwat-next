use clap::Parser;
use reqwest::StatusCode;
use std::process;
use usaidwat::cli::{Config, Error, Runner};
use usaidwat::service;

fn die(error_code: i32, message: &str) {
    eprintln!("{}", message);
    process::exit(error_code);
}

fn dispatch_err(username: &str, err: &Error) {
    let message = match err {
        Error::Service(service::Error::Http(StatusCode::NOT_FOUND)) => {
            format!("no such user: {username}")
        }
        _ => err.to_string(),
    };
    die(67, &message)
}

#[tokio::main]
async fn main() {
    let config = Config::parse();
    let username = config.username();
    env_logger::builder()
        .filter_level(config.verbosity().into())
        .init();
    match Runner::new(config).await {
        Ok(runner) => match runner.run() {
            Err(message) => die(1, &message),
            Ok(()) => (),
        },
        Err(err) => dispatch_err(&username, &err),
    }
}
