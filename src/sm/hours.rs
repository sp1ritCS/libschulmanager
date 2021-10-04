use super::{ModRequest, ParseRes, RequestManager};

pub mod request {
	use super::{ModRequest, ParseRes, RequestManager};
	use anyhow::{Result as ERes, Context};
    use serde::Serialize;
    use serde_json::Value;

    #[derive(Serialize, Debug)]
    pub struct SmHoursRequestActionParams<'s> {
    	pub attributes: Vec<&'s str>
    }

    #[derive(Serialize, Debug)]
    pub struct SmHoursRequestAction<'s> {
    	pub model: &'s str,
    	pub action: &'s str,
    	pub parameters: Vec<SmHoursRequestActionParams<'s>>
    }

    #[derive(Serialize, Debug)]
    pub struct SmHoursRequestParams<'s> {
    	pub action: SmHoursRequestAction<'s>
    }
    impl SmHoursRequestParams<'_> {
    	pub fn new() -> Self {
    		Self {
    			action: SmHoursRequestAction {
					model: "main/class-hour",
					action: "findAll",
					parameters: vec![SmHoursRequestActionParams {
						attributes: vec![
							"number",
							"from",
							"until",
							"fromByDay",
							"untilByDay"
						]
					}]
				}
    		}
    	}
    }

    pub struct Hours<'l> {
    	params: SmHoursRequestParams<'l>,
    	result: Option<super::response::Result>
    }
    impl Hours<'_> {
    	pub fn new() -> Self {
    		Self {
    			params: SmHoursRequestParams::new(),
    			result: None
    		}
    	}
    	pub fn get(&mut self) -> ERes<super::response::Result> {
    		self.result.take().ok_or(crate::errors::SmError::UninitializedData.into())
    	}
    }
    impl ModRequest for Hours<'_> {
    	fn get_module_name(&self) -> &'static str {"schedules"}
    	fn get_endpoint_name(&self) -> &'static str {"poqa"}
    	fn get_value(&self) -> ParseRes<serde_json::Value> {
    		serde_json::to_value(&self.params)
    	}
    	fn set_value(&mut self, value: Value) -> ERes<()> {
    		self.result = Some(serde_json::from_value(value).context("failed parsing hours")?);
    		Ok(())
    	}
    }
    impl <'l> RequestManager<'l> {
    	pub fn add_hours(&mut self, hrs: &'l mut Hours) -> ERes<()> {
			self._state.push(hrs);
			Ok(())
		}
    }
}

pub mod response {
	use serde::{Deserialize, de::{self, Deserializer}};
	fn from_str<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
		where T: std::str::FromStr,
			T::Err: std::fmt::Display,
			D: Deserializer<'de>
	{
		let s = String::deserialize(deserializer)?;
		T::from_str(&s).map_err(de::Error::custom)
	}

	#[derive(Deserialize, Debug)]
	#[serde(rename_all = "camelCase")]
	pub struct LessonHours {
		#[serde(deserialize_with = "from_str")]
		pub number: usize,
		pub from: String,
		pub until: String,
		pub from_by_day: Vec<String>,
		pub until_by_day: Vec<String>,
		pub id: usize
	}

	#[derive(Deserialize, Debug)]
	pub struct Result {
		pub status: u16,
		pub data: Vec<LessonHours>
	}
}
