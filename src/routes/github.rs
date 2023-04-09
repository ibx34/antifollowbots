use std::collections::HashMap;

use crate::{
    config::CONFIG,
    models::{GithubUser, GoodUser, Session},
    request_models::{GithubAccessToken, GithubOAuthCallback},
    AppState,
};
use axum::{
    extract::{Query, State},
    http::header,
    http::StatusCode,
    response::IntoResponse,
    Form,
};
use chrono::{Duration, NaiveDateTime, Utc};
use ring::rand::SecureRandom;
use serde_json::json;

pub async fn github_oauth_callback(
    State(app): State<AppState>,
    Query(query): Query<GithubOAuthCallback>,
) -> impl IntoResponse {
    let request_client = reqwest::ClientBuilder::new()
        .user_agent("Dogma (AntiFollowBotSite)/1.0")
        .build()
        .unwrap();

    let config = CONFIG.clone();
    let mut access_token_body = HashMap::new();
    access_token_body.insert("client_id", &config.github_id);
    access_token_body.insert("client_secret", &config.github_secret);
    access_token_body.insert("code", &query.code);
    access_token_body.insert("redirect_uri", &config.callback);

    let access_token_request = request_client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&access_token_body)
        .build()
        .unwrap();

    let access_token = request_client.execute(access_token_request).await.unwrap();
    let Ok(access_token) = access_token.json::<GithubAccessToken>().await else {
        return (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, "https://followbots.yiff.day/error?err='Failed to get access token from Github.'")],
            json!({
                "msg": "Failed to get access token from Github."
            }).to_string(),
        )
            .into_response();
    };

    let get_user_request = request_client
        .get("https://api.github.com/user")
        .header(
            "Authorization",
            &format!("{} {}", access_token.token_type, access_token.access_token),
        )
        .build()
        .unwrap();

    let user = request_client.execute(get_user_request).await.unwrap();
    let Ok(user) = user.json::<GithubUser>().await else {
        return (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, "https://followbots.yiff.day/error?err='Failed to get you from Github :('")],
            json!({
                "msg": "Failed to get authorized user from Github"
            }).to_string(),
        )
            .into_response();
    };

    let user = match sqlx::query_as::<_, GoodUser>(r#"SELECT * FROM good_users WHERE github = $1"#)
        .bind(user.id)
        .fetch_one(&app.database.0)
        .await
    {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            match sqlx::query_as::<_,GoodUser>(r#"INSERT INTO good_users(github) VALUES($1) RETURNING *"#)
                .bind(user.id)
                .fetch_one(&app.database.0)
                .await
            {
                Ok(user) => user,
                Err(_) => return (
                    StatusCode::TEMPORARY_REDIRECT,
                    [(header::LOCATION, "https://followbots.yiff.day/error?err='Failed to insert or get you from the database.'")],
                    json!({
                        "msg": "Failed to insert or get you from the database."
                    }).to_string(),
                )
                    .into_response()
            }
        }
        Err(err) => {
            println!("{err:?}");
            return (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, "https://followbots.yiff.day/error?err='Error while interacting with the database.'")],
            json!({
                "msg": "Error while interacting with the database."
            }).to_string(),
        )
            .into_response();
        }
    };

    // check if they have a session already
    let Ok(session) = Session::create(app, user.id, true).await else {
        return (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, "https://followbots.yiff.day/error?err='Error creating session.'")],
            json!({
                "msg": "Error while creating session."
            }).to_string(),
        )
            .into_response();
    };

    let now = Utc::now() + Duration::weeks(1);

    return (
        StatusCode::TEMPORARY_REDIRECT,
        [
            (header::LOCATION, "https://followbots.yiff.day"),
            (
                header::SET_COOKIE,
                &format!(
                    "_dogma_session={}; Expires={}; Secure; HttpOnly; SameSite=Lax",
                    session.key,
                    now.to_string()
                ),
            ),
        ],
        json!({}).to_string(),
    )
        .into_response();
}

pub async fn github_oauth_redirect(State(_app): State<AppState>) -> impl IntoResponse {
    let mut state: [u8; 64] = [0; 64];
    let sr = ring::rand::SystemRandom::new();
    if sr.fill(&mut state).is_err() {
        return (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, "https://followbots.yiff.day/error?err='Failed to create a crucial component of oauth. (1)'")],
            json!({
                "msg": "Failed to create a crucial component of oauth. (1)"
            }).to_string(),
        )
            .into_response();
    }

    return (
        StatusCode::TEMPORARY_REDIRECT,
        // Even though we ask them to login with Github they may need to provide an access token generated on Github to allow us to block accounts on their behalf
        [(header::LOCATION, &format!("https://github.com/login/oauth/authorize?client_id={}&redirect_uri=https://api.yiff.day/oauth/callback&scope=read:user%20user:follow&state={}", CONFIG.github_id, hex::encode(state)))],
        json!({}).to_string(),
    )
        .into_response();
}
