use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use crate::models::{AccessTokenRequest, AuthenticateUserRequest, RedisPocketAccessTokenResponse};
use redis::AsyncCommands;
use redis::Client;
use redis::{ErrorKind, RedisError};

use porus::pocket_sdk::PocketSdk;

pub async fn authenticate_user(
    pocket_sdk: web::Data<PocketSdk>,
    form: web::Json<AuthenticateUserRequest>,
) -> impl Responder {
    // Log the incoming request
    let form_json = serde_json::to_string(&form);
    if let Ok(json) = form_json {
        log::info!("Received request: {}", json);
    } else {
        log::error!("Failed to serialize the request form to JSON");
    }

    let redirect_uri = &form.redirect_uri;

    // Perform authentication logic using the Pocket SDK
    let request_token = match pocket_sdk.obtain_request_token(redirect_uri).await {
        Ok(token) => token,
        Err(err) => {
            log::error!("Error obtaining request token: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let response_body = json!({
        "success": true,
        "request_token": request_token,
    });

    // Log the response
    let response_json = serde_json::to_string(&response_body);
    if let Ok(json) = response_json {
        log::info!("Sending response: {}", json);
    } else {
        log::error!("Failed to serialize the response body to JSON");
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .json(response_body)
}

pub async fn save_access_token(
    form: web::Json<AccessTokenRequest>,
    pocket_sdk: web::Data<PocketSdk>,
    redis_client: web::Data<Client>,
) -> impl Responder {
    // Log the incoming request
    let form_json = serde_json::to_string(&form);
    if let Ok(json) = form_json {
        log::info!("Received request: {}", json);
    } else {
        log::error!("Failed to serialize the request form to JSON");
    }

    let request_token = &form.request_token;

    // Convert the request token to Pocket access token using the Pocket SDK
    let access_token_result = pocket_sdk
        .convert_request_token_to_access_token(request_token.as_str())
        .await;

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();

    if let Ok(access_token_response) = access_token_result {
        let redis_access_token_response = RedisPocketAccessTokenResponse {
            access_token: access_token_response.access_token.clone(),
            username: access_token_response.username.clone(),
        };

        let redis_result: redis::RedisResult<()> =
            match serde_json::to_string(&redis_access_token_response) {
                Ok(serialized) => redis_conn.hset("access_token", "data", serialized).await,
                Err(err) => {
                    let error_message = format!("Failed to serialize access token: {}", err);
                    let error =
                        RedisError::from((ErrorKind::TypeError, "serialization", error_message))
                            .into();
                    Err(error)
                }
            };

        if let Err(error) = redis_result {
            log::error!("Failed to store access token in Redis: {}", error);
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to store access token in Redis",
                "access_token_response": access_token_response,
            }));
        }

        // Log the response
        let response_json = serde_json::to_string(&access_token_response);
        if let Ok(json) = response_json {
            log::info!("Sending response: {}", json);
        } else {
            log::error!("Failed to serialize the access token response to JSON");
        }

        HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Pocket access token saved successfully.",
            "access_token_response": access_token_response,
        }))
    } else {
        log::error!("Access Token Conversion Failed: {:?}", access_token_result);
        HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": "Access Token Conversion Failed",
            "access_token_response": access_token_result,
        }))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/authenticate", web::post().to(authenticate_user))
        .route("/save-access-token", web::post().to(save_access_token));
}
