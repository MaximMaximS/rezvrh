use self::util::get_info;
use super::auth::{Auth, Credentials, LoginError, LoginResult};
use super::timetable::ParseError as TimetableParseError;
use reqwest::{redirect::Policy, Client as ReqwestClient, Url};
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use thiserror::Error;

mod info;
mod other;
mod timetable;
mod util;

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
    client: Arc<Client>,
    auth: Auth,
    classes: HashMap<String, String>,
    teachers: HashMap<String, String>,
    rooms: HashMap<String, String>,
}

impl Bakalari {
    /// Get client
    #[must_use]
    fn client(&self) -> &Client {
        &self.client
    }

    /// Create Bakalari instance from username and password
    ///
    /// # Errors
    /// Returns error if authentication fails
    pub async fn from_creds(creds: (String, String), url: Url) -> Result<Self, RequestError> {
        let client = Arc::new(Client::new(url));
        let auth = Auth::from_creds((creds.0, creds.1), &client).await?;
        let (classes, teachers, rooms) = get_info(
            client.reqwest_client(),
            client.url(),
            &auth.get_token(client.clone()).await?,
        )
        .await?;
        Ok(Self {
            client,
            auth,
            classes,
            teachers,
            rooms,
        })
    }

    /// Create Bakalari instance without storing credentials
    ///
    /// # Errors
    /// Returns error if authentication fails
    pub async fn from_creds_no_store(creds: (&str, &str), url: Url) -> Result<Self, RequestError> {
        let client = Arc::new(Client::new(url));
        let token = Credentials::login((&creds.0, &creds.1), &client).await?;
        let (classes, teachers, rooms) =
            get_info(client.reqwest_client(), client.url(), &token).await?;
        let auth = Auth::from_token(token);
        Ok(Self {
            client,
            auth,
            classes,
            teachers,
            rooms,
        })
    }

    /// Create Bakalari instance without authentication
    /// 
    /// # Errors
    /// Returns error if authentication fails
    pub async fn no_auth(url: Url) -> Result<Self, RequestError> {
        let client = Arc::new(Client::new(url));
        let (classes, teachers, rooms) =
            get_info(client.reqwest_client(), client.url(), "").await?;
        Ok(Self {
            client,
            auth: Auth::None,
            classes,
            teachers,
            rooms,
        })
    }

    /// Get token
    ///
    /// # Errors
    /// If renew fails
    async fn get_token(&self) -> LoginResult<Cow<'_, String>> {
        self.auth.get_token(self.client.clone()).await
    }
}

/// Error of request to api
#[derive(Debug, Error)]
pub enum RequestError {
    /// Authentication error
    #[error("{0}")]
    Login(#[from] LoginError),
    #[error("{0}")]
    Request(#[from] reqwest::Error),
    #[error("server returned unknown response: {0}")]
    UnknownResponse(&'static str),
    #[error("{0}")]
    ParseFailed(#[from] TimetableParseError),
}

pub type RequestResult<T> = Result<T, RequestError>;
