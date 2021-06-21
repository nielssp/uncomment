use std::env;

use actix_web::{HttpMessage, HttpResponse, cookie::Cookie, delete, error, get, post, web};
use argonautica::{Hasher, Verifier};
use chrono::{Duration, Local};
use log::{info, error};
use serde::{Deserialize, Serialize};

use crate::db::{NewUser, Repo, Session, User};

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct SessionUser {
    pub id: i64,
    pub username: String,
    pub name: String,
    pub email: String,
    pub website: String,
    pub trusted: bool,
    pub admin: bool,
}


impl From<User> for SessionUser {
    fn from(user: User) -> Self {
        SessionUser {
            id: user.id,
            username: user.username,
            name: user.name,
            email: user.email,
            website: user.website,
            trusted: user.trusted,
            admin: user.admin,
        }
    }
}

pub fn validate_session(
    request: web::HttpRequest,
    repo: &web::Data<Repo>,
) -> actix_web::Result<Session> {
    match request.cookie("uncomment_session")    {
        Some(cookie) => {
            let session = repo.get_session(cookie.value())?
                .ok_or_else(|| error::ErrorUnauthorized("unauthorized"))?;
            if session.valid_until >= Local::now() {
                Ok(session)
            } else {
                info!("session expired");
                repo.delete_session(&session.id)?;
                Err(error::ErrorUnauthorized("unauthorized"))
            }
        },
        None => Err(error::ErrorUnauthorized("missing session id")),
    }
}

pub fn generate_session_id() -> String {
    let bytes: [u8; 30] = rand::random();
    base64::encode(bytes)
}

pub fn get_secret_key() -> actix_web::Result<String> {
    env::var("UNCOMMENT_SECRET_KEY")
        .map_err(|_| {
            error::ErrorInternalServerError("missing secret key")
        })
}

pub fn hash_password(password: &str) -> actix_web::Result<String> {
    Hasher::default()
        .with_secret_key(get_secret_key()?)
        .with_password(password)
        .hash()
        .map_err(|e| {
            error!("hashing error {}", e);
            error::ErrorInternalServerError("internal server error")
        })
}

pub fn verify_password(hash: &str, password: &str) -> actix_web::Result<bool> {
    Verifier::default()
        .with_secret_key(get_secret_key()?)
        .with_hash(hash)
        .with_password(password)
        .verify()
        .map_err(|e| {
            error!("hash verification error {}", e);
            error::ErrorInternalServerError("internal server error")
        })
}

#[get("/auth")]
async fn get_auth(
    request: web::HttpRequest,
    repo: web::Data<Repo>,
) -> actix_web::Result<HttpResponse> {
    let session = validate_session(request, &repo)?;
    Ok(HttpResponse::Ok().json(SessionUser::from(session.user)))
}

#[post("/auth")]
async fn create_auth(
    data: web::Json<Credentials>,
    repo: web::Data<Repo>
) -> actix_web::Result<HttpResponse> {
    let user = repo.get_user(&data.username)?
        .ok_or_else(|| {
            info!("user not found: {}", data.username);
            error::ErrorBadRequest("invalid credentials")
        })?;
    if verify_password(&user.password, &data.password)? {
        let session_id = generate_session_id();
        repo.create_session(&session_id, Local::now() + Duration::hours(1), user.id)?;
        Ok(HttpResponse::Ok()
            .cookie(
                Cookie::build("uncomment_session", session_id)
                .finish()
            )
            .json(SessionUser::from(user)))
    } else {
        info!("invalid password");
        Err(error::ErrorBadRequest("invalid credentials"))
    }
}

#[delete("/auth")]
async fn delete_auth(
    request: web::HttpRequest,
    repo: web::Data<Repo>
) -> actix_web::Result<HttpResponse> {
    let session = validate_session(request, &repo)?;
    repo.delete_session(&session.id)?;
    Ok(HttpResponse::NoContent().body(""))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_auth)
        .service(create_auth)
        .service(delete_auth);
}

pub fn install(repo: &Repo) -> actix_web::Result<()> {
    if let (Ok(username), Ok(password)) = (
        env::var("UNCOMMENT_DEFAULT_ADMIN_USERNAME"),
        env::var("UNCOMMENT_DEFAULT_ADMIN_PASSWORD"),
    ) {
        if repo.admin_exists()? {
            return Ok(());
        }
        info!("Creating default admin user");
        repo.create_user(NewUser {
            username: username.clone(),
            password: hash_password(&password)?,
            name: username,
            email: "".to_owned(),
            website: "".to_owned(),
            trusted: true,
            admin: true,
        })?;
    }
    Ok(())
}
