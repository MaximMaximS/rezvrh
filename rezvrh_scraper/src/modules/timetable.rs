use day::ParseError as DayParseError;
use derive_more::Display;
use hour::ParseError as HourParseError;
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use {day::Day, hour::Hour};

mod day;
mod hour;
mod lesson;
mod util;

pub use lesson::Lesson;

/// Which timetable to get
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Which {
    /// Permanent timetable
    Permanent,
    /// Timetable for current week
    Actual,
    /// Timetable for next week
    Next,
}

#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RawType {
    Teacher,
    Class,
    Room,
}

/// Timetable type
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone)]
pub enum Type {
    #[display("teacher/{_0}")]
    Teacher(String),
    #[display("class/{_0}")]
    Class(String),
    #[display("room/{_0}")]
    Room(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Timetable {
    pub hours: Vec<Hour>,
    pub days: Vec<Day>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("failed to parse hour: {0}")]
    Hour(#[from] HourParseError),
    #[error("failed to parse day: {0}")]
    Day(#[from] DayParseError),
}

static HOUR_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.bk-hour-wrapper").unwrap());
static DAY_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.bk-timetable-row").unwrap());

impl Timetable {
    pub(super) fn parse(html: &str, table_type: &Type) -> Result<Self, ParseError> {
        let document = Html::parse_document(html);

        let hours = document
            .select(&HOUR_SELECTOR)
            .enumerate()
            .map(|(i, hour)| Hour::parse(hour, i))
            .collect::<Result<Vec<_>, _>>()?;

        let days = document
            .select(&DAY_SELECTOR)
            .map(|day| Day::parse(day, table_type))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { hours, days })
    }
}
