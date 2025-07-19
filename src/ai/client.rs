//! API clients for various AI services.

pub mod openai;

use crate::ai::Auth;

/// A client for an AI service's API.
pub trait APIClient {
    /// The client can make API requests of this type.
    type APIRequest: APIRequest;

    /// The client receives API responses of this type.
    type APIResponse: APIResponse;

    /// Creates a new client with the given authentication data.
    fn new(auth: Auth) -> Self;

    /// Sends the request to the AI service and receives a response.
    fn send(&self, request: &Self::APIRequest) -> APIResult<Self::APIResponse>;
}

/// A request to an AI service's API.
///
/// This trait follows a "builder" pattern where elements of the request
/// are built up over time.
///
/// Assuming you have enum called `Model` that specifies available AI models
/// for your service, and a `ConcreteAPIRequest` struct that implements
/// `APIRequest`, you would create an API request like this:
///
/// ```
/// # use usaidwat::ai::client::APIRequest;
/// #
/// # pub enum Model {
/// #     AIModel,
/// # }
/// #
/// # #[derive(Default)]
/// # pub struct ConcreteAPIRequest;
/// #
/// # impl APIRequest for ConcreteAPIRequest {
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
///
/// It is often useful for your concrete implementation to also implement [`Default`]
/// to return an instance with default values already set, although this is not
/// required.
pub trait APIRequest {
    /// An enum or other data structures providing options for different
    /// AI models, which are specific to each service.
    type Model;

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
    /// request's [input](APIRequest::input). Consult the documentation
    /// for your specific service to see if that is the case.
    fn instructions(self, instructions: impl Into<String>) -> Self;

    /// Sets the request's input and returns a new request.
    ///
    /// The input is often referred to as a "prompt" and is the text
    /// for which an AI service generates a response.
    fn input(self, input: impl Into<String>) -> Self;
}

/// A response from an AI service's API.
pub trait APIResponse {}

/// An API result that includes the response if successful or an error
/// if unsuccessful.
pub type APIResult<T> = Result<T, APIError>;

/// An API error.
pub enum APIError {}
