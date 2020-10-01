use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClassHour {
    pub id: usize,
    pub number: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Room {
    pub id: usize,
    pub name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subject {
    pub id: usize,
    pub abbreviation: String,
    pub name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Teacher {
    pub id: usize,
    pub abbreviation: String,
    pub firstname: Option<String>,
    pub lastname: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Class {
    pub id: usize,
    pub name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StudentGroup {
    pub id: usize,
    pub name: String,
    pub classId: Option<usize>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActualLesson {
    pub room: Room,
    pub subject: Subject,
    pub teachers: Vec<Teacher>,
    pub classes: Vec<Class>,
    pub studentGroups: Vec<StudentGroup>,
    pub comment: Option<String>,
    pub subjectLabel: String,
    pub lessonId: usize,
    pub substitutionId: Option<usize>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OriginalLesson {
    pub room: Room,
    pub subject: Subject,
    pub teachers: Vec<Teacher>,
    pub classes: Vec<Class>,
    pub studentGroups: Vec<StudentGroup>,
    pub comment: Option<String>,
    pub subjectLabel: String,
    pub lessonId: usize
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Datum {
    pub date: String,
    pub classHour: ClassHour,
    pub actualLesson: Option<ActualLesson>,
    pub comment: Option<String>,
    pub originalLessons: Option<Vec<OriginalLesson>>,
    pub isSubstitution: Option<bool>,
    pub isCancelled: Option<bool>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SmTimetableResp {
    pub status: u16,
    pub data: Vec<Datum>
}
