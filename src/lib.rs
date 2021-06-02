pub mod sm_req;
pub mod o365;
pub mod transformers;
pub mod errors;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, COOKIE};

pub struct SmSession {
    pub session: String,
    pub session_sig: String
}

pub struct SmOfficeUser {
    pub email: String,
    pub password: String
}

pub struct Schulmanager {
    pub client: reqwest::Client,
    pub token: String,
    pub student_id: usize,
    pub student_class_id: usize
}
impl Schulmanager {
    async fn get_user(client: &reqwest::Client, mut headers: HeaderMap) -> std::result::Result<(String, usize, usize), Box<dyn std::error::Error>> {
        let jwt_request = client.get("https://login.schulmanager-online.de/oidc/get-jwt")
            .headers(headers.clone())
            .send()
            .await?;
        
        let jwt = match jwt_request.headers().get("x-new-bearer-token") {
            Some(token) => Ok(token.clone()),
            None => Err(Box::new(crate::errors::SmError::NoJwt))
        }?;
        
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json;charset=UTF-8"));
        headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", jwt.to_str()?))?);
        let resp: sm_req::SmLoginStatus::Status = client.post("https://login.schulmanager-online.de/api/login-status")
            .headers(headers)
            .send()
            .await?
            .json()
            .await?;
        if !resp.isAuthenticated {
            Err(Box::new(errors::SmError::Unauthenticated))
        }else{
            match resp.user {
                Some(user) => Ok((jwt.to_str()?.to_owned(), user.associatedStudent.id, user.associatedStudent.classId)),
                None => Err(Box::new(errors::SmError::UnknownAuth))
            }
        }

    }
    pub async fn login_office(user: SmOfficeUser) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let o365 = o365::O365Auth::new(String::from("https://login.schulmanager-online.de/oidc/413"), String::from("82a6d564-b994-4598-aff6-e131f8cfb1ae")).await?;
        o365.login(user.email, user.password).await?;
        let (token, student_id, student_class_id) = Schulmanager::get_user(&o365.req_client, HeaderMap::new()).await?;
        Ok(Schulmanager {
            client: o365.req_client,
            token,
            student_id,
            student_class_id
        })
    }
    #[deprecated(since="0.1.1", note="please use `use_jwt` instead")]
    pub async fn use_session(session: SmSession) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        let cookies = format!("session={}; session.sig={}", session.session, session.session_sig);
        headers.insert(COOKIE, HeaderValue::from_str(&cookies).unwrap());
        let client = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .build()?;
        let (token, student_id, class_id) = Self::get_user(&client, headers).await?;
        Ok(Schulmanager {
            client,
            token,
            student_id: student_id,
            student_class_id: class_id
        })
    }
    pub async fn use_jwt(token: String) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", token))?);
        let client = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .build()?;
        let (token, student_id, class_id) = Self::get_user(&client, headers).await?;
        Ok(Schulmanager {
            client,
            token,
            student_id: student_id,
            student_class_id: class_id
        })
    }
}

pub struct SmTimetable {
    interna_timetable: Vec<sm_req::SmTimetableResponse::Response>
}

impl SmTimetable {
    pub async fn new(sm: Schulmanager, week: u32, year: Option<i32>) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json;charset=UTF-8"));
        headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", &sm.token))?);
        let body = vec![sm_req::SmTimetableRequest::Body::new(sm.student_id, sm.student_class_id, week, year)];
        let resp: Vec<sm_req::SmTimetableResponse::Response> = sm.client.post("https://login.schulmanager-online.de/api/calls")
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
    pub fn from_reader(reader: Box<dyn std::io::BufRead>) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(SmTimetable{
            interna_timetable: serde_json::from_reader(reader)?
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
    /*pub fn to_smart(self) -> Result<Vec<timetable::SmWeek>, Box<dyn std::error::Error>> {
        let mut timetables: Vec<timetable::SmWeek> = vec![];
        for timetable in self.interna_timetable {
            timetables.push(timetable::SmWeek::from_interna(timetable)?);
        }
        Ok(timetables)
    }*/
}

#[cfg(test)]
mod tests {
    use crate::{SmTimetable, SmSession, SmOfficeUser, Schulmanager};
    use chrono::{Local, Datelike, IsoWeek};
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }
    #[test]
    fn parser_test() -> Result<(), Box<dyn std::error::Error>> {
        const PATH: &'static str = "src/test_table.json";
        let file = std::fs::File::open(PATH)?;
        let reader = std::io::BufReader::new(file);
        let timetable: SmTimetable = SmTimetable::from_reader(Box::new(reader))?;
        assert!(timetable.is_success());
        Ok(())
    }

    #[test]
    fn parser_and_smart_test() -> Result<(), Box<dyn std::error::Error>> {
        const PATH: &'static str = "src/test_table.json";
        let file = std::fs::File::open(PATH)?;
        let reader = std::io::BufReader::new(file);
        let timetable: SmTimetable = SmTimetable::from_reader(Box::new(reader))?;
        let _smart = timetable.to_smart_v2_daymap()?;
        //let _smart = timetable.to_smart_v2_weekdays()?;
        println!("{:#?}", _smart);
        Ok(())
    }

    #[test]
    #[ignore]
    fn realworld_o365_auth() {
        let user = SmOfficeUser {
            email: String::from(std::env::var("SM_TEST_OFFICE_EMAIL").expect("SM_TEST_OFFICE_EMAIL is not defined")),
            password: String::from(std::env::var("SM_TEST_OFFICE_PASSWORD").expect("SM_TEST_OFFICE_PASSWORD is not defined"))
        };
        let schulmanager: Schulmanager = aw!(Schulmanager::login_office(user)).unwrap();
        let this_week: IsoWeek = Local::today().iso_week();
        let timetable: SmTimetable = aw!(SmTimetable::new(schulmanager, this_week.week(), None)).unwrap();
        assert!(timetable.is_success());
    }

    #[test]
    #[ignore]
    fn realworld_session_auth() {
        let user = SmSession {
            session: String::from(std::env::var("SM_TEST_SESSION").expect("SM_TEST_SESSION is not defined")),
            session_sig: String::from(std::env::var("SM_TEST_SESSION_SIG").expect("SM_TEST_SESSION_SIG is not defined"))
        };
        let schulmanager: Schulmanager = aw!(Schulmanager::use_session(user)).unwrap();
        let this_week: IsoWeek = Local::today().iso_week();
        let timetable: SmTimetable = aw!(SmTimetable::new(schulmanager, this_week.week(), None)).unwrap();
        assert!(timetable.is_success());
    }
    
    #[test]
    #[ignore]
    fn realworld_jwt_auth() {
        let token = String::from(std::env::var("SM_TEST_JWT").expect("SM_TEST_JWT is not defined"));
        let schulmanager: Schulmanager = aw!(Schulmanager::use_jwt(token)).unwrap();
        let this_week: IsoWeek = Local::today().iso_week();
        let timetable: SmTimetable = aw!(SmTimetable::new(schulmanager, this_week.week(), None)).unwrap();
        assert!(timetable.is_success());
    }
}
