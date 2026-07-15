use actix_web::dev::ServiceRequest;
use actix_web::Error;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use log::warn;

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // We check against an environment variable or use a default
    let expected_token = std::env::var("API_TOKEN").unwrap_or_else(|_| "secret-token".to_string());
    
    if credentials.token() == expected_token {
        Ok(req)
    } else {
        warn!("Invalid token provided");
        Err((actix_web::error::ErrorUnauthorized("Invalid API token"), req))
    }
}
