mod modules;

pub use modules::bakalari::Bakalari;
pub use modules::bakalari::RequestError as Error;
pub use modules::timetable::Lesson;
pub use modules::timetable::RawType as Type;
pub use modules::timetable::Timetable;
pub use modules::timetable::Type as Selector;
pub use modules::timetable::Which;
