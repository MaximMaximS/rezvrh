use super::lesson::ParseError as LessonParseError;
use super::{lesson::Lesson, util::single_iter, Type};
use chrono::{Datelike, NaiveDate};
use once_cell::sync::Lazy;
use scraper::{ElementRef, Selector};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Struct that hold one day of timetable
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Day {
    date: Option<NaiveDate>,
    name: String,
    lessons: Vec<Vec<Lesson>>,
}

/// Day parse error
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("no name")]
    NoName,
    #[error("no name text")]
    NoNameText,
    #[error("no date")]
    NoDate,
    #[error("failed to parse date: {0}")]
    ParseDate(String),
    #[error("no lessons")]
    NoLessons,
    #[error("failed to parse lesson: {0}")]
    Lesson(#[from] LessonParseError),
}

static NAME_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("span.bk-day-day").unwrap());
static DATE_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("span.bk-day-date").unwrap());
static CELL_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.bk-timetable-cell").unwrap());

impl Day {
    /// Parse day from html
    pub fn parse(day: ElementRef, timetable_type: &Type) -> Result<Self, ParseError> {
        let name = single_iter(day.select(&NAME_SELECTOR), || ParseError::NoName)?;
        let name = single_iter(name.text(), || ParseError::NoNameText)?
            .trim()
            .to_owned();

        let mut dates = single_iter(day.select(&DATE_SELECTOR), || ParseError::NoDate)?.text();
        let date = dates.next().map(|d| d.trim().to_owned());
        if date.is_some() && dates.next().is_some() {
            return Err(ParseError::NoDate);
        }

        let date = date
            .map(|d| {
                // Current year
                let year = chrono::Local::now().date_naive().year();
                // 12.6.
                let (day, month) = d.split_once('.').ok_or(ParseError::ParseDate(d.clone()))?;
                let (month, _) = month
                    .split_once('.')
                    .ok_or(ParseError::ParseDate(d.clone()))?;
                NaiveDate::from_ymd_opt(
                    year,
                    month
                        .parse()
                        .map_err(|_| ParseError::ParseDate(d.clone()))?,
                    day.parse().map_err(|_| ParseError::ParseDate(d.clone()))?,
                )
                .ok_or(ParseError::ParseDate(d.clone()))
            })
            .transpose()?;

        let lessons = day
            .select(&CELL_SELECTOR)
            .map(|lesson| Lesson::parse(lesson, timetable_type))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            date,
            name,
            lessons,
        })
    }
}
