use chrono::{Duration, NaiveDate, NaiveTime};
use derive_more::Display;
use grid::Grid;

pub use parser::TimetableError as ParseTimetableError;

mod api;
mod parser;

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

/// Timetable type
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Type {
    Teacher,
    Class,
    Room,
}

#[derive(Debug)]
pub struct Lesson {}

#[derive(Debug)]
pub struct Day {
    date: Option<String>,
    name: String,
    lessons: Vec<Option<Lesson>>,
}

#[derive(Debug)]
pub struct Hour {
    start: NaiveTime,
    duration: Duration,
}

#[derive(Debug)]
pub struct Timetable {
    hours: Vec<Hour>,
    days: Vec<Day>,
}
