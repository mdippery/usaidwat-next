// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! Services for communicating with APIs using HTTP.

use crate::ai::Auth;
use hypertyper::{Client, HTTPResult, HTTPService as BaseHTTPService};
use reqwest::{IntoUrl, header};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// A general service for making HTTP calls to an API.
///
/// While this may appear to be more like a "client", think of it as a
/// proxy for a (possibly remote) API service. The term "service" is used
/// here in the same spirit as that of [`HTTPService`](BaseHTTPService).
pub trait APIService: BaseHTTPService {
    /// Send a POST request to the `uri` with the JSON object `data` as
    /// the POST request body.
    ///
    /// The response is deserialized from a string to the JSON object
    /// specified in the `R` type parameter.
    fn post<U, D, R>(
        &self,
        uri: U,
        auth: &Auth,
        data: &D,
    ) -> impl Future<Output = HTTPResult<R>> + Send
    where
        U: IntoUrl + Send,
        D: Serialize + Sync,
        R: DeserializeOwned;
}

/// A concrete implementation of an HTTP API service.
///
/// This is the "default" service used by most AI API clients. It more or
/// less just wraps a Reqwest client, making it easier to swap out the
/// service for a deterministic service when writing tests. Most AI API
/// clients should use this `APIService` by default.
#[derive(Debug)]
pub struct HTTPService {
    client: Client,
}

impl Default for HTTPService {
    fn default() -> Self {
        let client = Self::client();
        Self { client }
    }
}

impl BaseHTTPService for HTTPService {
    fn user_agent() -> String {
        // TODO: Correctly infer user agent in crate
        // When this code is split off into a separate crate, this is going
        // to end up using the name of the _crate_, not the consumer of the
        // crate. I'm going to have to figure out how best to deal with that
        // issue.
        format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}

impl APIService for HTTPService {
    // This is covered by the openai_service_https integration test.
    // It would be amazing to test this via unit tests, but that's going
    // to be really hard.
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
