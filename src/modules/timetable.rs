use chrono::{Duration, NaiveDate, NaiveTime};
use derive_more::Display;
use grid::Grid;
use lesson::Lesson;

pub use parser::TimetableError as ParseTimetableError;

mod api;
mod lesson;
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

pub struct Hour {
    start: NaiveTime,
    duration: Duration,
}

pub struct Timetable {
    start: NaiveDate,
    hours: Vec<Hour>,
    lessons: Grid<Lesson>,
}
