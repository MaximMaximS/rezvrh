use super::{Bakalari, RequestError, RequestResult};

impl Bakalari {
    /// Test if connection is working
    ///
    /// # Errors
    /// Returns error if request fails
    ///
    /// # Panics
    /// If url join fails (shouldn't)
    pub async fn test(&self) -> RequestResult<()> {
        let client = self.client();
        let res = client
            .reqwest_client()
            .get(client.url().join("timetable/public").unwrap())
            .header("Cookie", format!("BakaAuth={}", self.get_token().await?))
            .send()
            .await?;

        let text = res.text().await?;

        if !text.contains("timetable") {
            return Err(RequestError::UnknownResponse("timetable not present"));
        }

        Ok(())
    }
}
