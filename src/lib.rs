pub mod sm;
#[cfg(feature = "microsoft")]
pub mod o365;
pub mod transformers;
pub mod errors;
use sm::{RequestManager, ResultBody};
use sm::{Timetable, TimetableResult};
use sm::{Hours, HoursResult};
use isahc::{prelude::*, HttpClient, cookies::CookieJar, Request};
use http::{header::{self, HeaderMap, HeaderValue}, method::Method};
use anyhow::Result;

fn set_json(headers: &mut HeaderMap) {
	headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json;charset=UTF-8"));
}
fn set_jwt(headers: &mut HeaderMap, jwt: &str) -> std::result::Result<(), header::InvalidHeaderValue> {
	headers.insert(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", jwt))?);
	Ok(())
}

pub struct SmOfficeUser {
    pub email: String,
    pub password: String
}

#[derive(Debug)]
pub enum ClientAuthMethod<'c> {
	CookieAuth(&'c CookieJar),
	JwtAuth(String)
}

#[derive(Debug)]
pub struct Schulmanager {
    pub client: HttpClient,
    pub token: String,
    pub student_id: usize,
    pub student_class_id: usize
}
impl <'l> Schulmanager {
	pub async fn new(auth: ClientAuthMethod<'_>) -> Result<Self> {
		let (client, auth) = match auth {
			ClientAuthMethod::CookieAuth(jar) => {
				(HttpClient::builder()
					.cookies()
					.cookie_jar(jar.clone())
					.build()?
				, None)
			},
			ClientAuthMethod::JwtAuth(jwt) => (HttpClient::new()?, Some(jwt))
		};

		let mut get_jwt_request = Request::builder()
			.method(Method::GET)
			.uri("https://login.schulmanager-online.de/oidc/get-jwt")
			.body(())?;
		set_json(get_jwt_request.headers_mut());
		if let Some(auth) = auth {
			set_jwt(get_jwt_request.headers_mut(), &auth)?;
		}

		let get_jwt = client.send_async(get_jwt_request).await?;
		let jwt = match get_jwt.headers().get("x-new-bearer-token") {
            Some(token) => Ok(token.clone()),
            None => Err(Box::new(crate::errors::SmError::NoJwt))
        }?;

        let mut get_user_request = Request::builder()
			.method(Method::POST)
			.uri("https://login.schulmanager-online.de/api/login-status")
			.body(())?;
		set_json(get_user_request.headers_mut());
		set_jwt(get_user_request.headers_mut(), jwt.to_str()?)?;

		let get_user: sm::LoginStatus = client.send_async(get_user_request).await?
			.json().await?;
		if !get_user.is_authenticated {
            Err(Box::new(errors::SmError::Unauthenticated).into())
        }else{
            match get_user.user {
                Some(user) => {
                	Ok(Schulmanager {
                		client,
                		token: jwt.to_str()?.to_owned(),
                		student_id: user.associated_student.id,
                		student_class_id: user.associated_student.class_id
                	})
                },
                None => Err(Box::new(errors::SmError::UnknownAuth).into())
            }
        }
	}

	#[cfg(feature = "microsoft")]
    pub async fn login_office(user: SmOfficeUser) -> Result<Self> {
        let o365 = o365::O365Auth::new(String::from("https://login.schulmanager-online.de/oidc/413"), String::from("82a6d564-b994-4598-aff6-e131f8cfb1ae")).await?;
        o365.login(user.email, user.password).await?;
        Self::new(ClientAuthMethod::CookieAuth(o365.req_client.cookie_jar().unwrap())).await
    }

    #[deprecated(since = "0.2", note = "consider using Schulmanager::new directly")]
    pub async fn use_jwt(token: String) -> Result<Self> {
        Self::new(ClientAuthMethod::JwtAuth(token)).await
    }

    pub async fn make_request(&'l self, mgr: &'l mut RequestManager<'l>) -> Result<()> {
    	let body: String = {
    		let body = mgr.get_request()?;
    		serde_json::to_string(&body)?
    	};

    	let mut request = Request::builder()
			.method(Method::POST)
			.uri("https://login.schulmanager-online.de/api/calls")
			.body(body)?;
		set_json(request.headers_mut());
		set_jwt(request.headers_mut(), &self.token)?;

		let resp: ResultBody = self.client.send_async(request).await?
			.json().await?;

		mgr.get_results(resp)
    }

	pub async fn get_timetable(&self, week: u32, year: Option<i32>) -> Result<SmTimetable> {
		let mut mgr = RequestManager::new();

		let mut params = Timetable::new(self.student_id, self.student_class_id, week, year);
		mgr.add_timetable(&mut params).expect("TODO");

		self.make_request(&mut mgr).await?;
		Ok(SmTimetable {
			interna_timetable: params.get()?
		})
	}

	pub async fn get_hours(&self) -> Result<SmHours> {
		let mut mgr = RequestManager::new();

		let mut params = Hours::new();
		mgr.add_hours(&mut params).expect("TODO");

		self.make_request(&mut mgr).await?;

		Ok(SmHours(params.get()?))
	}
}

pub struct SmTimetable {
	interna_timetable: TimetableResult
}

impl SmTimetable {
    pub fn from_reader(reader: Box<dyn std::io::BufRead>) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let result: ResultBody = serde_json::from_reader(reader)?;
        let mut mgr = RequestManager::new();

		let mut params = Timetable::new(0, 0, 1, None);
		mgr.add_timetable(&mut params)?;

		mgr.get_results(result)?;

		Ok(SmTimetable {
			interna_timetable: params.get()?
		})
    }
}

pub struct SmHours(HoursResult);

use std::collections::BTreeMap;
use chrono::NaiveTime;

pub type SchoolHours = Vec<(NaiveTime, NaiveTime)>;
pub type SchoolHoursMap = BTreeMap<usize, SchoolHours>;

impl SmHours {
	pub fn from_reader(reader: Box<dyn std::io::BufRead>) -> Result<Self> {
		let result: ResultBody = serde_json::from_reader(reader)?;
        let mut mgr = RequestManager::new();

        let mut params = Hours::new();
		mgr.add_hours(&mut params)?;

		mgr.get_results(result)?;
		Ok(SmHours(params.get()?))
    }
    pub fn is_success(&self) -> bool {
        let status = self.0.status;
        status >= 200 && status < 300
    }

    pub fn parse(&self) -> Result<SchoolHoursMap> {
		self.0.data.iter().map(|e| -> Result<(usize, SchoolHours)> {
			let mut times = Vec::new();
			for (i, start) in e.from_by_day.iter().enumerate() {
				times.push((NaiveTime::parse_from_str(start, "%H:%M:%S")?, NaiveTime::parse_from_str(&e.until_by_day[i], "%H:%M:%S")?))
			}
			Ok((e.number, times))
		}).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, Datelike, IsoWeek};
    use futures_test as fut;

    #[test]
    fn timetable_parser_test() -> Result<(), Box<dyn std::error::Error>> {
        const PATH: &'static str = "src/test_table.json";
        let file = std::fs::File::open(PATH)?;
        let reader = std::io::BufReader::new(file);
        let _timetable: SmTimetable = SmTimetable::from_reader(Box::new(reader))?;
        Ok(())
    }

    #[test]
    fn timetable_parser_and_smart_test() -> Result<(), Box<dyn std::error::Error>> {
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
    fn hours_parser_test() -> Result<(), Box<dyn std::error::Error>> {
        const PATH: &'static str = "src/test_hours.json";
        let file = std::fs::File::open(PATH)?;
        let reader = std::io::BufReader::new(file);
        let hours: SmHours = SmHours::from_reader(Box::new(reader))?;
        let _parsed = hours.parse()?;
        println!("{:#?}", _parsed);
        Ok(())
    }

	#[cfg(feature = "microsoft")]
    #[fut::test]
    #[ignore]
    async fn realworld_o365_auth() {
        let user = SmOfficeUser {
            email: String::from(std::env::var("SM_TEST_OFFICE_EMAIL").expect("SM_TEST_OFFICE_EMAIL is not defined")),
            password: String::from(std::env::var("SM_TEST_OFFICE_PASSWORD").expect("SM_TEST_OFFICE_PASSWORD is not defined"))
        };
        let schulmanager: Schulmanager = Schulmanager::login_office(user).await.unwrap();
        let this_week: IsoWeek = Local::today().iso_week();
        let _timetable: SmTimetable = schulmanager.get_timetable(this_week.week(), None).await.unwrap();
        let hours: SmHours = schulmanager.get_hours().await.unwrap();
        assert!(hours.is_success());
    }
    
    #[fut::test]
    #[ignore]
    async fn realworld_jwt_auth() {
        let token = String::from(std::env::var("SM_TEST_JWT").expect("SM_TEST_JWT is not defined"));
        let schulmanager: Schulmanager = Schulmanager::new(ClientAuthMethod::JwtAuth(token)).await.unwrap();
        let this_week: IsoWeek = Local::today().iso_week();
        let _timetable: SmTimetable = schulmanager.get_timetable(this_week.week(), None).await.unwrap();
        let hours: SmHours = schulmanager.get_hours().await.unwrap();
        assert!(hours.is_success());
    }
}
