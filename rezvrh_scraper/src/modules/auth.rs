use super::bakalari::Client;
use reqwest::Response;
use std::borrow::Cow;
use std::sync::Arc;
use thiserror::Error;

pub use credentials::Credentials;

mod credentials;

/// Authentication error
#[derive(Debug, Error)]
pub enum LoginError {
    /// Generic request error
    #[error("{0}")]
    Request(#[from] reqwest::Error),
    /// Login error (probably wrong credentials)
    #[error("login failed")]
    Login(Response),
    /// Parsing of cookei from resposne failed
    #[error("failed to parse cookie")]
    CookieParse,
}

pub type LoginResult<T> = Result<T, LoginError>;

/// Authentication types
#[derive(Debug)]
pub enum Auth {
    /// Username and password
    Credentials(Credentials),
    // Token (might expire)
    Token(String),
    None,
}

impl Auth {
    /// Get api token
    ///
    /// # Errors
    /// If token renew fails
    pub async fn get_token(&self, client: Arc<Client>) -> LoginResult<Cow<'_, String>> {
        match self {
            Self::Token(token) => Ok(Cow::Borrowed(token)),
            Self::Credentials(creds) => Ok(Cow::Owned(creds.get_token(client.clone()).await?)),
            Self::None => Ok(Cow::Owned(String::new())),
        }
    }

    /// Create auth from username and password
    ///
    /// # Errors
    /// If login fails
    pub async fn from_creds(creds: (String, String), client: &Client) -> LoginResult<Self> {
        Ok(Self::Credentials(Credentials::new(creds, client).await?))
    }

    /// Create auth from token
    ///
    #[must_use]
    pub const fn from_token(token: String) -> Self {
        Self::Token(token)
    }
}
