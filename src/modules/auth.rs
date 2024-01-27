use std::time::{Duration, Instant};

use reqwest::Response;
use thiserror::Error;

use super::bakalari::Client;

/// Struct to hold token that expires after certain time
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TempToken {
    token: String,
    expiration: Instant,
}

/// Lifetime of [`TempToken`] in seconds
const TOKEN_LIFETIME: u64 = 60 * 5;

impl TempToken {
    /// Create token with expiration [`TOKEN_LIFETIME`]
    fn new(token: String) -> Self {
        Self {
            token,
            expiration: Instant::now() + Duration::from_secs(TOKEN_LIFETIME),
        }
    }

    /// Whether token is expired
    fn expired(&self) -> bool {
        Instant::now() > self.expiration
    }

    /// Get reference to token if it is not expired
    fn get(&self) -> Option<&str> {
        if self.expired() {
            return None;
        }
        Some(&self.token)
    }
}

/// Struct that hold the credentials and token
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Credentials {
    username: String,
    password: String,
    token: TempToken,
}

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

impl Credentials {
    /// Create new credentials from username and password
    ///
    /// # Errors
    /// If login fails
    pub async fn new((username, password): (String, String), client: &Client) -> LoginResult<Self> {
        let token = TempToken::new(Self::login((&username, &password), client).await?);

        Ok(Self {
            username,
            password,
            token,
        })
    }

    /// Get token, and renew in case it expired
    ///
    /// # Errors
    /// If renew fails
    ///
    /// # Panics
    /// Panics if token expires somehow (shouldn't)
    pub async fn get_token(&mut self, client: &Client) -> LoginResult<&str> {
        if self.token.expired() {
            self.renew(client).await?;
        }
        let tkn = self.token.get().unwrap();
        Ok(tkn)
    }

    /// Renew token
    async fn renew(&mut self, client: &Client) -> LoginResult<()> {
        let new_token = Self::login((&self.username, &self.password), client).await?;
        self.token = TempToken::new(new_token);
        Ok(())
    }

    // Issue new token from api
    pub async fn login((username, password): (&str, &str), client: &Client) -> LoginResult<String> {
        let res = client
            .reqwest_client()
            .post(client.url().join("Login").unwrap())
            .body(format!("username={username}&password={password}"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await?;

        if res.status().as_u16() != 302 {
            return Err(LoginError::Login(res));
        }

        let v = res
            .headers()
            .get_all("Set-Cookie")
            .iter()
            .filter_map(|h| h.to_str().ok())
            .filter_map(|h| h.split_once(';'))
            .map(|h| h.0)
            .filter_map(|h| h.split_once('='))
            .find(|h| h.0 == "BakaAuth")
            .map(|h| h.1)
            .ok_or(LoginError::CookieParse)?;

        Ok(v.to_owned())
    }
}

/// Authentication types
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Auth {
    /// Username and password
    Credentials(Credentials),
    // Token (might expire)
    Token(String),
}

impl Auth {
    /// Get api token
    ///
    /// # Errors
    /// If token renew fails
    pub async fn get_token(&mut self, client: &Client) -> LoginResult<&str> {
        match self {
            Self::Token(token) => Ok(token),
            Self::Credentials(creds) => creds.get_token(client).await,
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
