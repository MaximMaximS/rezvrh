use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use base64::prelude::*;
use rezvrh_scraper::{Bakalari, Error as BakalariError};
use thiserror::Error;

// Extract basic auth from headers
fn auth(headers: &HeaderMap) -> Option<(String, String)> {
    let auth = headers.get("authorization")?;
    let auth = auth.to_str().ok()?;
    let auth = auth.strip_prefix("Basic ")?;
    let auth = BASE64_STANDARD.decode(auth).ok()?;
    let auth = String::from_utf8(auth).ok()?;
    let (username, password) = auth.split_once(':')?;
    Some((username.to_string(), password.to_string()))
}

#[derive(Debug, Error)]
enum ApiError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Bad URL")]
    BadUrl,
    #[error("Scrape error: {0}")]
    ScrapeError(#[from] BakalariError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Self::ScrapeError(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
            Self::BadUrl => (StatusCode::BAD_REQUEST, "No URL provided").into_response(),
        }
    }
}

#[derive(serde::Deserialize)]
struct GetQuery {
    url: String,
}

async fn get_api(headers: HeaderMap, query: Query<GetQuery>) -> Result<Bakalari, ApiError> {
    let (username, password) = auth(&headers).ok_or(ApiError::Unauthorized)?;
    let url = query.url.parse().map_err(|_| ApiError::BadUrl)?;
    Ok(Bakalari::from_creds_no_store((&username, &password), url).await?)
}

async fn get_rooms(
    headers: HeaderMap,
    query: Query<GetQuery>,
) -> Result<Json<Vec<String>>, ApiError> {
    let bakalari = get_api(headers, query).await?;
    let classes = bakalari.get_objects(rezvrh_scraper::Type::Room);
    Ok(Json(classes))
}

async fn get_classes(
    headers: HeaderMap,
    query: Query<GetQuery>,
) -> Result<Json<Vec<String>>, ApiError> {
    let bakalari = get_api(headers, query).await?;
    let classes = bakalari.get_objects(rezvrh_scraper::Type::Class);
    Ok(Json(classes))
}

async fn get_teachers(
    headers: HeaderMap,
    query: Query<GetQuery>,
) -> Result<Json<Vec<String>>, ApiError> {
    let bakalari = get_api(headers, query).await?;
    let classes = bakalari.get_objects(rezvrh_scraper::Type::Teacher);
    Ok(Json(classes))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/classes", get(get_classes))
        .route("/rooms", get(get_rooms))
        .route("/teachers", get(get_teachers));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
