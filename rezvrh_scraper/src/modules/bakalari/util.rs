use super::RequestError;
use once_cell::sync::Lazy;
use reqwest::{Client as ReqwestClient, Url};
use scraper::{Html, Selector};
use std::collections::HashMap;

/// Extract options for specified selector
pub fn get_map(
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

static CLASSES_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("select#selectedClass > option[value]").unwrap());
static TEACHERS_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("select#selectedTeacher > option[value]").unwrap());
static ROOMS_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("select#selectedRoom > option[value]").unwrap());

/// Get classes, teachers and rooms
///
/// # Errors
/// If request fails
pub async fn get_info(
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
        .get(url.join("timetable/public").unwrap())
        .header("Cookie", format!("BakaAuth={token}"))
        .send()
        .await?;

    let text = res.text().await?;

    let document = Html::parse_document(&text);

    let classes = get_map(&document, &CLASSES_SELECTOR)?;
    let teachers = get_map(&document, &TEACHERS_SELECTOR)?;
    let rooms = get_map(&document, &ROOMS_SELECTOR)?;

    Ok((classes, teachers, rooms))
}
