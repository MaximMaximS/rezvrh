use crate::{modules::timetable::Type, Bakalari};

impl Bakalari {
    /// Get list of classes
    #[must_use]
    pub fn get_classes(&self) -> Vec<String> {
        self.classes
            .keys()
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    }

    /// Get class selector
    #[must_use]
    pub fn get_class(&self, class: &str) -> Option<Type> {
        self.classes.get(class).map(|id| Type::Class(id))
    }

    /// Get list of teachers
    #[must_use]
    pub fn get_teachers(&self) -> Vec<String> {
        self.teachers
            .keys()
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    }

    /// Get teacher selector
    #[must_use]
    pub fn get_teacher(&self, teacher: &str) -> Option<Type> {
        self.teachers.get(teacher).map(|id| Type::Teacher(id))
    }

    /// Get list of rooms
    #[must_use]
    pub fn get_rooms(&self) -> Vec<String> {
        self.rooms
            .keys()
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    }

    /// Get room selector
    #[must_use]
    pub fn get_room(&self, room: &str) -> Option<Type> {
        self.rooms.get(room).map(|id| Type::Room(id))
    }
}
