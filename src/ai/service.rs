// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! Services for communicating with APIs using HTTP.

pub use hypertyper::auth::Auth;
use hypertyper::service::HTTPService;
use hypertyper::{HTTPClient, HTTPClientFactory, HTTPResult};
use reqwest::{IntoUrl, header};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// A concrete implementation of an HTTP API service.
///
/// This is the "default" service used by most AI API clients. It more or
/// less just wraps a Reqwest client, making it easier to swap out the
/// service for a deterministic service when writing tests. Most AI API
/// clients should use this `APIService` by default.
#[derive(Debug)]
pub struct Service {
    client: HTTPClient,
}

impl Service {
    /// Creates a new HTTP service using clients from the given factory.
    pub fn new(factory: HTTPClientFactory) -> Self {
        let client = factory.create();
        Self { client }
    }
}

impl HTTPService for Service {
    /// Sends a GET request to a Reddit API endpoint and returns the raw body.
    ///
    /// # Panics
    ///
    /// Always, because the AI HTTP service only needs to make POST requests.
    async fn get<U>(&self, _uri: U) -> HTTPResult<String>
    where
        U: IntoUrl + Send,
    {
        // TODO: Don't define this on HTTP client.
        // unimplemented!() is a bit of a cop-out. Ideally hypertyper::service::HTTPService
        // would allow us to only implement the methods we want to implement, but as of
        // v0.2.0, that is not possible, so we will panic() here.
        unimplemented!("AI HTTP service does not support GET requests");
    }

    /// Send a POST request to the `uri` with the JSON object `data` as
    /// the POST request body.
    ///
    /// The response is deserialized from a string to the JSON object
    /// specified by the `R` type parameter.
    async fn post<U, D, R>(&self, uri: U, auth: &Auth, data: &D) -> HTTPResult<R>
    where
        U: IntoUrl + Send,
        D: Serialize + Sync,
        R: DeserializeOwned,
    {
        let auth_header = format!("Bearer {}", auth.api_key());
        let json_object = self
            .client
            .post(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, auth_header)
            .json(data)
            .send()
            .await?
            .json::<R>()
            .await?;
        Ok(json_object)
    }
}
