mod login;
pub use login::Status as LoginStatus;

pub mod timetable;
pub use timetable::{request::Timetable, response::Result as TimetableResult};
pub mod hours;
pub use hours::{request::Hours, response::Result as HoursResult};

use crate::errors::SmError;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use anyhow::Result as ERes;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request<'s> {
	pub module_name: &'s str,
	pub endpoint_name: &'s str,
	pub parameters: Value
}
impl <'s> Request<'s> {
	fn new(request: &dyn ModRequest) -> ERes<Self> {
		Ok(Self {
			module_name: request.get_module_name(),
			endpoint_name: request.get_endpoint_name(),
			parameters: request.get_value()?
		})
	}
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody<'s> {
	pub bundle_version: &'s str,
	pub requests: Vec<Request<'s>>
}
impl <'s> RequestBody<'s> {
	fn new(requests: &Vec<&'s mut dyn ModRequest>) -> ERes<Self> {
		Ok(Self {
			bundle_version: "fee1dead",
			requests: requests.iter().map(|req| Request::new(*req)).collect::<ERes<Vec<Request<'s>>>>()?
		})
	}
}

#[derive(Deserialize, Debug)]
pub struct Result {
	pub status: u16,
	pub data: Value
}

#[derive(Deserialize, Debug)]
pub struct ResultBody {
	pub results: Vec<Result>,
	#[serde(default)]
	pub system_status_messages: Vec<Value>
}

pub type ParseRes<T> = std::result::Result<T, serde_json::Error>;
pub trait ModRequest {
	fn get_module_name(&self) -> &'static str;
	fn get_endpoint_name(&self) -> &'static str;

	fn get_value(&self) -> ParseRes<Value>;
	fn set_value(&mut self, value: Value) -> ERes<()>;
}

pub struct RequestManager<'l> {
	pub _state: Vec<&'l mut dyn ModRequest>
}
impl <'l, 'r> RequestManager<'l> {
	pub fn new() -> Self {
		Self {
			_state: Vec::new()
		}
	}

	pub fn get_request(&'r self) -> ERes<RequestBody> {
		RequestBody::new(&self._state)
	}

	pub fn get_results(&'l mut self, mut result: ResultBody) -> ERes<()> {
		self._state.iter_mut().enumerate().map(|(i, request)| -> ERes<()> {
			let status = result.results[i].status;
			if status < 200 || status >= 300 {
				return Err(SmError::NonvalidStatusCode { statuscode: status }.into())
			}
			request.set_value(serde_json::from_value(result.results.remove(i).data)?)
		}).collect()
	}
}
