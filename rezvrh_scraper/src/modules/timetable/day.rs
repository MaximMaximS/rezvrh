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

static DATE_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("span.bk-day-date").unwrap());
static CELL_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.bk-timetable-cell").unwrap());

impl Day {
    /// Parse day from html
    pub fn parse(day: ElementRef, timetable_type: &Type) -> Result<Self, ParseError> {
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
                let date = NaiveDate::from_ymd_opt(
                    year,
                    month
                        .parse()
                        .map_err(|_| ParseError::ParseDate(d.clone()))?,
                    day.parse().map_err(|_| ParseError::ParseDate(d.clone()))?,
                )
                .ok_or(ParseError::ParseDate(d.clone()))?;

                // Check diff by months
                let now = chrono::Local::now().date_naive();
                let diff = date - now;
                let date = if diff.num_days() < -60 {
                    // Next year
                    NaiveDate::from_ymd_opt(year + 1, date.month(), date.day())
                        .ok_or(ParseError::ParseDate(d.clone()))?
                } else if diff.num_days() > 60 {
                    // Last year
                    NaiveDate::from_ymd_opt(year - 1, date.month(), date.day())
                        .ok_or(ParseError::ParseDate(d.clone()))?
                } else {
                    date
                };
                Ok::<_, ParseError>(date)
            })
            .transpose()?;

        let lessons = day
            .select(&CELL_SELECTOR)
            .map(|lesson| Lesson::parse(lesson, timetable_type))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { date, lessons })
    }
}
