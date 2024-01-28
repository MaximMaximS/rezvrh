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

/// Which timetable to get
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Which {
    /// Permanent timetable
    #[display("permanent")]
    Permanent,
    /// Timetable for current week
    #[display("actual")]
    Actual,
    /// Timetable for next week
    #[display("next")]
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
pub enum Type<'a> {
    #[display("teacher/{_0}")]
    Teacher(&'a str),
    #[display("class/{_0}")]
    Class(&'a str),
    #[display("room/{_0}")]
    Room(&'a str),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Timetable {
    hours: Vec<Hour>,
    days: Vec<Day>,
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
    pub fn parse(html: &str, table_type: &Type) -> Result<Self, ParseError> {
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
