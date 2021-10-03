use crate::sm_req::SmTimetableResponse::{Response, ActualLesson as InternaActualLesson, OriginalLesson as InternaOriginalLesson, Subject as InternaSubject, Teacher as InternaTeacher, Class as InternaClass, StudentGroup as InternaStudentGroup, Event as InternaEvent};
use std::collections::BTreeMap;
use chrono::{Weekday, NaiveDate, Datelike};
use serde::Serialize;

fn string_vec_calc(classes_s: Vec<InternaClass>, groups: Vec<InternaStudentGroup>) -> (Vec<String>, Vec<String>) {
    let mut classes: Vec<String> = vec![];
    let mut student_groups: Vec<String> = vec![];
    for class in classes_s {
        classes.push(class.name);
    }
    for student_group in groups {
        student_groups.push(student_group.name);
    }
    (classes, student_groups)
}

#[derive(Serialize, Clone, Debug)]
pub enum TimetableElement {
    Lesson(Lesson),
    Substitution(Lesson, Lesson),
    Cancelled(Lesson),
    Event(Event)
}

#[derive(Serialize, Clone, Debug)]
pub struct Subject {
    pub abbreviation: String,
    pub name: String
}
impl Subject {
    pub fn new(subj: InternaSubject) -> Self {
        Subject {
            abbreviation: subj.abbreviation,
            name: subj.name
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Teacher {
    pub abbreviation: String,
    pub firstname: Option<String>,
    pub lastname: Option<String>
}
impl Teacher {
    pub fn new(teacher: InternaTeacher) -> Self {
        Teacher {
            abbreviation: teacher.abbreviation,
            firstname: teacher.firstname,
            lastname: teacher.lastname
        }
    }
    pub fn new_vec(iteachers: Vec<InternaTeacher>) -> Vec<Self> {
        let mut teachers: Vec<Teacher> = vec![];
        for teacher in iteachers {
            teachers.push(Teacher::new(teacher));
        }
        teachers
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Lesson {
    pub room: String,
    pub subject: Subject,
    pub teachers: Vec<Teacher>,
    pub classes: Vec<String>,
    pub student_groups: Vec<String>,
    pub comment: Option<String>,
    pub subject_label: String,
}
impl Lesson {
    pub fn from_actual(lesson: InternaActualLesson, comment: Option<String>) -> Self {
        let (classes, student_groups) = string_vec_calc(lesson.classes, lesson.studentGroups);
        Lesson {
            room: lesson.room.name,
            subject: Subject::new(lesson.subject),
            teachers: Teacher::new_vec(lesson.teachers),
            classes,
            student_groups,
            comment,
            subject_label: lesson.subjectLabel
        }
    }
    pub fn from_orig(lesson: InternaOriginalLesson, comment: Option<String>) -> Self {
        let (classes, student_groups) = string_vec_calc(lesson.classes, lesson.studentGroups);
        Lesson {
            room: lesson.room.name,
            subject: Subject::new(lesson.subject),
            teachers: Teacher::new_vec(lesson.teachers),
            classes,
            student_groups,
            comment,
            subject_label: lesson.subjectLabel
        }
    }
    pub fn from_orig_vec(lessons: &Vec<InternaOriginalLesson>, comment: Option<String>) -> Option<Self> {
        let mut orig: Option<Self> = None;
        for lesson in lessons {
            orig = Some(Lesson::from_orig(lesson.clone(), comment.clone()))
        }
        orig
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Event {
    pub text: String,
    pub teachers: Vec<Teacher>,
    pub classes: Vec<String>,
    pub student_groups: Vec<String>
}
impl Event {
    pub fn new(event: InternaEvent) -> Self {
        let (classes, student_groups) = string_vec_calc(event.classes, event.studentGroups);
        Event {
            text: event.text,
            teachers: Teacher::new_vec(event.teachers),
            classes,
            student_groups
        }
    }
}

macro_rules! skip_none {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => {
                //eprintln!("Nothing was defined");
                continue;
            }
        }
    };
}

fn check_treemap(map: &mut BTreeMap<usize, Vec<TimetableElement>>, key: usize, value: TimetableElement) {
    match map.get_mut(&key) {
        Some(val) => val.push(value),
        None => {
            map.insert(key, vec![value]);
        }
    };
}

#[derive(Serialize, Clone, Debug)]
pub struct DayMap {
    pub map: BTreeMap<NaiveDate, BTreeMap<usize, Vec<TimetableElement>>>
}
impl DayMap {
    pub fn from_interna(interna_timetable: Response) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let mut map = BTreeMap::new();
        for ilesson in interna_timetable.data {
            #[allow(unused_assignments)]
            let mut tte: Option<TimetableElement> = None;
            if ilesson.isNew.is_some() {
                tte = Some(TimetableElement::Event(Event::new(skip_none!(ilesson.event))))
            } else if ilesson.isSubstitution.is_some() {
                tte = Some(TimetableElement::Substitution(Lesson::from_actual(skip_none!(ilesson.actualLesson), ilesson.comment.clone()), skip_none!(Lesson::from_orig_vec(skip_none!(&ilesson.originalLessons), ilesson.comment))));
            } else if ilesson.isCancelled.is_some() {
                tte = Some(TimetableElement::Cancelled(skip_none!(Lesson::from_orig_vec(skip_none!(&ilesson.originalLessons), ilesson.comment.clone()))));
            } else {
                tte = Some(TimetableElement::Lesson(Lesson::from_actual(skip_none!(ilesson.actualLesson), ilesson.comment.clone())));
            }
            let date = NaiveDate::parse_from_str(&ilesson.date, "%F")?;
            match map.get_mut(&date) {
                Some(val) => check_treemap(val, ilesson.classHour.number.parse()?, skip_none!(tte)),
                None => {
                    map.insert(date, BTreeMap::new());
                    check_treemap(map.get_mut(&date).unwrap(), ilesson.classHour.number.parse()?, skip_none!(tte))
                }
            }
        }
        Ok(DayMap {
            map
        })
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Weekdays {
    pub monday: BTreeMap<usize, Vec<TimetableElement>>,
    pub tuesday: BTreeMap<usize, Vec<TimetableElement>>,
    pub wednesday: BTreeMap<usize, Vec<TimetableElement>>,
    pub thursday: BTreeMap<usize, Vec<TimetableElement>>,
    pub friday: BTreeMap<usize, Vec<TimetableElement>>
}
impl Weekdays {
    pub fn from_interna(interna_timetable: Response) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let mut week = Self {
            monday: BTreeMap::new(),
            tuesday: BTreeMap::new(),
            wednesday: BTreeMap::new(),
            thursday: BTreeMap::new(),
            friday: BTreeMap::new()
        };
        for ilesson in interna_timetable.data {
            #[allow(unused_assignments)]
            let mut tte: Option<TimetableElement> = None;
            if ilesson.isNew.is_some() {
                tte = Some(TimetableElement::Event(Event::new(skip_none!(ilesson.event))))
            } else if ilesson.isSubstitution.is_some() {
                tte = Some(TimetableElement::Substitution(Lesson::from_actual(skip_none!(ilesson.actualLesson), ilesson.comment.clone()), skip_none!(Lesson::from_orig_vec(skip_none!(&ilesson.originalLessons), ilesson.comment))));
            } else if ilesson.isCancelled.is_some() {
                tte = Some(TimetableElement::Cancelled(skip_none!(Lesson::from_orig_vec(skip_none!(&ilesson.originalLessons), ilesson.comment.clone()))));
            } else {
                tte = Some(TimetableElement::Lesson(Lesson::from_actual(skip_none!(ilesson.actualLesson), ilesson.comment.clone())));
            }
            let date = NaiveDate::parse_from_str(&ilesson.date, "%F")?;
            match date.weekday() {
                Weekday::Mon => check_treemap(&mut week.monday, ilesson.classHour.number.parse()?, skip_none!(tte)),
                Weekday::Tue => check_treemap(&mut week.tuesday, ilesson.classHour.number.parse()?, skip_none!(tte)),
                Weekday::Wed => check_treemap(&mut week.wednesday, ilesson.classHour.number.parse()?, skip_none!(tte)),
                Weekday::Thu => check_treemap(&mut week.thursday, ilesson.classHour.number.parse()?, skip_none!(tte)),
                Weekday::Fri => check_treemap(&mut week.friday, ilesson.classHour.number.parse()?, skip_none!(tte)),
                _ => {
                    eprintln!("The \"smart\" representation does not suport lessons on sat/sun");
                }
            };
        }
        Ok(week)
    }
}