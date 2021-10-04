pub use super::{ModRequest, ParseRes, RequestManager};

pub mod request {
	use super::{ModRequest, ParseRes, RequestManager};
	use anyhow::{Result as ERes, Context};
    use serde::Serialize;
    use serde_json::Value;
    use chrono::{Date, Weekday, Datelike, Local, TimeZone};
    /*  Thanks to harmic for his brilliant stackoverflow answer
    https://stackoverflow.com/questions/64174950/get-date-of-start-end-of-week */
    fn week_bounds(week: u32, year: i32) -> (Date<Local>, Date<Local>) {
        let mon: Date<Local> = Local.isoywd(year, week, Weekday::Mon);
        let sun: Date<Local> = Local.isoywd(year, week, Weekday::Sun);
        (mon, sun)
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct TimetableBodyParamsStudent {
        pub id: usize,
        pub class_id: usize
    }

    #[derive(Serialize, Debug)]
    pub struct TimetableBodyParams {
        pub student: TimetableBodyParamsStudent,
        pub start: String,
        pub end: String
    }
    impl TimetableBodyParams {
    	pub fn new(id: usize, class_id: usize, week: u32, oyear: Option<i32>) -> Self {
    		let year = oyear.unwrap_or(Local::now().year());
            let (mon, sun) = week_bounds(week, year);
    		Self {
    			student: TimetableBodyParamsStudent {
                    id: id,
                    class_id: class_id
                },
                start: mon.format("%F").to_string(),
                end: sun.format("%F").to_string()
    		}
    	}
    }

    #[derive(Debug)]
    pub struct Timetable {
    	params: TimetableBodyParams,
    	result: Option<super::response::Result>
    }
    impl Timetable {
    	pub fn new(id: usize, class_id: usize, week: u32, oyear: Option<i32>) -> Self {
    		Self {
    			params: TimetableBodyParams::new(id, class_id, week, oyear),
    			result: None
    		}
    	}
    	pub fn get(&mut self) -> ERes<super::response::Result> {
    		self.result.take().ok_or(crate::errors::SmError::UninitializedData.into())
    	}
    }
    impl ModRequest for Timetable {
    	fn get_module_name(&self) -> &'static str {"schedules"}
    	fn get_endpoint_name(&self) -> &'static str {"get-actual-lessons"}
    	fn get_value(&self) -> ParseRes<Value> {
    		serde_json::to_value(&self.params)
    	}
    	fn set_value(&mut self, value: Value) -> ERes<()> {
    		self.result = Some(serde_json::from_value(value).context("failed parsing timetable")?);
    		Ok(())
    	}
    }
    impl <'l> RequestManager<'l> {
    	pub fn add_timetable(&mut self, tt: &'l mut Timetable) -> ERes<()> {
			self._state.push(tt);
			Ok(())
		}
    }
}

pub mod response {
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
    #[serde(rename_all = "camelCase")]
    pub struct StudentGroup {
        pub id: usize,
        pub name: String,
        pub class_id: Option<usize>
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Event {
        pub text: String,
        pub teachers: Vec<Teacher>,
        pub classes: Vec<Class>,
        pub student_groups: Vec<StudentGroup>,
        pub absence_id: usize
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct ActualLesson {
        pub room: Room,
        pub subject: Subject,
        pub teachers: Vec<Teacher>,
        pub classes: Vec<Class>,
        pub student_groups: Vec<StudentGroup>,
        pub comment: Option<String>,
        pub subject_label: String,
        pub lesson_id: Option<usize>,
        pub substitution_id: Option<usize>
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct OriginalLesson {
        pub room: Room,
        pub subject: Subject,
        pub teachers: Vec<Teacher>,
        pub classes: Vec<Class>,
        pub student_groups: Vec<StudentGroup>,
        pub comment: Option<String>,
        pub subject_label: String,
        pub lesson_id: usize
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Datum {
        pub date: String,
        pub class_hour: ClassHour,
        pub actual_lesson: Option<ActualLesson>,
        pub comment: Option<String>,
        pub original_lessons: Option<Vec<OriginalLesson>>,
        pub event: Option<Event>,
        pub is_substitution: Option<bool>,
        pub is_cancelled: Option<bool>,
        pub is_new: Option<bool>
    }

    pub type Result = Vec<Datum>;
}
