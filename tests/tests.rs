#[path = "../src/handlers.rs"]
mod handlers;

#[path = "../src/models.rs"]
mod models;

#[cfg(test)]
mod tests {
    use super::handlers::*;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use actix_web::{web, App};
    use dotenv::dotenv;
    use serde_json::json;

    use porus::pocket_sdk::PocketSdk;

    #[actix_rt::test]
    async fn test_authenticate_user() {
        let pocket_sdk = PocketSdk::new("80908-b39061ed0999bb292f0fe716".to_string());
        let mut app = test::init_service(
            App::new()
                .wrap(actix_web::middleware::Logger::default())
                .app_data(web::Data::new(pocket_sdk.clone()))
                .configure(configure_routes),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/authenticate")
            .set_json(&json!({
                "redirect_uri": "http://example.com"
            }))
            .to_request();
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
        dotenv().ok();
        env_logger::init();

        let pocket_sdk = PocketSdk::new("80908-b39061ed0999bb292f0fe716".to_string());
        let mut app = test::init_service(
            App::new()
                .wrap(actix_web::middleware::Logger::default())
                .app_data(web::Data::new(pocket_sdk.clone()))
                .configure(configure_routes),
        )
        .await;

        // Step 1: Authenticate the user and obtain the request token
        let auth_req = test::TestRequest::post()
            .uri("/authenticate")
            .set_json(&json!({
                "redirect_uri": "http://example.com"
            }))
            .to_request();
        let auth_resp = test::call_service(&mut app, auth_req).await;

        assert_eq!(auth_resp.status(), StatusCode::OK);
        let auth_body = test::read_body(auth_resp).await;
        println!("Authentication Response body: {:?}", auth_body);
        let auth_json_body: serde_json::Value = serde_json::from_slice(&auth_body).unwrap();
        println!("Authentication JSON body: {:?}", auth_json_body);
        assert_eq!(auth_json_body["success"], true);
        let request_token = auth_json_body["request_token"]["code"].as_str().unwrap();
        println!("Request token: {}", request_token);

        // Step 2: Simulate user authorization on pocket.com
        // Replace this step with the actual authorization flow, such as redirecting the user to the Pocket website
        // Build the authorization URL
        let authorize_url = format!(
            "https://getpocket.com/auth/authorize?request_token={}&redirect_uri=http://example.com",
            request_token
        );
        println!("Authorization URL: {}", authorize_url);

        // Simulate waiting for user input
        println!("Please authorize the application using the above URL.");
        println!("Press Enter to continue after authorization...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        // Step 3: Save the access token using the obtained request token
        let save_token_req = test::TestRequest::post()
            .uri("/save-access-token")
            .set_json(&json!({ "request_token": request_token }))
            .to_request();
        let save_token_resp = test::call_service(&mut app, save_token_req).await;

        let save_token_status = save_token_resp.status();
        let save_token_body = test::read_body(save_token_resp).await;
        println!("Save Token Response body: {:?}", save_token_body);
        let save_token_json_body: serde_json::Value =
            serde_json::from_slice(&save_token_body).unwrap();

        assert_eq!(save_token_status, StatusCode::OK);
        assert_eq!(save_token_json_body["success"], true);
        assert_eq!(
            save_token_json_body["message"],
            "Pocket access token saved successfully."
        );
    }
}
