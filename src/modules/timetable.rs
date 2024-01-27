use chrono::NaiveTime;
use derive_more::Display;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Display, PartialEq, Eq, Hash, Clone)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Lesson {
    Regular {
        class: String,
        subject: String,
        abbr: String,
        teacher: String,
        teacher_abbr: Option<String>,
        room: String,
        group: Option<String>,
        topic: Option<String>,
    },
    Substitution {
        class: String,
        subject: String,
        abbr: String,
        teacher: String,
        teacher_abbr: Option<String>,
        room: String,
        group: Option<String>,
        topic: Option<String>,
    },
    Canceled,
    Absent {
        info: String,
        abbr: String,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Day {
    date: Option<String>,
    name: String,
    lessons: Vec<Vec<Lesson>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Hour {
    start: NaiveTime,
    duration: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Timetable {
    hours: Vec<Hour>,
    days: Vec<Day>,
}
