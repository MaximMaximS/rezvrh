use std::collections::HashMap;

use once_cell::sync::Lazy;
use reqwest::{redirect::Policy, Client as ReqwestClient, Url};
use scraper::{Html, Selector};

use super::{
    api::RequestError,
    auth::{Auth, Credentials, LoginError, LoginResult},
};

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
    classes: HashMap<String, String>,
    teachers: HashMap<String, String>,
    rooms: HashMap<String, String>,
}

static CLASSES_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("select#selectedClass > option[value]").unwrap());

impl Bakalari {
    /// Get classes
    #[must_use]
    pub const fn get_classes(&self) -> &HashMap<String, String> {
        &self.classes
    }

    /// Get teachers
    #[must_use]
    pub const fn get_teachers(&self) -> &HashMap<String, String> {
        &self.teachers
    }

    /// Get rooms
    #[must_use]
    pub const fn get_rooms(&self) -> &HashMap<String, String> {
        &self.rooms
    }


    /// Get classes, teachers and rooms
    /// 
    /// # Errors
    /// If request fails
    async fn get_info(
        client: &ReqwestClient,
        url: &Url,
        token: &str,
    ) -> Result<
        (
            HashMap<String, String>,
            HashMap<String, String>,
            HashMap<String, String>,
        ),
        RequestError,
    > {
        let res = client
            .get(url.join("/timetable/public").unwrap())
            .header("Cookie", format!("BakaAuth={}", token))
            .send()
            .await?;

        let text = res.text().await?;

        let document = Html::parse_document(&text);

        let classes = document
            .select(&CLASSES_SELECTOR)
            .map(|e| {
                let mut texts = e.text();
                let name = texts
                    .next()
                    .ok_or_else(|| RequestError::UnknownResponse("missing class name"))?;
                let id = e
                    .attr("value")
                    .ok_or_else(|| RequestError::UnknownResponse("missing value attr"))?;

                Ok::<_, RequestError>((name.trim().to_owned(), id.trim().to_owned()))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok((classes, HashMap::new(), HashMap::new()))
    }

    /// Get client
    #[must_use]
    pub const fn client(&self) -> &Client {
        &self.client
    }

    /// Create Bakalari instance from username and password
    ///
    /// # Errors
    /// Returns error if authentication fails
    pub async fn from_creds(creds: (String, String), url: Url) -> Result<Self, RequestError> {
        let client = Client::new(url);
        let mut auth = Auth::from_creds((creds.0, creds.1), &client).await?;
        let (classes, teachers, rooms) = Self::get_info(client.reqwest_client(), client.url(), &auth.get_token(&client).await?).await?;
        Ok(Self { client, auth , classes, teachers, rooms })
    }

    /// Create Bakalari instance without storing credentials
    ///
    /// # Errors
    /// Returns error if authentication fails
    pub async fn from_creds_no_store(creds: (&str, &str), url: Url) -> Result<Self, RequestError> {
        let client = Client::new(url);
        let token = Credentials::login((&creds.0, &creds.1), &client).await?;
        let (classes, teachers, rooms) = Self::get_info(client.reqwest_client(), client.url(), &token).await?;
        let auth = Auth::from_token(token);
        Ok(Self { client, auth , classes, teachers, rooms })
    }

    /// Get token
    ///
    /// # Errors
    /// If renew fails
    pub async fn get_token(&mut self) -> LoginResult<&str> {
        self.auth.get_token(&self.client).await
    }
}
