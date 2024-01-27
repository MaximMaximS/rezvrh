use super::super::api::{RequestError, RequestResult};
use super::super::bakalari::Bakalari;
use super::{Timetable, Which};
use bimap::BiHashMap;
use scraper::{Html, Selector};
use std::{fs, io};

impl Bakalari {
    /// Test api connection
    ///
    /// # Errors
    /// If request fails
    ///
    /// # Panics
    /// If url join fails
    pub async fn test(&mut self) -> RequestResult<()> {
        let client = self.client();
        let res = client
            .reqwest_client()
            .get(client.url().join("/timetable/public").unwrap())
            .header("Cookie", format!("BakaAuth={}", self.get_token().await?))
            .send()
            .await?;

        let text = res.text().await?;

        if !text.contains("timetable") {
            return Err(RequestError::UnknownResponse("timetable not present"));
        }

        Ok(())
    }

    /// Get list of classes (name - id)
    ///
    /// # Errors
    /// If request fails
    ///
    /// # Panics
    /// If url join fails (shouldn't)
    pub async fn get_classes(&mut self) -> RequestResult<BiHashMap<String, String>> {
        let client = self.client();
        let res = client
            .reqwest_client()
            .get(client.url().join("/timetable/public").unwrap())
            .header("Cookie", format!("BakaAuth={}", self.get_token().await?))
            .send()
            .await?;

        let text = res.text().await?;

        let document = Html::parse_document(&text);
        let selector = Selector::parse("select#selectedClass > option[value]").unwrap();

        let options = document
            .select(&selector)
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
            .collect::<Result<BiHashMap<_, _>, _>>()?;

        Ok(options)
    }

    /// Get class timetable
    ///
    /// # Errors
    /// If request fails
    ///
    /// # Panics
    /// If url join fails (shouldn't)
    pub async fn get_class(&mut self, id: &str, which: Which) -> RequestResult<Timetable> {
        let client = self.client();
        let res = client
            .reqwest_client()
            .get(
                client
                    .url()
                    .join(&format!("/timetable/public/{which}/class/{id}"))
                    .unwrap(),
            )
            .header("Cookie", format!("BakaAuth={}", self.get_token().await?))
            .send()
            .await?;

        let html = res.text().await?;

        Ok(Timetable::parse(&html, super::Type::Class)?)
    }
}
