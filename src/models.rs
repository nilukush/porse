use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AuthenticateUserRequest {
    pub redirect_uri: String,
}

#[derive(Deserialize)]
pub struct AccessTokenRequest {
    pub request_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedisPocketAccessTokenResponse {
    pub access_token: String,
    pub username: String,
}
