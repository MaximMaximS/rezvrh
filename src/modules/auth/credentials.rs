use super::{LoginError, LoginResult};
use crate::modules::bakalari::Client;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, oneshot};

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

type TokenRequest = (Arc<Client>, oneshot::Sender<LoginResult<String>>);

/// Struct that hold the credentials and token
#[derive(Debug)]
pub struct Credentials {
    sender: mpsc::Sender<TokenRequest>,
}

impl Credentials {
    /// Create new credentials from username and password
    ///
    /// # Errors
    /// If login fails
    pub async fn new((username, password): (String, String), client: &Client) -> LoginResult<Self> {
        let token = TempToken::new(Self::login((&username, &password), client).await?);

        let (sender, mut receiver) = mpsc::channel::<TokenRequest>(10);

        tokio::spawn(async move {
            let mut store = token;

            while let Some((client, sender)) = receiver.recv().await {
                let token = if let Some(token) = store.get() {
                    Ok(token.to_owned())
                } else {
                    let token = Self::login((&username, &password), &client).await;
                    token.map(|token| {
                        store = TempToken::new(token);
                        store.token.clone()
                    })
                };
                sender.send(token).unwrap();
            }
        });

        Ok(Self { sender })
    }

    /// Get token, and renew in case it expired
    ///
    /// # Errors
    /// If renew fails
    ///
    /// # Panics
    /// Panics if token expires somehow (shouldn't)
    pub async fn get_token(&self, client: Arc<Client>) -> LoginResult<String> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send((client, tx))
            .await
            .expect("failed to send token request");
        rx.await.unwrap()
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
