use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use base64::prelude::*;
use rezvrh_scraper::{Bakalari, Error as BakalariError, Timetable, Type, Which};
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
    #[error("Bad URL")]
    BadUrl,
    #[error("Scrape error: {0}")]
    ScrapeError(#[from] BakalariError),
    #[error("invalid or missing selector")]
    InvalidSelector,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::ScrapeError(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
            Self::BadUrl => (StatusCode::BAD_REQUEST, "Invalid or missing URL").into_response(),
            Self::InvalidSelector => {
                (StatusCode::BAD_REQUEST, "Invalid or missing selector").into_response()
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct GetQuery {
    url: String,
}

async fn get_api(headers: &HeaderMap, url: &str) -> Result<Bakalari, ApiError> {
    let url = url.parse().map_err(|_| ApiError::BadUrl)?;
    match auth(headers) {
        Some((username, password)) => {
            Ok(Bakalari::from_creds_no_store((&username, &password), url).await?)
        }
        None => Ok(Bakalari::no_auth(url).await?),
    }
}

async fn get_rooms(
    headers: HeaderMap,
    query: Query<GetQuery>,
) -> Result<Json<Vec<String>>, ApiError> {
    let bakalari = get_api(&headers, &query.url).await?;
    let classes = bakalari.get_objects(rezvrh_scraper::Type::Room);
    Ok(Json(classes))
}

async fn get_classes(
    headers: HeaderMap,
    query: Query<GetQuery>,
) -> Result<Json<Vec<String>>, ApiError> {
    let bakalari = get_api(&headers, &query.url).await?;
    let classes = bakalari.get_objects(rezvrh_scraper::Type::Class);
    Ok(Json(classes))
}

async fn get_teachers(
    headers: HeaderMap,
    query: Query<GetQuery>,
) -> Result<Json<Vec<String>>, ApiError> {
    let bakalari = get_api(&headers, &query.url).await?;
    let classes = bakalari.get_objects(rezvrh_scraper::Type::Teacher);
    Ok(Json(classes))
}

async fn get_class_timetable(
    Path((class_name, which)): Path<(String, Which)>,
    headers: HeaderMap,
    query: Query<GetQuery>,
) -> Result<Json<Timetable>, ApiError> {
    let bakalari = get_api(&headers, &query.url).await?;
    let selector = bakalari
        .get_selector(Type::Class, &class_name)
        .ok_or(ApiError::InvalidSelector)?;
    let timetable = bakalari.get_timetable(which, &selector).await?;
    Ok(Json(timetable))
}

async fn get_teacher_timetable(
    Path((teacher_name, which)): Path<(String, Which)>,
    headers: HeaderMap,
    query: Query<GetQuery>,
) -> Result<Json<Timetable>, ApiError> {
    let bakalari = get_api(&headers, &query.url).await?;
    let selector = bakalari
        .get_selector(Type::Teacher, &teacher_name)
        .ok_or(ApiError::InvalidSelector)?;
    let timetable = bakalari.get_timetable(which, &selector).await?;
    Ok(Json(timetable))
}

async fn get_room_timetable(
    Path((room_name, which)): Path<(String, Which)>,
    headers: HeaderMap,
    query: Query<GetQuery>,
) -> Result<Json<Timetable>, ApiError> {
    let bakalari = get_api(&headers, &query.url).await?;
    let selector = bakalari
        .get_selector(Type::Room, &room_name)
        .ok_or(ApiError::InvalidSelector)?;
    let timetable = bakalari.get_timetable(which, &selector).await?;
    Ok(Json(timetable))
}

/*
async fn get_timetable(
    headers: HeaderMap,
    query: Query<GetQuery>,
) -> Result<Json<Timetable>, ApiError> {
    let bakalari = get_api(&headers, &query).await?;
    todo!()
}
*/

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/classes", get(get_classes))
        .route("/rooms", get(get_rooms))
        .route("/teachers", get(get_teachers))
        .route(
            "/timetable/class/:class_name/:which",
            get(get_class_timetable),
        )
        .route(
            "/timetable/teacher/:teacher_name/:which",
            get(get_teacher_timetable),
        )
        .route("/timetable/room/:room_name/:which", get(get_room_timetable));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
