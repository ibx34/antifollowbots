use crate::{db::Database, AppState};
use anyhow::{bail, Result};
use chrono::NaiveDateTime;
use redis::{Cmd, ConnectionLike, ErrorKind, RedisError};
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: i64,
    pub key: String,
    pub creator: i64,
    pub created_at: NaiveDateTime,
}

impl Session {
    pub async fn create(app: AppState, user: i64, cache: bool) -> Result<Self> {
        let mut key: [u8; 64] = [0; 64];
        let sr = ring::rand::SystemRandom::new();
        if sr.fill(&mut key).is_err() {
            bail!("Failed to generate session key.");
        }
        match sqlx::query_as::<_, Session>(
            r#"INSERT INTO sessions(creator,key) VALUES($1,$2) RETURNING *"#,
        )
        .bind(&user)
        .bind(&key)
        .fetch_one(&app.database.0)
        .await
        {
            Ok(s) => {
                if cache {
                    let mut redis_client = app.redis.clone().get_connection()?;
                    let stringy_session = serde_json::to_string(&s)?;
                    redis_client.req_command(&Cmd::set_ex(
                        &format!("Dogma-sessions:{}", s.key),
                        stringy_session,
                        3600,
                    ))?;
                }
                Ok(s)
            }
            // TODO: Improve this so errors are accurate
            Err(_err) => bail!("Failed to create session"),
        }
    }

    pub async fn get_session(app: AppState, key: String, cache: bool) -> Result<Self> {
        let mut redis_client = app.redis.clone().get_connection()?;
        match redis_client.req_command(&Cmd::get(&format!("Dogma-sessions:{key}"))) {
            Ok(val) => match val {
                redis::Value::Nil => {
                    match sqlx::query_as::<_, Session>(r#"SELECT * FROM sessions WHERE key = $1"#)
                        .bind(&key)
                        .fetch_one(&app.database.0)
                        .await
                    {
                        Ok(s) => {
                            if cache {
                                let stringy_session = serde_json::to_string(&s)?;
                                redis_client.req_command(&Cmd::set_ex(
                                    &format!("Dogma-sessions:{key}"),
                                    stringy_session,
                                    3600,
                                ))?;
                            }
                            Ok(s)
                        }
                        Err(sqlx::Error::RowNotFound) => bail!("Session doesn't exist."),
                        // TODO: Improve this so errors are accurate
                        Err(_err) => bail!("Failed to get session from database"),
                    }
                }
                redis::Value::Data(data) => Ok(serde_json::from_slice::<Session>(&data)?),
                _ => bail!("Invalid response from Redis"),
            },
            Err(_err) => {
                bail!("Error getting session from Redis")
            }
        }
    }
}
