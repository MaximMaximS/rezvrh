use super::{Bakalari, RequestResult};
use crate::modules::timetable::{Timetable, Type, Which};

impl Bakalari {
    /// Get specific timetable
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

        Ok(Timetable::parse(&html, timetable_type)?)
    }
}
