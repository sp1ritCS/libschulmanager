mod sm_req;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, COOKIE};
use std::collections::BTreeMap;

pub struct SmUser {
    pub session: String,
    pub session_sig: String,
    pub student_id: usize,
    pub student_class_id: usize
}

enum SmLessonStatus {
    Lesson,
    Substitution(SmSubstitutionLesson),
    Cancelled
}

struct SmSubject {
    abbreviation: String,
    name: String
}

struct SmTeacher {
    abbreviation: String,
    firstname: Option<String>,
    lastname: Option<String>
}

struct SmLesson {
    status: SmLessonStatus,
    room: String,
    subject: SmSubject,
    teachers: Vec<SmTeacher>,
    classes: Vec<String>,
    student_group: Vec<String>,
    comment: Option<String>,
    subject_label: String,
}
struct SmSubstitutionLesson {
    room: String,
    subject: SmSubject,
    teachers: Vec<SmTeacher>,
    classes: Vec<String>,
    student_group: Vec<String>,
    comment: Option<String>,
    subject_label: String,
}

/* smart */
pub struct SmWeek {
    monday: BTreeMap<usize, Vec<SmLesson>>,
    tuesday: BTreeMap<usize, Vec<SmLesson>>,
    wednesday: BTreeMap<usize, Vec<SmLesson>>,
    thursday: BTreeMap<usize, Vec<SmLesson>>,
    friday: BTreeMap<usize, Vec<SmLesson>>
}

pub struct SmTimetable {
    interna_timetable: Vec<sm_req::SmTimetableResp>
}

impl SmTimetable {
    pub async fn new(user: SmUser) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json;charset=UTF-8"));
        let cookies = format!("session={}; session.sig={}", user.session, user.session_sig);
        headers.insert(COOKIE, HeaderValue::from_str(&cookies).unwrap());
        let client = reqwest::Client::new();
        let resp: Vec<sm_req::SmTimetableResp> = client.post("https://login.schulmanager-online.de/api/calls")
            .headers(headers)
            .body(format!("[{{\"moduleName\":\"schedules\",\"endpointName\":\"get-actual-lessons\",\"parameters\":{{\"student\":{{\"id\":{}}},\"start\":\"2020-09-28\",\"end\":\"2020-10-04\"}}}}]", user.student_id))
            .send()
            .await?
            .json()
            .await?;
        println!("{:#?}", resp);
        Ok(SmTimetable{
            interna_timetable: resp
        })
    }
    pub fn is_success(self) -> bool {
        let mut success: bool = true;
        for table in self.interna_timetable {
            if table.status < 200 || table.status >= 300 {
                success = false;
            }
        }
        success
    }
    pub fn to_smart(self) -> SmWeek {

    }
}

#[cfg(test)]
mod tests {
    use crate::{SmTimetable, SmUser};
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }
    #[test]
    fn it_works() {
        let user = SmUser {
            session: String::from(std::env::var("SM_TEST_SESSION").expect("SM_TEST_SESSION is not defined")),
            session_sig: String::from(std::env::var("SM_TEST_SESSION_SIG").expect("SM_TEST_SESSION_SIG is not defined")),
            student_id: std::env::var("SM_TEST_ID").expect("SM_TEST_ID is not defined").parse().expect("Invalid Student ID"),
            student_class_id: std::env::var("SM_TEST_CLASSID").expect("SM_TEST_CLASSID").parse().expect("Invalid Class ID")
        };
        assert!(aw!(SmTimetable::new(user)).unwrap().is_success())
    }
}
