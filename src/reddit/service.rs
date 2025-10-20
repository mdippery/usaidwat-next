// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! HTTPS connector for the Reddit API.
//!
//! Service structures in this module provide a low-level way to interact
//! with the Reddit API over HTTPS, essentially a specialized HTTPS client
//! specifically for Reddit.

use hypertyper::{HTTPClient, HTTPClientFactory, HTTPError, HTTPGet, HTTPResult, IntoUrl};
use reqwest::header;

/// A service for retrieving information for Reddit users.
///
/// Using this trait, clients can implement different ways of connecting
/// to the Reddit API, such as an actual connector for production code,
/// and a mocked connector for testing purposes.
pub trait Service: HTTPGet {
    /// Performs a GET request to the `resource` associated with the given
    /// `username` and returns it as a parsed JSON response.
    fn get_resource(
        &self,
        username: &str,
        resource: &str,
    ) -> impl Future<Output = HTTPResult<String>> + Send;
}

/// A service that contacts the Reddit API directly to retrieve information.
pub struct RedditService {
    client: HTTPClient,
}

impl Default for RedditService {
    /// Creates a new Reddit service.
    fn default() -> Self {
        let factory = HTTPClientFactory::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        let client = factory.create();
        Self { client }
    }
}

impl RedditService {
    fn query_string(&self, resource: &str) -> &str {
        match resource {
            "comments" => "?limit=100",
            "submitted" => "?limit=100",
            _ => "",
        }
    }

    fn uri(&self, username: &str, resource: &str) -> String {
        let qs = self.query_string(resource);
        format!("https://www.reddit.com/user/{username}/{resource}.json{qs}")
    }
}

impl HTTPGet for RedditService {
    /// Sends a GET request to a Reddit API endpoint and returns the raw body.
    async fn get<U>(&self, uri: U) -> HTTPResult<String>
    where
        U: IntoUrl + Send,
    {
        let resp = self.client.get(uri).send().await?;

        if !resp.status().is_success() {
            Err(HTTPError::Http(resp.status()))
        } else {
            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .ok_or(HTTPError::MissingContentType)?
                .to_str()?;
            if !content_type.starts_with("application/json") {
                Err(HTTPError::UnexpectedContentType(content_type.to_string()))
            } else {
                Ok(resp.text().await?)
            }
        }
    }
}

impl Service for RedditService {
    async fn get_resource(&self, username: &str, resource: &str) -> HTTPResult<String> {
        let uri = self.uri(username, resource);
        self.get(&uri).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_a_query_string_with_comment_limits() {
        let service = RedditService::default();
        let qs = service.query_string("comments");
        assert_eq!(qs, "?limit=100");
    }

    #[test]
    fn it_returns_a_query_string_with_post_limits() {
        let service = RedditService::default();
        let qs = service.query_string("submitted");
        assert_eq!(qs, "?limit=100");
    }

    #[test]
    fn it_returns_an_empty_query_string_for_profiles() {
        let service = RedditService::default();
        let qs = service.query_string("about");
        assert_eq!(qs, "");
    }

    #[test]
    fn it_returns_a_uri_for_comments() {
        let service = RedditService::default();
        let actual_uri = service.uri("mipadi", "comments");
        let expected_uri = "https://www.reddit.com/user/mipadi/comments.json?limit=100";
        assert_eq!(actual_uri, expected_uri);
    }

    #[test]
    fn it_returns_a_uri_for_posts() {
        let service = RedditService::default();
        let actual_uri = service.uri("mipadi", "submitted");
        let expected_uri = "https://www.reddit.com/user/mipadi/submitted.json?limit=100";
        assert_eq!(actual_uri, expected_uri);
    }

    #[test]
    fn it_returns_a_uri_for_profiles() {
        let service = RedditService::default();
        let actual_uri = service.uri("mipadi", "about");
        let expected_uri = "https://www.reddit.com/user/mipadi/about.json";
        assert_eq!(actual_uri, expected_uri);
    }
}
