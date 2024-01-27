use super::{Day, Hour, Lesson, Timetable, Type};
use chrono::NaiveTime;
use once_cell::sync::Lazy;
use scraper::{ElementRef, Html, Selector};
use std::fs;
use thiserror::Error;

// Get first element from iterator, or return error if there is more than one or none
fn single_iter<T, I, E, F>(mut iter: I, err: F) -> Result<T, E>
where
    I: Iterator<Item = T>,
    F: FnOnce() -> E + Copy,
{
    let first = iter.next().ok_or_else(err)?;
    if iter.next().is_some() {
        return Err(err());
    }
    Ok(first)
}

static HOUR_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.bk-hour-wrapper").unwrap());
static NUM_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.num").unwrap());
static TIMES_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.hour > span").unwrap());
static DAY_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.bk-timetable-row").unwrap());
static DAY_NAME_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("span.bk-day-day").unwrap());
static DAY_DATE_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("span.bk-day-date").unwrap());
static DAY_LESSON_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.bk-timetable-cell").unwrap());
static DAY_ITEM_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.day-item").unwrap());

/// Error that can occur while parsing hour
#[derive(Debug, Error)]
pub enum HourError {
    #[error("no number")]
    NoNum,
    #[error("no number text")]
    NoNumText,
    #[error("failed to parse number: {0}")]
    ParseNum(std::num::ParseIntError),
    #[error("mismatched number")]
    MismatchedNum,
    #[error("no from")]
    NoFrom,
    #[error("no from text")]
    NoFromText,
    #[error("failed to parse from: {0}")]
    ParseFrom(chrono::ParseError),
    #[error("no dash")]
    NoDash,
    #[error("no to")]
    NoTo,
    #[error("no to text")]
    NoToText,
    #[error("failed to parse to: {0}")]
    ParseTo(chrono::ParseError),
}

impl Hour {
    pub fn parse(hour: ElementRef, i: usize) -> Result<Self, HourError> {
        let num = single_iter(hour.select(&NUM_SELECTOR), || HourError::NoNum)?;
        let num = single_iter(num.text(), || HourError::NoNumText)?;
        let num = num.parse::<usize>().map_err(HourError::ParseNum)?;
        if num != i + 1 {
            return Err(HourError::MismatchedNum);
        }

        let mut times = hour.select(&TIMES_SELECTOR);
        let from = times.next().ok_or(HourError::NoFrom)?;
        times.next().ok_or(HourError::NoDash)?;
        let to = single_iter(times, || HourError::NoTo)?;

        let from = single_iter(from.text(), || HourError::NoFromText)?;
        let from = NaiveTime::parse_from_str(from, "%H:%M").map_err(HourError::ParseFrom)?;

        let to = single_iter(to.text(), || HourError::NoToText)?;
        let to = NaiveTime::parse_from_str(to, "%H:%M").map_err(HourError::ParseTo)?;
        let duration = to - from;

        Ok(Self {
            start: from,
            duration,
        })
    }
}

/// Error that can occur while parsing lesson
#[derive(Debug, Error)]
pub enum LessonError {}

impl Lesson {
    pub fn parse(lesson: ElementRef, timetable_type: &Type) -> Result<Option<Self>, LessonError> {
        fs::write("/tmp/lesson", lesson.html()).unwrap();

        let item = lesson.select(&DAY_ITEM_SELECTOR).next();
        let Some(item) = item else {
            return Ok(None);
        };

        Ok(Some(Self {}))
    }
}

/// Error that can occur while parsing day
#[derive(Debug, Error)]
pub enum DayError {
    #[error("no name")]
    NoName,
    #[error("no name text")]
    NoNameText,
    #[error("no date")]
    NoDate,
    #[error("failed to parse date: {0}")]
    ParseDate(chrono::ParseError),
    #[error("no lessons")]
    NoLessons,
    #[error("failed to parse lesson: {0}")]
    Lesson(#[from] LessonError),
}

impl Day {
    pub fn parse(day: ElementRef, timetable_type: &Type) -> Result<Self, DayError> {
        let name = single_iter(day.select(&DAY_NAME_SELECTOR), || DayError::NoName)?;
        let name = single_iter(name.text(), || DayError::NoNameText)?
            .trim()
            .to_owned();

        let mut dates = single_iter(day.select(&DAY_DATE_SELECTOR), || DayError::NoDate)?.text();
        let date = dates.next().map(|d| d.trim().to_owned());
        if date.is_some() && dates.next().is_some() {
            return Err(DayError::NoDate);
        }

        let lessons = day
            .select(&DAY_LESSON_SELECTOR)
            .map(|lesson| Lesson::parse(lesson, timetable_type))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            date,
            name,
            lessons,
        })
    }
}

/// Error that can occur while parsing timetable
#[derive(Debug, Error)]
pub enum TimetableError {
    #[error("failed to parse hour: {0}")]
    Hour(#[from] HourError),
    #[error("failed to parse day: {0}")]
    Day(#[from] DayError),
}

impl Timetable {
    pub fn parse(html: &str, table_type: &Type) -> Result<Self, TimetableError> {
        fs::write("/tmp/timetable", html).unwrap();

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
