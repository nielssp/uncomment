/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment authentication handling

use actix_web::{HttpMessage, HttpResponse, cookie::Cookie, delete, error, get, post, put, web};
use argonautica::{Hasher, Verifier};
use chrono::{Duration, Local};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{db::{Pool, sessions::{self, Session}, users::{self, NewUser, User}}, settings::Settings};

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePassword {
    pub existing_password: String,
    pub new_password: String,
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

pub async fn validate_session(
    request: web::HttpRequest,
    pool: &web::Data<Pool>,
) -> actix_web::Result<Session> {
    match request.cookie("uncomment_session")    {
        Some(cookie) => {
            let session = sessions::get_session(pool, cookie.value()).await?
                .ok_or_else(|| error::ErrorUnauthorized("UNAUTHORIZED"))?;
            if session.valid_until >= Local::now() {
                Ok(session)
            } else {
                info!("session expired");
                sessions::delete_session(pool, &session.id).await?;
                Err(error::ErrorUnauthorized("UNAUTHORIZED"))
            }
        },
        None => Err(error::ErrorUnauthorized("MISSING_SESSION")),
    }
}

pub async fn validate_admin_session(
    request: web::HttpRequest,
    pool: &web::Data<Pool>,
) -> actix_web::Result<Session> {
    let session = validate_session(request, pool).await?;
    if session.user.admin {
        Ok(session)
    } else {
        Err(error::ErrorForbidden("INSUFFICIENT_PRIVILEGES"))
    }
}

pub fn generate_session_id() -> String {
    let bytes: [u8; 30] = rand::random();
    base64::encode(bytes)
}

pub fn hash_password(password: &str, settings: &Settings) -> actix_web::Result<String> {
    Hasher::default()
        .configure_iterations(settings.argon2_iterations)
        .configure_memory_size(settings.argon2_memory_size)
        .with_secret_key(&settings.secret_key)
        .with_password(password)
        .hash()
        .map_err(|e| {
            error!("hashing error {}", e);
            error::ErrorInternalServerError("INTERNAL_SERVER_ERROR")
        })
}

pub fn verify_password(hash: &str, password: &str, settings: &Settings) -> actix_web::Result<bool> {
    Verifier::default()
        .with_secret_key(&settings.secret_key)
        .with_hash(hash)
        .with_password(password)
        .verify()
        .map_err(|e| {
            error!("hash verification error {}", e);
            error::ErrorInternalServerError("INTERNAL_SERVER_ERROR")
        })
}

#[get("/auth")]
async fn get_auth(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
) -> actix_web::Result<HttpResponse> {
    let session = validate_session(request, &pool).await?;
    Ok(HttpResponse::Ok().json(SessionUser::from(session.user)))
}

#[post("/auth")]
async fn create_auth(
    data: web::Json<Credentials>,
    pool: web::Data<Pool>,
    settings: web::Data<Settings>,
) -> actix_web::Result<HttpResponse> {
    let password = users::get_password_by_username(&pool, &data.username).await?
        .ok_or_else(|| {
            info!("user not found: {}", data.username);
            error::ErrorBadRequest("INVALID_CREDENTIALS")
        })?;
    if verify_password(&password.password, &data.password, &settings)? {
        let session_id = generate_session_id();
        let user = users::get_user_by_id(&pool, password.user_id).await?
            .ok_or_else(|| {
                info!("user not found by id: {}", password.user_id);
                error::ErrorBadRequest("INVALID_CREDENTIALS")
            })?;
        sessions::create_session(&pool, &session_id, Local::now() + Duration::hours(1), user.id).await?;
        Ok(HttpResponse::Ok()
            .cookie(Cookie::build("uncomment_session", session_id)
                .max_age(time::Duration::hours(1))
                .http_only(true)
                .finish())
            .json(SessionUser::from(user)))
    } else {
        info!("invalid password");
        Err(error::ErrorBadRequest("INVALID_CREDENTIALS"))
    }
}

#[delete("/auth")]
async fn delete_auth(
    request: web::HttpRequest,
    pool: web::Data<Pool>
) -> actix_web::Result<HttpResponse> {
    let session = validate_session(request, &pool).await?;
    sessions::delete_session(&pool, &session.id).await?;
    Ok(HttpResponse::NoContent().body(""))
}

#[put("/password")]
async fn update_password(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    data: web::Json<UpdatePassword>,
    settings: web::Data<Settings>,
) -> actix_web::Result<HttpResponse> {
    let session = validate_session(request, &pool).await?;
    let password = users::get_password_by_user_id(&pool, session.user.id).await?
        .ok_or_else(|| {
            info!("user not found by id: {}", session.user.id);
            error::ErrorInternalServerError("INTERNAL_SERVER_ERROR")
        })?;
    if verify_password(&password.password, &data.existing_password, &settings)? {
        users::change_password(&pool, session.user.id, &hash_password(&data.new_password, &settings)?).await?;
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(error::ErrorBadRequest("INVALID_PASSWORD"))
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_auth)
        .service(create_auth)
        .service(delete_auth)
        .service(update_password);
}

pub async fn install(pool: &Pool, settings: &Settings) -> actix_web::Result<()> {
    if let (Some(username), Some(password)) = (
        settings.default_admin_username.as_ref(),
        settings.default_admin_password.as_ref(),
    ) {
        if users::admin_exists(pool).await? {
            return Ok(());
        }
        info!("Creating default admin user");
        users::create_user(pool, NewUser {
            username: username.clone(),
            password: hash_password(password, &settings)?,
            name: username.clone(),
            email: "".to_owned(),
            website: "".to_owned(),
            trusted: true,
            admin: true,
        }).await?;
    }
    Ok(())
}
