use super::{Hour, Timetable, Type};
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
    pub fn parse(hour: ElementRef, cnt: &mut usize) -> Result<Self, HourError> {
        //let hour_selector = Selector::parse("div.bk-hour-wrapper").unwrap();

        let num = single_iter(hour.select(&NUM_SELECTOR), || HourError::NoNum)?;
        let num = single_iter(num.text(), || HourError::NoNumText)?;
        let num = num.parse::<usize>().map_err(HourError::ParseNum)?;
        if num != *cnt {
            return Err(HourError::MismatchedNum);
        }
        *cnt += 1;

        let mut times = hour.select(&TIMES_SELECTOR);
        let from = times.next().ok_or(HourError::NoFrom)?;
        times.next().ok_or(HourError::NoDash)?;
        let to = single_iter(times, || HourError::NoTo)?;

        let from = single_iter(from.text(), || HourError::NoFromText)?;
        let from = NaiveTime::parse_from_str(from, "%H:%M").map_err(HourError::ParseFrom)?;

        let to = single_iter(to.text(), || HourError::NoToText)?;
        let to = NaiveTime::parse_from_str(to, "%H:%M").map_err(HourError::ParseTo)?;
        let duration = to - from;

        println!("{}: {} - {} ({})", num, from, to, duration.num_minutes());

        Ok(Self {
            start: from,
            duration,
        })
    }
}

/// Error that can occur while parsing timetable
#[derive(Debug, Error)]
pub enum TimetableError {
    #[error("failed to parse hour: {0}")]
    Hour(#[from] HourError),
}

impl Timetable {
    pub fn parse(html: &str, table_type: Type) -> Result<Self, TimetableError> {
        fs::write("/tmp/timetable", html).unwrap();

        let document = Html::parse_document(html);

        let mut cnt = 1usize;

        let hours = document
            .select(&HOUR_SELECTOR)
            .map(|hour| Hour::parse(hour, &mut cnt))
            .collect::<Result<Vec<_>, HourError>>()?;

        todo!()
    }
}
