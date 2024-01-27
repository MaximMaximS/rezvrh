use tokio::time::Instant;

use super::super::api::{RequestError, RequestResult};
use super::super::bakalari::Bakalari;
use super::{Timetable, Type, Which};

impl Bakalari {
    /// Test api connection
    ///
    /// # Errors
    /// If request fails
    ///
    /// # Panics
    /// If url join fails
    pub async fn test(&self) -> RequestResult<()> {
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

    /// Get timetable
    ///
    /// # Errors
    /// If request fails
    ///
    /// # Panics
    /// If url join fails (shouldn't)
    pub async fn get_timetable(
        &self,
        which: Which,
        timetable_type: &Type<'_>,
    ) -> RequestResult<Timetable> {
        let client = self.client();
        let res = client
            .reqwest_client()
            .get(
                client
                    .url()
                    .join(&format!("/timetable/public/{which}/{timetable_type}"))
                    .unwrap(),
            )
            .header("Cookie", format!("BakaAuth={}", self.get_token().await?))
            .send()
            .await?;

        let html = res.text().await?;

        let start = Instant::now();

        let res = Ok(Timetable::parse(&html, timetable_type)?);

        println!("parsing took {:?}", start.elapsed());

        res
    }
}
