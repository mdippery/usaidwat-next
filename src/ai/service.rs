//! Services for communicating with APIs using HTTP.

use crate::ai::Auth;
use crate::http::{HTTPResult, HTTPService as BaseHTTPService};
use reqwest::header;
use reqwest::{Client, IntoUrl};
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

impl HTTPService {
    pub fn new() -> Self {
        let client = Self::client();
        Self { client }
    }
}

impl BaseHTTPService for HTTPService {}

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
