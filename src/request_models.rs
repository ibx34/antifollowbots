use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubOAuthCallback {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubAccessToken {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}
