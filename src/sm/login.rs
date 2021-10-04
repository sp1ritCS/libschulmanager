use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Student {
    pub id: usize,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    sex: Option<String>,
    pub class_id: usize
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub email: Option<String>,
    pub username: Option<String>,
    pub local_username: Option<String>,
    pub id: usize,
    has_administrator_rights: bool,
    last_seen_notification_timestamp: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub associated_student: Student
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub is_authenticated: bool,
    pub user: Option<User>
}
