use actix_web::{HttpResponse, get, post, delete, error, web};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub username: String,
    pub password: String,
}

#[get("/auth")]
async fn get_auth() -> actix_web::Result<HttpResponse> {
    Err(error::ErrorNotImplemented("not implemented"))
}

#[post("/auth")]
async fn create_auth() -> actix_web::Result<HttpResponse> {
    Err(error::ErrorNotImplemented("not implemented"))
}

#[delete("/auth")]
async fn delete_auth() -> actix_web::Result<HttpResponse> {
    Err(error::ErrorNotImplemented("not implemented"))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_auth)
        .service(create_auth)
        .service(delete_auth);
}
