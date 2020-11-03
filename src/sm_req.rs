#![allow(non_snake_case)]
pub mod SmLoginStatus {
    use serde::Deserialize;
    #[derive(Deserialize, Debug, Clone)]
    pub struct Student {
        pub id: usize,
        pub firstname: Option<String>,
        pub lastname: Option<String>,
        sex: Option<String>,
        pub classId: usize
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct User {
        pub email: Option<String>,
        pub username: Option<String>,
        pub localUsername: Option<String>,
        pub id: usize,
        hasAdministratorRights: bool,
        lastSeenNotificationTimestamp: Option<String>,
        pub firstname: Option<String>,
        pub lastname: Option<String>,
        pub associatedStudent: Student
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Status {
        pub isAuthenticated: bool,
        pub user: User
    }
}

pub mod SmTimetableRequest {
    use serde::Serialize;
    use chrono::{Date, Weekday, Datelike, Local, TimeZone};
    /*  Thanks to harmic for his brilliant stackoverflow answer
    https://stackoverflow.com/questions/64174950/get-date-of-start-end-of-week */
    fn week_bounds(week: u32, year: i32) -> (Date<Local>, Date<Local>) {
        let mon: Date<Local> = Local.isoywd(year, week, Weekday::Mon);
        let sun: Date<Local> = Local.isoywd(year, week, Weekday::Sun);
        (mon, sun)
    }

    #[derive(Serialize, Debug)]
    pub struct TimetableBodyParamsStudent {
        pub id: usize,
        pub classId: usize
    }

    #[derive(Serialize, Debug)]
    pub struct TimetableBodyParams {
        pub student: TimetableBodyParamsStudent,
        pub start: String,
        pub end: String
    }

    #[derive(Serialize, Debug)]
    pub struct Body {
        pub moduleName: String,
        pub endpointName: String,
        pub parameters: TimetableBodyParams
    }
    impl Body {
        pub fn new(id: usize, class_id: usize, week: u32, oyear: Option<i32>) -> Self {
            let year = oyear.unwrap_or(Local::now().year());
            let (mon, sun) = week_bounds(week, year);
            Body {
                moduleName: String::from("schedules"),
                endpointName: String::from("get-actual-lessons"),
                parameters: TimetableBodyParams {
                    student: TimetableBodyParamsStudent {
                        id: id,
                        classId: class_id
                    },
                    start: mon.format("%F").to_string(),
                    end: sun.format("%F").to_string()
                }
            }
        }
    }
}

pub mod SmTimetableResponse {
    use serde::Deserialize;
    #[derive(Deserialize, Debug, Clone)]
    pub struct ClassHour {
        pub id: usize,
        pub number: String
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Room {
        pub id: usize,
        pub name: String
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Subject {
        pub id: usize,
        pub abbreviation: String,
        pub name: String
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Teacher {
        pub id: usize,
        pub abbreviation: String,
        pub firstname: Option<String>,
        pub lastname: Option<String>
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Class {
        pub id: usize,
        pub name: String
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct StudentGroup {
        pub id: usize,
        pub name: String,
        pub classId: Option<usize>
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Event {
        pub text: String,
        pub teachers: Vec<Teacher>,
        pub classes: Vec<Class>,
        pub studentGroups: Vec<StudentGroup>,
        pub absenceId: usize
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct ActualLesson {
        pub room: Room,
        pub subject: Subject,
        pub teachers: Vec<Teacher>,
        pub classes: Vec<Class>,
        pub studentGroups: Vec<StudentGroup>,
        pub comment: Option<String>,
        pub subjectLabel: String,
        pub lessonId: Option<usize>,
        pub substitutionId: Option<usize>
    }

    #[derive(Deserialize, Debug, Clone)]
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

    #[derive(Deserialize, Debug)]
    pub struct Datum {
        pub date: String,
        pub classHour: ClassHour,
        pub actualLesson: Option<ActualLesson>,
        pub comment: Option<String>,
        pub originalLessons: Option<Vec<OriginalLesson>>,
        pub event: Option<Event>,
        pub isSubstitution: Option<bool>,
        pub isCancelled: Option<bool>,
        pub isNew: Option<bool>
    }

    #[derive(Deserialize, Debug)]
    pub struct Response {
        pub status: u16,
        pub data: Vec<Datum>
    }
}