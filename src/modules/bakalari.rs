use super::timetable::Type;
use once_cell::sync::Lazy;
use reqwest::{redirect::Policy, Client as ReqwestClient, Url};
use scraper::{Html, Selector};
use std::{borrow::Cow, collections::HashMap, sync::Arc};

use super::{
    api::RequestError,
    auth::{Auth, Credentials, LoginResult},
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
    client: Arc<Client>,
    auth: Auth,
    classes: HashMap<String, String>,
    teachers: HashMap<String, String>,
    rooms: HashMap<String, String>,
}

static CLASSES_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("select#selectedClass > option[value]").unwrap());
static TEACHERS_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("select#selectedTeacher > option[value]").unwrap());
static ROOMS_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("select#selectedRoom > option[value]").unwrap());

impl Bakalari {
    /// Get classes
    #[must_use]
    pub fn get_classes(&self) -> Vec<String> {
        self.classes
            .keys()
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    }

    /// Get class
    #[must_use]
    pub fn get_class(&self, class: &str) -> Option<Type> {
        self.classes.get(class).map(|id| Type::Class(id))
    }

    /// Get teachers
    #[must_use]
    pub fn get_teachers(&self) -> Vec<String> {
        self.teachers
            .keys()
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    }

    /// Get teacher
    #[must_use]
    pub fn get_teacher(&self, teacher: &str) -> Option<Type> {
        self.teachers.get(teacher).map(|id| Type::Teacher(id))
    }

    /// Get rooms
    #[must_use]
    pub fn get_rooms(&self) -> Vec<String> {
        self.rooms
            .keys()
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    }

    /// Get room
    #[must_use]
    pub fn get_room(&self, room: &str) -> Option<Type> {
        self.rooms.get(room).map(|id| Type::Room(id))
    }

    fn get_map(
        document: &Html,
        selector: &Selector,
    ) -> Result<HashMap<String, String>, RequestError> {
        document
            .select(selector)
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
            .collect::<Result<HashMap<_, _>, _>>()
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
            .header("Cookie", format!("BakaAuth={token}"))
            .send()
            .await?;

        let text = res.text().await?;

        tokio::fs::write("/tmp/test.html", &text).await.unwrap();

        let document = Html::parse_document(&text);

        let classes = Self::get_map(&document, &CLASSES_SELECTOR)?;
        let teachers = Self::get_map(&document, &TEACHERS_SELECTOR)?;
        let rooms = Self::get_map(&document, &ROOMS_SELECTOR)?;

        Ok((classes, teachers, rooms))
    }

    /// Get client
    #[must_use]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Create Bakalari instance from username and password
    ///
    /// # Errors
    /// Returns error if authentication fails
    pub async fn from_creds(creds: (String, String), url: Url) -> Result<Self, RequestError> {
        let client = Arc::new(Client::new(url));
        let auth = Auth::from_creds((creds.0, creds.1), &client).await?;
        let (classes, teachers, rooms) = Self::get_info(
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
            Self::get_info(client.reqwest_client(), client.url(), &token).await?;
        let auth = Auth::from_token(token);
        Ok(Self {
            client,
            auth,
            classes,
            teachers,
            rooms,
        })
    }

    /// Get token
    ///
    /// # Errors
    /// If renew fails
    pub async fn get_token(&self) -> LoginResult<Cow<'_, String>> {
        self.auth.get_token(self.client.clone()).await
    }
}
