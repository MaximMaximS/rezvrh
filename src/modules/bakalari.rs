use reqwest::{redirect::Policy, Client as ReqwestClient, Url};

use super::auth::{Auth, Credentials, LoginError, LoginResult};

/// Struct that holds HTTP Client and base url
#[derive(Debug)]
pub struct Client {
    reqwest_client: ReqwestClient,
    url: Url,
}

impl Client {
    /// Get reqwest client
    pub const fn reqwest_client(&self) -> &ReqwestClient {
        &self.reqwest_client
    }

    /// Get base url
    pub const fn url(&self) -> &Url {
        &self.url
    }

    /// Create new Bakalari Client
    ///
    /// # Panics
    /// This method fails if a TLS backend cannot be initialized, or the resolver cannot load the system configuration.
    #[must_use]
    pub fn new(url: Url) -> Self {
        Self {
            reqwest_client: ReqwestClient::builder()
                .redirect(Policy::none())
                .build()
                .unwrap(),
            url,
        }
    }
}

/// Bakalari api struct
#[derive(Debug)]
pub struct Bakalari {
    client: Client,
    auth: Auth,
}

impl Bakalari {
    /// Get client
    #[must_use]
    pub const fn client(&self) -> &Client {
        &self.client
    }

    /// Create Bakalari instance from username and password
    ///
    /// # Errors
    /// Returns error if authentication fails
    pub async fn from_creds(creds: (String, String), url: Url) -> Result<Self, LoginError> {
        let client = Client::new(url);
        let auth = Auth::from_creds((creds.0, creds.1), &client).await?;
        Ok(Self { client, auth })
    }

    /// Create Bakalari instance without storing credentials
    ///
    /// # Errors
    /// Returns error if authentication fails
    pub async fn from_creds_no_store(creds: (&str, &str), url: Url) -> Result<Self, LoginError> {
        let client = Client::new(url);
        let token = Credentials::login((&creds.0, &creds.1), &client).await?;
        let auth = Auth::from_token(token);
        Ok(Self { client, auth })
    }

    /// Get token
    ///
    /// # Errors
    /// If renew fails
    pub async fn get_token(&mut self) -> LoginResult<&str> {
        self.auth.get_token(&self.client).await
    }
}
