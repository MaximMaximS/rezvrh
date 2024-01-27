use super::{Day, Hour, Lesson, Timetable, Type};
use chrono::NaiveTime;
use once_cell::sync::Lazy;
use scraper::{Element, ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
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
static DAY_CELL_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.bk-timetable-cell").unwrap());
static DAY_ITEM_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.day-item").unwrap());
static LESSON_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.day-item-hover").unwrap());
static LESSON_ABBR_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.middle").unwrap());
static LESSON_TEACHER_ABBR_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.bottom").unwrap());

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
        let duration = duration.num_minutes();

        Ok(Self {
            start: from,
            duration,
        })
    }
}

/// Error that can occur while parsing lesson
#[derive(Debug, Error)]
pub enum LessonError {
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

use serde::de::IntoDeserializer;

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    let opt = opt.as_deref();
    match opt {
        None | Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer()).map(Some),
    }
}

// {"type":"atom","subjecttext":"Český jazyk a literatura | po 22.1. | 2 (8:55 - 9:40)","teacher":"Mgr. Ivona Liptáková","room":"207","group":"","theme":"Anarchističtí buřiči - P. Bezruč","notice":"","changeinfo":"","homeworks":null,"absencetext":null,"hasAbsent":false,"absentInfoText":""}

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

impl Lesson {
    fn get_prop(
        elem: ElementRef,
        selector: &Selector,
        prop: &'static str,
    ) -> Result<String, LessonError> {
        let elem = single_iter(elem.select(selector), || LessonError::MissingProperty(prop))?;
        let elem = single_iter(elem.text(), || LessonError::MissingProperty(prop))?;
        Ok(elem.trim().to_owned())
    }
    #[allow(clippy::too_many_lines)]
    fn parse_single(lesson: ElementRef, timetable_type: &Type) -> Result<Self, LessonError> {
        let data = lesson
            .value()
            .attr("data-detail")
            .ok_or(LessonError::NoData)?;
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

                let subjecttext = subjecttext.ok_or(LessonError::MissingProperty("subjecttext"))?;

                let (subject, _) = subjecttext
                    .split_once(" | ")
                    .ok_or_else(|| LessonError::BadSubjectText(subjecttext.clone()))?;
                if subject.is_empty() {
                    return Err(LessonError::BadSubjectText(subjecttext.clone()));
                }
                let subject = subject.trim().to_owned();

                let abbr = Self::get_prop(lesson, &LESSON_ABBR_SELECTOR, "abbr")?;

                let teacher = match teacher {
                    Some(t) => t,
                    None => {
                        if let Type::Teacher(t) = timetable_type {
                            t.to_owned().to_owned()
                        } else {
                            return Err(LessonError::MissingProperty("teacher"));
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
                    .ok_or(LessonError::MissingProperty("room"))?
                    .trim()
                    .to_owned();

                let topic = theme;

                let class = if let Type::Class(class) = timetable_type {
                    class.to_owned().to_owned()
                } else {
                    group
                        .as_ref()
                        .ok_or(LessonError::MissingProperty("group"))?
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
                        info_absent_name.ok_or(LessonError::MissingProperty("info_absent_name"))?;

                    let abbr = absent_info.ok_or(LessonError::MissingProperty("absent_info"))?;

                    Ok(Self::Absent { info, abbr })
                } else {
                    Err(LessonError::DataTypeMismatch)
                }
            }
            LessonData::Canceled { subjecttext: _ } => {
                if lesson.has_class(
                    &"pink".into(),
                    scraper::CaseSensitivity::AsciiCaseInsensitive,
                ) {
                    Ok(Self::Canceled)
                } else {
                    Err(LessonError::DataTypeMismatch)
                }
            }
        }
    }

    pub fn parse(lesson: ElementRef, timetable_type: &Type) -> Result<Vec<Self>, LessonError> {
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
            .select(&DAY_CELL_SELECTOR)
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
