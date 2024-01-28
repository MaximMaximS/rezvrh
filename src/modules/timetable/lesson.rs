use super::{
    util::{empty_string_as_none, single_iter},
    Type,
};
use once_cell::sync::Lazy;
use scraper::{Element, ElementRef, Selector};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum LessonData {
    #[serde(rename = "atom")]
    Regular {
        #[serde(deserialize_with = "empty_string_as_none")]
        subjecttext: Option<String>,
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
static LESSON_ABBR_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.middle").unwrap());
static LESSON_TEACHER_ABBR_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.bottom").unwrap());
static DAY_ITEM_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.day-item").unwrap());

impl Lesson {
    fn get_prop(
        elem: ElementRef,
        selector: &Selector,
        prop: &'static str,
    ) -> Result<String, ParseError> {
        let elem = single_iter(elem.select(selector), || ParseError::MissingProperty(prop))?;
        let elem = single_iter(elem.text(), || ParseError::MissingProperty(prop))?;
        Ok(elem.trim().to_owned())
    }
    #[allow(clippy::too_many_lines)]
    fn parse_single(lesson: ElementRef, timetable_type: &Type) -> Result<Self, ParseError> {
        let data = lesson
            .value()
            .attr("data-detail")
            .ok_or(ParseError::NoData)?;
        let data = serde_json::from_str::<LessonData>(data)?;

        match data {
            LessonData::Regular {
                subjecttext,
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

                let subjecttext = subjecttext.ok_or(ParseError::MissingProperty("subjecttext"))?;

                let (subject, _) = subjecttext
                    .split_once(" | ")
                    .ok_or_else(|| ParseError::BadSubjectText(subjecttext.clone()))?;
                if subject.is_empty() {
                    return Err(ParseError::BadSubjectText(subjecttext.clone()));
                }
                let subject = subject.trim().to_owned();

                let abbr = Self::get_prop(lesson, &LESSON_ABBR_SELECTOR, "abbr")?;

                let teacher = match teacher {
                    Some(t) => t,
                    None => {
                        if let Type::Teacher(t) = timetable_type {
                            t.to_owned().to_owned()
                        } else {
                            return Err(ParseError::MissingProperty("teacher"));
                        }
                    }
                };

                let teacher_abbr =
                    Self::get_prop(lesson, &LESSON_TEACHER_ABBR_SELECTOR, "teacher_abbr");

                let teacher_abbr = match teacher_abbr {
                    Ok(abbr) => Some(abbr),
                    Err(e) => {
                        if let Type::Teacher(_) = timetable_type {
                            None
                        } else {
                            return Err(e);
                        }
                    }
                };

                let room = room
                    .ok_or(ParseError::MissingProperty("room"))?
                    .trim()
                    .to_owned();

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
                    Ok(Self::Substitution {
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
                    Ok(Self::Regular {
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

                    Ok(Self::Absent { info, abbr })
                } else {
                    Err(ParseError::DataTypeMismatch)
                }
            }
            LessonData::Canceled { subjecttext: _ } => {
                if lesson.has_class(
                    &"pink".into(),
                    scraper::CaseSensitivity::AsciiCaseInsensitive,
                ) {
                    Ok(Self::Canceled)
                } else {
                    Err(ParseError::DataTypeMismatch)
                }
            }
        }
    }

    pub fn parse(lesson: ElementRef, timetable_type: &Type) -> Result<Vec<Self>, ParseError> {
        let item = lesson.select(&DAY_ITEM_SELECTOR).next();
        let Some(item) = item else {
            return Ok(Vec::new());
        };

        let lessons = item
            .select(&LESSON_SELECTOR)
            .map(|lesson| Self::parse_single(lesson, timetable_type))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(lessons)
    }
}
