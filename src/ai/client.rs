// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! API clients for various AI services.

pub mod openai;

use hypertyper::HTTPError;
use std::fmt::Debug;

/// A client for an AI service's API.
///
/// API clients for specific AI services should adopt this trait to provide
/// a common interface for interacting with all AI services in a uniform manner.
pub trait AIClient {
    /// The client can make API requests of this type.
    type AIRequest: AIRequest;

    /// The client receives API responses of this type.
    type AIResponse: AIResponse;

    /// Sends the request to the AI service and receives a response.
    fn send(
        &self,
        request: &Self::AIRequest,
    ) -> impl Future<Output = AIResult<Self::AIResponse>> + Send;
}

/// A request to an AI service's API.
///
/// Different AI services may offer different options when making API requests,
/// and different [`AIClient`] implementations may offer different capabilities.
/// `AIRequest` offers a uniform way to make requests to AI services that
/// may differ slightly in behavior. If a service does not offer a particular
/// feature, then its `AIRequest` implementations can do nothing for calls
/// to the accompanying methods.
///
/// # Examples
///
/// This trait follows a "builder" pattern where elements of the request
/// are built up over time.
///
/// Assuming you have enum called `Model` that specifies available AI models
/// for your service, and a `ConcreteAPIRequest` struct that implements
/// `AIRequest`, you would create an API request like this:
///
/// ```
/// # use usaidwat::ai::client::{AIModel, AIRequest};
/// #
/// # #[derive(Clone, Copy, Debug, Default)]
/// # pub enum Model {
/// #     #[default]
/// #     AIModel,
/// # }
/// #
/// # impl AIModel for Model {
/// #     fn flagship() -> Self {
/// #         Model::AIModel
/// #     }
/// #
/// #     fn best() -> Self {
/// #         Model::AIModel
/// #     }
/// #
/// #     fn fastest() -> Self {
/// #         Model::AIModel
/// #     }
/// #
/// #     fn cheapest() -> Self {
/// #         Model::AIModel
/// #     }
/// # }
/// #
/// # #[derive(Default)]
/// # pub struct ConcreteAPIRequest;
/// #
/// # impl AIRequest for ConcreteAPIRequest {
/// #     type Model = Model;
/// #     fn model(self, model: Self::Model) -> Self { self }
/// #     fn instructions(self, instructions: impl Into<String>) -> Self { self }
/// #     fn input(self, input: impl Into<String>) -> Self { self }
/// # }
/// #
/// let request = ConcreteAPIRequest::default()
///     .model(Model::AIModel)
///     .instructions("Be really snarky.")
///     .input("How do I make an API request?");
/// ```
pub trait AIRequest: Default {
    /// An enum or other data structures providing options for different
    /// AI models, which are specific to each service.
    type Model: AIModel;

    /// Sets the model used by the API request and returns a new
    /// request.
    ///
    /// AI services often have many different models; consult the
    /// documentation for your specific AI model for options.
    fn model(self, model: Self::Model) -> Self;

    /// Sets specialized instructions for the request and returns a new
    /// request.
    ///
    /// Some AI models allow callers to specify instructions for
    /// generating responses, such as tone, goals, or examples of
    /// correct responses. Consult the API documentation for your
    /// specific service to see if it allows instructions to be
    /// specified. If not, this method can be a no-op.
    ///
    /// Often specialized instructions will take precedence over the
    /// request's [input](AIRequest::input). Consult the documentation
    /// for your specific service to see if that is the case.
    fn instructions(self, instructions: impl Into<String>) -> Self;

    /// Sets the request's input and returns a new request.
    ///
    /// The input is often referred to as a "prompt" and is the text
    /// for which an AI service generates a response.
    fn input(self, input: impl Into<String>) -> Self;
}

/// A response from an AI service's API.
pub trait AIResponse {
    /// Concatenates the output from an AI service into a single string.
    fn concatenate(&self) -> String;
}

/// An API result that includes the response if successful or an error
/// if unsuccessful.
pub type AIResult<T> = Result<T, AIError>;

/// An error from an AI service's API.
pub type AIError = HTTPError;

/// An AI model specification.
pub trait AIModel: Clone + Copy + Default + Debug {
    /// The service's standard or default model.
    ///
    /// Often this is the same as the [best](AIModel::best()), but
    /// there is no guarantee that is true.
    fn flagship() -> Self;

    /// The "best" model available for a given LLM.
    ///
    /// "Best" is obviously subjective, but generally this is the model
    /// that offers the best price/performance ratio, and is what its
    /// provider has defined to be the "best".
    fn best() -> Self;

    /// The least expensive model available for a given LLM.
    fn cheapest() -> Self;

    /// The fastest model available for a given LLM.
    fn fastest() -> Self;
}
