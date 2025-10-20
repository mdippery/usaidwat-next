// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! Services for communicating with AI service providers over HTTP.
//!
//! [`Service`] acts as a proxy for an AI service provider's API (often
//! implemented as a REST API). As an implementation-agnostic definition
//! of a service, it allows consumers to implement a single API client
//! that can communicate with the AI using various mechanisms. In
//! particularly, it provides an easy way to "mock" an API client's
//! HTTP functionality in testing by providing a mocked `Service`
//! implementation for an API client under test, or an actual HTTP
//! client when the API client is used in production.
//!
//! # See Also
//!
//! - [`hypertyper.service`] for an example of how to use `Service` to mock
//!   HTTP calls.
//!
//! [`hypertyper.service`]: https://docs.rs/hypertyper/latest/hypertyper/service/index.html

use hypertyper::{Auth, HTTPClient, HTTPClientFactory, HTTPPost, HTTPResult, IntoUrl};
use reqwest::header;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// A concrete implementation of an HTTP API service.
///
/// This is the "default" service used by most AI API clients. It more or
/// less just wraps a Reqwest client, making it easier to swap out the
/// service for a deterministic service when writing tests. Most AI API
/// clients should use this `Service` by default.
#[derive(Debug)]
pub struct Service {
    client: HTTPClient,
}

impl Service {
    /// Creates a new HTTP service that communicate using clients from the
    /// given factory.
    pub fn new(factory: HTTPClientFactory) -> Self {
        let client = factory.create();
        Self { client }
    }
}

impl HTTPPost for Service {
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
