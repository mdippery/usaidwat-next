// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025-2026 Michael Dippery <michael@monkey-robot.com>

use clap::Parser;
use hypertyper::HttpError;
use reqwest::StatusCode;
use std::process;
use usaidwat::cli::{Config, Runner};

fn die(error_code: i32, message: &str) {
    eprintln!("{}", message);
    process::exit(error_code);
}

fn dispatch_err(username: &str, err: &anyhow::Error) {
    let message = match err.downcast_ref::<HttpError>() {
        Some(HttpError::Http(StatusCode::NOT_FOUND)) => {
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
        Ok(runner) => {
            if let Err(err) = runner.run().await {
                die(1, &err.to_string())
            }
        }
        Err(err) => dispatch_err(&username, &err),
    }
}
