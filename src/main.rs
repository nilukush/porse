use actix_web::web::{Data, Json};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger;
use redis::AsyncCommands;
use redis::Client;
use redis::{ErrorKind, RedisError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::error::Error;
use std::fmt;

use porus::PocketSdk; // Replace `pocket_sdk` with the actual name of your SDK crate

// API handler for authenticating a user and obtaining a request token

// API handler for authenticating a user and obtaining a request token
async fn authenticate_user(pocket_sdk: web::Data<PocketSdk>) -> impl Responder {
    // Perform authentication logic using the Pocket SDK
    let request_token = match pocket_sdk.obtain_request_token().await {
        Ok(token) => token,
        Err(err) => {
            eprintln!("Error obtaining request token: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let response_body = json!({
        "success": true,
        "request_token": request_token,
    });

    HttpResponse::Ok()
        .content_type("application/json")
        .json(response_body)
}

// API handler for saving the Pocket access token
#[derive(Deserialize)]
struct AccessTokenRequest {
    request_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedisPocketAccessTokenResponse {
    access_token: String,
    username: String,
}

#[derive(Debug)]
struct CustomRedisError(String);

impl fmt::Display for CustomRedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CustomRedisError: {}", self.0)
    }
}

impl Error for CustomRedisError {}

impl From<(redis::ErrorKind, &'static str, String)> for CustomRedisError {
    fn from((kind, _desc, message): (redis::ErrorKind, &'static str, String)) -> Self {
        CustomRedisError(format!("{:?}: {}", kind, message))
    }
}

async fn save_access_token(
    form: Json<AccessTokenRequest>,
    pocket_sdk: Data<PocketSdk>,
    redis_client: Data<Client>,
) -> impl Responder {
    let request_token = &form.request_token;

    // Convert the request token to Pocket access token using the Pocket SDK
    let access_token_result = pocket_sdk
        .convert_request_token_to_access_token(request_token.as_str())
        .await;

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();

    if let Ok(access_token_response) = access_token_result {
        let redis_access_token_response = RedisPocketAccessTokenResponse {
            access_token: access_token_response.access_token,
            username: access_token_response.username,
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
            println!("Failed to store access token in Redis: {}", error);
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to store access token in Redis",
            }));
        }
    } else {
        println!("Access Token Conversion Failed: {:?}", access_token_result);
        return HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": "Access Token Conversion Failed",
        }));
    }

    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Pocket access token saved successfully.",
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    // Initialize the Pocket SDK and other dependencies
    let consumer_key = "80908-b39061ed0999bb292f0fe716".to_string();
    let redirect_uri = "pocketapp1234:authorizationFinished".to_string();
    let pocket_sdk = PocketSdk::new(consumer_key, redirect_uri); // Replace with the initialization logic for your SDK

    // Connect to Redis
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL not set");
    let redis_client = Client::open(redis_url).expect("Failed to connect to Redis");

    // Start the Actix Web server
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pocket_sdk.clone()))
            .app_data(Data::new(redis_client.clone()))
            .route("/authenticate", web::post().to(authenticate_user))
            .route("/save-access-token", web::post().to(save_access_token))
    })
    .bind("127.0.0.1:8080")? // Replace with the desired address and port
    .run()
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use serde_json::json;

    #[actix_rt::test]
    async fn test_authenticate_user() {
        let pocket_sdk = PocketSdk::new(
            "80908-b39061ed0999bb292f0fe716".to_string(),
            "pocketapp1234:authorizationFinished".to_string(),
        );
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pocket_sdk))
                .route("/authenticate", web::post().to(authenticate_user)),
        )
        .await;

        let req = test::TestRequest::post().uri("/authenticate").to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        println!("Response body: {:?}", body);
        let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        println!("JSON body: {:?}", json_body);
        assert_eq!(json_body["success"], true);
        let request_token = json_body["request_token"]["code"].as_str().unwrap();
        println!("Request token: {}", request_token);
    }

    #[actix_rt::test]
    async fn test_save_access_token() {
        let pocket_sdk = PocketSdk::new("consumer_key".to_string(), "redirect_uri".to_string());
        let redis_client = Client::open("redis_url").expect("Failed to connect to Redis");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pocket_sdk))
                .app_data(Data::new(redis_client))
                .route("/save-access-token", web::post().to(save_access_token)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/save-access-token")
            .set_json(&json!({ "request_token": "test_token" }))
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json_body["success"], true);
        assert_eq!(
            json_body["message"],
            "Pocket access token saved successfully."
        );
    }
}
