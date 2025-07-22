//! Services for communicating with APIs using HTTP.

use crate::http::HTTPService as BaseHTTPService;
use reqwest::Client;

/// A general service for making HTTP calls to an API.
///
/// While this may appear to be more like a "client", think of it as a
/// proxy for a (possibly remote) API service. The term "service" is used
/// here in the same spirit as that of [`HTTPService`](http::HTTPService).
pub trait APIService: BaseHTTPService {}

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

impl APIService for HTTPService {}
