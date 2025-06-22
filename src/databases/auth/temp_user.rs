use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SignupData {
    pub name: String,
    pub number: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TempUser {
    pub temp_id: String,
    pub name: String,
    pub number: String,
    pub email: String,
    pub password: String,
    pub code: String,
}
