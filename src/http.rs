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

//! Services for communicating with APIs using HTTP.

use reqwest::{Client, ClientBuilder, header};
use thiserror::Error;

/// A general service for making HTTP calls.
///
/// It might be a bit odd to refer to this trait as a "service", since
/// it appears to be more of a _client_ implementation, but think of
/// this as a proxy for a remote _service_ (even though a _client_ is used
/// to communicate with that remote service). A service might not always
/// be remote, such as when the implementation is a deterministic service
/// used for testing.
pub trait HTTPService {
    /// Default HTTP client that can be used to make HTTP requests.
    fn client() -> Client {
        ClientBuilder::new()
            .user_agent(Self::user_agent())
            .build()
            // Better error handling? According to the docs, build() only
            // fails if a TLS backend cannot be initialized, or if DNS
            // resolution cannot be initialized, and both of these seem
            // like unrecoverable errors for us.
            .expect("could not create a new HTTP client")
    }

    /// An appropriate user agent to use when making HTTP requests.
    fn user_agent() -> String {
        format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}

/// The result of an HTTP request.
pub type HTTPResult<T> = Result<T, HTTPError>;

/// Indicates an error has occurred when making an HTTP call.
#[derive(Debug, Error)]
pub enum HTTPError {
    /// An error that occurred while making an HTTP request.
    #[error("Error while making or processing an HTTP request: {0}")]
    Request(#[from] reqwest::Error),

    /// An error that occurred while trying to serialize a POST body.
    #[error("Error serializing POST body: {0}")]
    Serialization(#[from] serde_json::Error),

    /// An unsuccessful HTTP status code in an HTTP response.
    #[error("Request returned HTTP {0}")]
    Http(reqwest::StatusCode),

    /// A missing Content-Type header in a response.
    #[error("Missing Content-Type header")]
    MissingContentType,

    /// An invalid Content-Type header.
    #[error("Invalid Content-Type header value: {0}")]
    InvalidContentType(#[from] header::ToStrError),

    /// A Content-Type that is not understood by the service.
    #[error("Unexpected content type: {0}")]
    UnexpectedContentType(String),
}

#[cfg(test)]
mod tests {
    use crate::http::HTTPService;
    use regex::Regex;

    #[allow(dead_code)]
    struct UserAgentTestService {}
    impl HTTPService for UserAgentTestService {}

    #[test]
    fn it_returns_user_agent_with_version_number() {
        let user_agent = UserAgentTestService::user_agent();
        let version_re = Regex::new(r"^[a-z]+ v\d+\.\d+\.\d+(-(alpha|beta)(\.\d+)?)?$").unwrap();
        assert!(
            version_re.is_match(&user_agent),
            "{} does not match {}",
            user_agent,
            version_re,
        );
    }
}
