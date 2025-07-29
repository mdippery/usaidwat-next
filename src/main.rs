// usaidwat
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap::Parser;
use reqwest::StatusCode;
use std::process;
use usaidwat::cli::{Config, Error, Runner};
use usaidwat::http::HTTPError;

fn die(error_code: i32, message: &str) {
    eprintln!("{}", message);
    process::exit(error_code);
}

fn dispatch_err(username: &str, err: &Error) {
    let message = match err {
        Error::Service(HTTPError::Http(StatusCode::NOT_FOUND)) => {
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
        Ok(runner) => match runner.run().await {
            Err(message) => die(1, &message),
            Ok(()) => (),
        },
        Err(err) => dispatch_err(&username, &err),
    }
}
