pub mod smartv1;
pub mod smartv2;

impl crate::SmTimetable {
    pub fn to_smart_v1(self) -> Result<Vec<smartv1::SmWeek>, Box<dyn std::error::Error>> {
        let mut timetables: Vec<smartv1::SmWeek> = vec![];
        for timetable in self.interna_timetable {
            timetables.push(smartv1::SmWeek::from_interna(timetable)?);
        }
        Ok(timetables)
    }
    pub fn to_smart_v2_weekdays(self) -> Result<Vec<smartv2::Weekdays>, Box<dyn std::error::Error>> {
        let mut timetables: Vec<smartv2::Weekdays> = vec![];
        for timetable in self.interna_timetable {
            timetables.push(smartv2::Weekdays::from_interna(timetable)?);
        }
        Ok(timetables)
    }
    pub fn to_smart_v2_daymap(self) -> Result<Vec<smartv2::DayMap>, Box<dyn std::error::Error>> {
        let mut timetables: Vec<smartv2::DayMap> = vec![];
        for timetable in self.interna_timetable {
            timetables.push(smartv2::DayMap::from_interna(timetable)?);
        }
        Ok(timetables)
    }
}