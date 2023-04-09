use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// src/sessions.rs
pub use crate::sessions::Session;

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub id: i64,
    pub url: String,
    pub name: Option<String>,
    pub following: i64,
    pub followers: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GoodUser {
    pub id: i64,
    pub github: i64,
    pub personal_token: Option<String>,
    pub reports: i64,
    pub created: NaiveDateTime,
}
