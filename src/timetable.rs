use std::collections::BTreeMap;
use crate::sm_req::{SmTimetableResp, ActualLesson, OriginalLesson, Subject, Teacher, Class, StudentGroup, Event};
use chrono::{NaiveDate, Datelike, Weekday};
use std::clone::Clone;
use serde::Serialize;

fn string_vec_calc(classes_s: Vec<Class>, groups: Vec<StudentGroup>) -> (Vec<String>, Vec<String>) {
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
enum SmLessonStatus {
    Lesson,
    Substitution(SmSubstitutedLesson),
    Event(SmEvent),
    Cancelled
}

#[derive(Serialize, Clone, Debug)]
struct SmSubject {
    abbreviation: String,
    name: String
}
impl SmSubject {
    pub fn new(subj: Subject) -> Self {
        SmSubject {
            abbreviation: subj.abbreviation,
            name: subj.name
        }
    }
}

#[derive(Serialize, Clone, Debug)]
struct SmTeacher {
    abbreviation: String,
    firstname: Option<String>,
    lastname: Option<String>
}
impl SmTeacher {
    pub fn new(teacher: Teacher) -> Self {
        SmTeacher {
            abbreviation: teacher.abbreviation,
            firstname: teacher.firstname,
            lastname: teacher.lastname
        }
    }
    pub fn new_vec(teachers: Vec<Teacher>) -> Vec<Self> {
        let mut sm_teachers: Vec<SmTeacher> = vec![];
        for teacher in teachers {
            sm_teachers.push(SmTeacher::new(teacher));
        }
        sm_teachers
    }
}
#[derive(Serialize, Debug)]
struct SmLesson {
    status: SmLessonStatus,
    room: String,
    subject: SmSubject,
    teachers: Vec<SmTeacher>,
    classes: Vec<String>,
    student_groups: Vec<String>,
    comment: Option<String>,
    subject_label: String,
}
impl SmLesson {
    pub fn from_actual(lesson: ActualLesson, status: SmLessonStatus, comment: Option<String>) -> Self {
        let (classes, student_groups) = string_vec_calc(lesson.classes, lesson.studentGroups);
        SmLesson {
            status: status,
            room: lesson.room.name,
            subject: SmSubject::new(lesson.subject),
            teachers: SmTeacher::new_vec(lesson.teachers),
            classes: classes,
            student_groups: student_groups,
            comment: comment,
            subject_label: lesson.subjectLabel
        }
    }
    pub fn from_orig(lesson: OriginalLesson, status: SmLessonStatus, comment: Option<String>) -> Self {
        let (classes, student_groups) = string_vec_calc(lesson.classes, lesson.studentGroups);
        SmLesson {
            status: status,
            room: lesson.room.name,
            subject: SmSubject::new(lesson.subject),
            teachers: SmTeacher::new_vec(lesson.teachers),
            classes: classes,
            student_groups: student_groups,
            comment: comment,
            subject_label: lesson.subjectLabel
        }
    }
    pub fn from_orig_vec(lessons: &Vec<OriginalLesson>, status: SmLessonStatus, comment: Option<String>) -> Option<Self> {
        let mut orig: Option<Self> = None;
        for lesson in lessons {
            /*let (classes, student_groups) = string_vec_calc(lesson.classes.clone(), lesson.studentGroups.clone());
            orig = Some(SmLesson {
                status: status.clone(),
                room: lesson.room.name.clone(),
                subject: SmSubject::new(lesson.subject.clone()),
                teachers: SmTeacher::new_vec(lesson.teachers.clone()),
                classes: classes.clone(),
                student_groups: student_groups.clone(),
                comment: comment.clone(),
                subject_label: lesson.subjectLabel.clone()
            })*/
            orig = Some(SmLesson::from_orig(lesson.clone(), status.clone(), comment.clone()))
        }
        orig
    }
}
#[derive(Serialize, Clone, Debug)]
struct SmSubstitutedLesson {
    room: String,
    subject: SmSubject,
    teachers: Vec<SmTeacher>,
    classes: Vec<String>,
    student_groups: Vec<String>,
    comment: Option<String>,
    subject_label: String,
}
impl SmSubstitutedLesson {
    pub fn from_orig(lesson: OriginalLesson, comment: Option<String>) -> Self {
        let (classes, student_groups) = string_vec_calc(lesson.classes, lesson.studentGroups);
        SmSubstitutedLesson {
            room: lesson.room.name,
            subject: SmSubject::new(lesson.subject),
            teachers: SmTeacher::new_vec(lesson.teachers),
            classes: classes,
            student_groups: student_groups,
            comment: comment,
            subject_label: lesson.subjectLabel
        }
    }
    pub fn from_orig_vec(lessons: &Vec<OriginalLesson>, comment: Option<String>) -> Option<Self> {
        let mut orig: Option<Self> = None;
        for lesson in lessons {
            orig = Some(SmSubstitutedLesson::from_orig(lesson.clone(), comment.clone()))
        }
        orig
    }
}

#[derive(Serialize, Clone, Debug)]
struct SmEvent {
    text: String,
    teachers: Vec<SmTeacher>,
    classes: Vec<String>,
    student_groups: Vec<String>
}
impl SmEvent {
    pub fn from_orig(event: Event) -> Self {
        let (classes, student_groups) = string_vec_calc(event.classes, event.studentGroups);
        SmEvent {
            text: event.text,
            teachers: SmTeacher::new_vec(event.teachers),
            classes: classes,
            student_groups: student_groups
        }
    }
}

/* smart */
#[derive(Serialize, Debug)]
pub struct SmWeek {
    monday: BTreeMap<usize, SmLesson>,
    tuesday: BTreeMap<usize, SmLesson>,
    wednesday: BTreeMap<usize, SmLesson>,
    thursday: BTreeMap<usize, SmLesson>,
    friday: BTreeMap<usize, SmLesson>
}

#[macro_use]
macro_rules! skip_none {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => {
                eprintln!("Nothing was defined");
                continue;
            }
        }
    };
}

impl SmWeek {
    pub fn from_interna(interna_timetable: SmTimetableResp) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let mut week = Self {
            monday: BTreeMap::new(),
            tuesday: BTreeMap::new(),
            wednesday: BTreeMap::new(),
            thursday: BTreeMap::new(),
            friday: BTreeMap::new()
        };
        for ilesson in interna_timetable.data {
            #[allow(unused_assignments)]
            let mut lesson: Option<SmLesson> = None;
            if ilesson.isSubstitution.is_some() {
                let status = SmLessonStatus::Substitution(skip_none!(SmSubstitutedLesson::from_orig_vec(skip_none!(&ilesson.originalLessons), ilesson.comment.clone())));
                lesson = Some(SmLesson::from_actual(skip_none!(ilesson.actualLesson), status, ilesson.comment));
            } else if ilesson.isCancelled.is_some() {
                lesson = Some(skip_none!(SmLesson::from_orig_vec(skip_none!(&ilesson.originalLessons), SmLessonStatus::Cancelled, ilesson.comment)));
            } else {
                lesson = Some(SmLesson::from_actual(skip_none!(ilesson.actualLesson), SmLessonStatus::Lesson, ilesson.comment));
            }
            let date = NaiveDate::parse_from_str(&ilesson.date, "%F")?;
            match date.weekday() {
                Weekday::Mon => week.monday.insert(ilesson.classHour.number.parse()?, skip_none!(lesson)),
                Weekday::Tue => week.tuesday.insert(ilesson.classHour.number.parse()?, skip_none!(lesson)),
                Weekday::Wed => week.wednesday.insert(ilesson.classHour.number.parse()?, skip_none!(lesson)),
                Weekday::Thu => week.thursday.insert(ilesson.classHour.number.parse()?, skip_none!(lesson)),
                Weekday::Fri => week.friday.insert(ilesson.classHour.number.parse()?, skip_none!(lesson)),
                _ => {
                    eprintln!("The \"smart\" representation does not suport lessons on sat/sun");
                    None
                }
            };
        }
        Ok(week)
    }
}
