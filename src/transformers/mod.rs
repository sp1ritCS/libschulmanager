#[cfg(feature = "smartv1")]
pub mod smartv1;
#[cfg(feature = "smartv2")]
pub mod smartv2;

impl crate::SmTimetable {
	#[cfg(feature = "smartv1")]
    pub fn to_smart_v1(self) -> Result<smartv1::SmWeek, Box<dyn std::error::Error>> {
        smartv1::SmWeek::from_interna(self.interna_timetable)
    }
    #[cfg(feature = "smartv2")]
    pub fn to_smart_v2_weekdays(self) -> Result<smartv2::Weekdays, Box<dyn std::error::Error>> {
        smartv2::Weekdays::from_interna(self.interna_timetable)
    }
    #[cfg(feature = "smartv2")]
    pub fn to_smart_v2_daymap(self) -> Result<smartv2::DayMap, Box<dyn std::error::Error>> {
        smartv2::DayMap::from_interna(self.interna_timetable)
    }
}
