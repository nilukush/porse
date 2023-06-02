use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use redis::Client;
use std::env;

mod handlers;
mod models;

pub use handlers::*;
pub use models::*;

use porus::pocket_sdk::PocketSdk;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Create a file appender
    let file_path = "logs/porse.log";
    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} [{l}] - {m}{n}")))
        .build(file_path)
        .expect("Failed to create file appender");

    // Configure the root logger with the file appender
    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(
            Root::builder().appender("file").build(LevelFilter::Debug), // Set the log level to Debug
        )
        .expect("Failed to build log4rs config");

    // Initialize the logger with the configuration
    log4rs::init_config(config).expect("Failed to initialize log4rs");

    log::info!("Logging initialized");
    log::info!("Staring the application");

    // Initialize the Pocket SDK and other dependencies
    let consumer_key = "80908-b39061ed0999bb292f0fe716".to_string();
    let pocket_sdk = PocketSdk::new(consumer_key); // Replace with the initialization logic for your SDK

    // Connect to Redis
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL not set");
    let redis_client = Client::open(redis_url).expect("Failed to connect to Redis");

    // Start the Actix Web server
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::new(pocket_sdk.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            .configure(configure_routes)
    })
    .bind("0.0.0.0:8080")? // Replace with the desired address and port
    .run()
    .await?;

    Ok(())
}
