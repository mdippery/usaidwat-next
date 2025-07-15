//! AI services.

use std::{env, error, fmt};

/// Manages authentication keys for AI service APIs.
#[derive(Debug)]
pub struct Auth {
    api_key: String,
}

impl Auth {
    /// Creates a new `Auth` structure using the given API key.
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: String::from(api_key),
        }
    }

    /// Retrieves an API key from the environment.
    ///
    /// Returns an error if the API key cannot be retrieved from the
    /// environment.
    pub fn from_env(envvar: &str) -> AuthResult {
        let api_key = env::var(envvar).map_err(AuthError::EnvError)?;
        Ok(Self { api_key })
    }

    /// The actual API key.
    ///
    /// # Examples
    ///
    /// ```
    /// use usaidwat::ai::Auth;
    /// let auth = Auth::new("ThisIsMyApiKey");
    /// assert_eq!(auth.api_key(), "ThisIsMyApiKey");
    /// ```
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

/// Standard result type for [`Auth`] creation.
type AuthResult = Result<Auth, AuthError>;

/// Indicates an error when creating an authentication key.
#[derive(Debug)]
pub enum AuthError {
    /// An error occurred while retrieving a key from the environment.
    EnvError(env::VarError),
}

impl From<env::VarError> for AuthError {
    fn from(error: env::VarError) -> Self {
        AuthError::EnvError(error)
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::EnvError(err) => write!(f, "Environment error: {err}"),
        }
    }
}

impl error::Error for AuthError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            AuthError::EnvError(err) => Some(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env::{with_var, with_var_unset};

    #[test]
    fn it_creates_an_auth_key_from_the_environment() {
        let key_name = "AUTH_API_KEY";
        let key_value = "ThisIsMyApiKey";
        with_var(key_name, Some(key_value), || {
            let auth = Auth::from_env(key_name);
            assert!(auth.is_ok());
            let auth = auth.unwrap();
            assert_eq!(auth.api_key(), key_value);
        })
    }

    #[test]
    fn it_returns_an_error_if_a_key_is_not_set_in_environment() {
        let key_name = "AUTH_API_KEY";
        with_var_unset(key_name, || {
            let auth = Auth::from_env(key_name);
            assert!(auth.is_err());
            assert!(matches!(
                auth.unwrap_err(),
                AuthError::EnvError(env::VarError::NotPresent)
            ));
        })
    }
}
