use super::util::single_iter;
use chrono::NaiveTime;
use once_cell::sync::Lazy;
use scraper::{ElementRef, Selector};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Struct that hold one hour of timetable (header)
#[derive(Debug, Serialize, Deserialize)]
pub struct Hour {
    start: NaiveTime,
    duration: i64,
}

/// Hour parse error
#[derive(Debug, Error)]
pub enum ParseError {
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

static NUM_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.num").unwrap());
static TIMES_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.hour > span").unwrap());

impl Hour {
    /// Parse hour from html
    pub fn parse(hour: ElementRef, i: usize) -> Result<Self, ParseError> {
        let num = single_iter(hour.select(&NUM_SELECTOR), || ParseError::NoNum)?;
        let num = single_iter(num.text(), || ParseError::NoNumText)?;
        let num = num.parse::<usize>().map_err(ParseError::ParseNum)?;
        if num != i + 1 {
            return Err(ParseError::MismatchedNum);
        }

        let mut times = hour.select(&TIMES_SELECTOR);
        let from = times.next().ok_or(ParseError::NoFrom)?;
        times.next().ok_or(ParseError::NoDash)?;
        let to = single_iter(times, || ParseError::NoTo)?;

        let from = single_iter(from.text(), || ParseError::NoFromText)?;
        let from = NaiveTime::parse_from_str(from, "%H:%M").map_err(ParseError::ParseFrom)?;

        let to = single_iter(to.text(), || ParseError::NoToText)?;
        let to = NaiveTime::parse_from_str(to, "%H:%M").map_err(ParseError::ParseTo)?;
        let duration = to - from;
        let duration = duration.num_minutes();

        Ok(Self {
            start: from,
            duration,
        })
    }
}
