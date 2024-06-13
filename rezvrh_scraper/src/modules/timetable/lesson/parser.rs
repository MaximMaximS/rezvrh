use once_cell::sync::Lazy;
use scraper::{ElementRef, Selector};

use crate::modules::timetable::Type;

use super::{get_prop, ParseError};

type ParseResult<T> = Result<T, ParseError>;

/// Parse subject from subjecttext
pub fn subject(s: Option<String>) -> ParseResult<String> {
    let subjecttext = s.ok_or(ParseError::MissingProperty("subjecttext"))?;

    let (subject, _) = subjecttext
        .split_once(" | ")
        .ok_or_else(|| ParseError::BadSubjectText(subjecttext.clone()))?;
    if subject.is_empty() {
        return Err(ParseError::BadSubjectText(subjecttext.clone()));
    }
    Ok(subject.trim().to_owned())
}

static TEACHER_ABBR_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.bottom").unwrap());

/// Parse teacher
pub fn teacher(
    lesson: ElementRef,
    teacher: Option<String>,
    timetable_type: &Type,
) -> ParseResult<(String, Option<String>)> {
    let teacher = match teacher {
        Some(t) => t,
        None => {
            if let Type::Teacher(t) = timetable_type {
                t.to_owned()
            } else {
                return Err(ParseError::MissingProperty("teacher"));
            }
        }
    };

    let teacher_abbr = if let Type::Teacher(_) = timetable_type {
        None
    } else {
        Some(get_prop(lesson, &TEACHER_ABBR_SELECTOR, "teacher_abbr")?)
    };

    Ok((teacher, teacher_abbr))
}
