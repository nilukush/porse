use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use env_logger;
use redis::Client;
use std::env;

mod handlers;
mod models;

pub use handlers::*;
pub use models::*;

use crate::handlers::{authenticate_user, save_access_token};

use porus::pocket_sdk::PocketSdk;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    // Initialize the Pocket SDK and other dependencies
    let consumer_key = "80908-b39061ed0999bb292f0fe716".to_string();
    let pocket_sdk = PocketSdk::new(consumer_key); // Replace with the initialization logic for your SDK

    // Connect to Redis
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL not set");
    let redis_client = Client::open(redis_url).expect("Failed to connect to Redis");

    // Start the Actix Web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pocket_sdk.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            .route("/authenticate", web::post().to(authenticate_user))
            .route("/save-access-token", web::post().to(save_access_token))
    })
    .bind("127.0.0.1:8080")? // Replace with the desired address and port
    .run()
    .await?;

    Ok(())
}
