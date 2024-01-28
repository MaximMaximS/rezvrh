use self::parser::subject;

use super::{
    util::{empty_string_as_none, single_iter},
    Type,
};
use once_cell::sync::Lazy;
use scraper::{Element, ElementRef, Selector};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod parser;

/// Struct that hold one lesson of timetable
#[derive(Debug, Serialize, Deserialize)]
pub enum Lesson {
    Regular {
        class: String,
        subject: String,
        abbr: String,
        teacher: String,
        teacher_abbr: Option<String>,
        room: Option<String>,
        group: Option<String>,
        topic: Option<String>,
    },
    Substitution {
        class: String,
        subject: String,
        abbr: String,
        teacher: String,
        teacher_abbr: Option<String>,
        room: Option<String>,
        group: Option<String>,
        topic: Option<String>,
    },
    Canceled,
    Absent {
        info: String,
        abbr: String,
    },
}

/// Data that is stored in data-detail attribute
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum LessonData {
    #[serde(rename = "atom")]
    Regular {
        #[serde(rename = "subjecttext")]
        #[serde(deserialize_with = "empty_string_as_none")]
        subject_text: Option<String>,
        #[serde(deserialize_with = "empty_string_as_none")]
        teacher: Option<String>,
        #[serde(deserialize_with = "empty_string_as_none")]
        room: Option<String>,
        #[serde(deserialize_with = "empty_string_as_none")]
        group: Option<String>,
        #[serde(deserialize_with = "empty_string_as_none")]
        theme: Option<String>,
        #[serde(deserialize_with = "empty_string_as_none")]
        notice: Option<String>,
        #[serde(deserialize_with = "empty_string_as_none")]
        changeinfo: Option<String>,
        #[serde(deserialize_with = "empty_string_as_none")]
        homeworks: Option<String>,
        #[serde(deserialize_with = "empty_string_as_none")]
        absencetext: Option<String>,
        #[serde(rename = "hasAbsent")]
        has_absent: bool,
        #[serde(rename = "absentInfoText")]
        #[serde(deserialize_with = "empty_string_as_none")]
        absent_info_text: Option<String>,
    },
    #[serde(rename = "removed")]
    Canceled {
        #[serde(deserialize_with = "empty_string_as_none")]
        subjecttext: Option<String>,
    },
    #[serde(rename = "absent")]
    Absent {
        #[serde(rename = "InfoAbsentName")]
        #[serde(deserialize_with = "empty_string_as_none")]
        info_absent_name: Option<String>,
        #[serde(rename = "absentinfo")]
        #[serde(deserialize_with = "empty_string_as_none")]
        absent_info: Option<String>,
    },
}

/// Lesson parse error
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("missing data-detail attribute")]
    NoData,
    #[error("failed to parse json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("missing property: {0}")]
    MissingProperty(&'static str),
    #[error("bad subjecttext: {0}")]
    BadSubjectText(String),
    #[error("data type mismatch")]
    DataTypeMismatch,
}

static LESSON_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.day-item-hover").unwrap());
static ABBR_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.middle").unwrap());

static DAY_ITEM_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.day-item").unwrap());

fn get_prop(
    elem: ElementRef,
    selector: &Selector,
    prop: &'static str,
) -> Result<String, ParseError> {
    let elem = single_iter(elem.select(selector), || ParseError::MissingProperty(prop))?;
    let elem = single_iter(elem.text(), || ParseError::MissingProperty(prop))?;
    Ok(elem.trim().to_owned())
}

fn parse_single(lesson: ElementRef, timetable_type: &Type) -> Result<Lesson, ParseError> {
    let data = lesson
        .value()
        .attr("data-detail")
        .ok_or(ParseError::NoData)?;
    let data = serde_json::from_str::<LessonData>(data)?;

    match data {
        LessonData::Regular {
            subject_text,
            teacher,
            room,
            group,
            theme,
            notice: _,
            changeinfo: _,
            homeworks: _,
            absencetext: _,
            has_absent: _,
            absent_info_text: _,
        } => {
            let substituion = lesson.has_class(
                &"pink".into(),
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            );

            let subject = subject(subject_text)?;

            let abbr = get_prop(lesson, &ABBR_SELECTOR, "abbr")?;

            let (teacher, teacher_abbr) = parser::teacher(lesson, teacher, timetable_type)?;

            let topic = theme;

            let class = if let Type::Class(class) = timetable_type {
                class.to_owned().to_owned()
            } else {
                group
                    .as_ref()
                    .ok_or(ParseError::MissingProperty("group"))?
                    .to_owned()
            };

            if substituion {
                Ok(Lesson::Substitution {
                    class,
                    subject,
                    abbr,
                    teacher,
                    teacher_abbr,
                    room,
                    group,
                    topic,
                })
            } else {
                Ok(Lesson::Regular {
                    class,
                    subject,
                    abbr,
                    teacher,
                    teacher_abbr,
                    room,
                    group,
                    topic,
                })
            }
        }
        LessonData::Absent {
            info_absent_name,
            absent_info,
        } => {
            if lesson.has_class(
                &"green".into(),
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            ) {
                let info =
                    info_absent_name.ok_or(ParseError::MissingProperty("info_absent_name"))?;

                let abbr = absent_info.ok_or(ParseError::MissingProperty("absent_info"))?;

                Ok(Lesson::Absent { info, abbr })
            } else {
                Err(ParseError::DataTypeMismatch)
            }
        }
        LessonData::Canceled { subjecttext: _ } => {
            if lesson.has_class(
                &"pink".into(),
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            ) {
                Ok(Lesson::Canceled)
            } else {
                Err(ParseError::DataTypeMismatch)
            }
        }
    }
}

impl Lesson {
    pub fn parse(lesson: ElementRef, timetable_type: &Type) -> Result<Vec<Self>, ParseError> {
        let item = lesson.select(&DAY_ITEM_SELECTOR).next();
        let Some(item) = item else {
            return Ok(Vec::new());
        };

        let lessons = item
            .select(&LESSON_SELECTOR)
            .map(|lesson| parse_single(lesson, timetable_type))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(lessons)
    }
}
