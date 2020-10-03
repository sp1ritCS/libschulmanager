mod sm_req;
pub mod timetable;
mod errors;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, COOKIE};

pub struct SmUser {
    pub session: String,
    pub session_sig: String,
    pub student_id: usize,
    pub student_class_id: usize
}

pub struct SmTimetable {
    interna_timetable: Vec<sm_req::SmTimetableResp>
}

impl SmTimetable {
    pub async fn new(user: SmUser, week: u32) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json;charset=UTF-8"));
        let cookies = format!("session={}; session.sig={}", user.session, user.session_sig);
        headers.insert(COOKIE, HeaderValue::from_str(&cookies).unwrap());
        let body = vec![sm_req::TimetableBody::new(user.student_id, user.student_class_id, week)];
        let client = reqwest::Client::new();
        let resp: Vec<sm_req::SmTimetableResp> = client.post("https://login.schulmanager-online.de/api/calls")
            .headers(headers)
            .body(serde_json::to_string(&body)?)
            .send()
            .await?
            .json()
            .await?;
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
    pub fn to_smart(self) -> Result<Vec<timetable::SmWeek>, Box<dyn std::error::Error>> {
        let mut timetables: Vec<timetable::SmWeek> = vec![];
        for timetable in self.interna_timetable {
            timetables.push(timetable::SmWeek::from_interna(timetable)?);
        }
        Ok(timetables)
    }
}

#[cfg(test)]
mod tests {
    use crate::{SmTimetable, SmUser};
    use chrono::{Local, Datelike, IsoWeek};
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
        let this_week: IsoWeek = Local::today().iso_week();
        let timetable: SmTimetable = aw!(SmTimetable::new(user, this_week.week())).unwrap();
        assert!(timetable.is_success());
    }
}
