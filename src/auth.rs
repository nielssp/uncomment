/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment authentication handling

use actix_web::{HttpMessage, HttpResponse, cookie::Cookie, delete, error, get, post, put, web};
use argonautica::{Hasher, Verifier};
use chrono::{Duration, Local};
use log::{info, error};
use serde::{Deserialize, Serialize};

use crate::{db::{NewUser, Repo, Session, User}, settings::Settings};

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

pub fn validate_admin_session(
    request: web::HttpRequest,
    repo: &web::Data<Repo>,
) -> actix_web::Result<Session> {
    let session = validate_session(request, repo)?;
    if session.user.admin {
        Ok(session)
    } else {
        Err(error::ErrorForbidden("insufficient privileges"))
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
            error::ErrorInternalServerError("internal server error")
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
    repo: web::Data<Repo>,
    settings: web::Data<Settings>,
) -> actix_web::Result<HttpResponse> {
    let user = repo.get_user(&data.username)?
        .ok_or_else(|| {
            info!("user not found: {}", data.username);
            error::ErrorBadRequest("invalid credentials")
        })?;
    if verify_password(&user.password, &data.password, &settings)? {
        let session_id = generate_session_id();
        repo.create_session(&session_id, Local::now() + Duration::hours(1), user.id)?;
        Ok(HttpResponse::Ok()
            .cookie(Cookie::build("uncomment_session", session_id)
                .max_age(time::Duration::hours(1))
                .http_only(true)
                .finish())
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

#[put("/password")]
async fn update_password(
    request: web::HttpRequest,
    repo: web::Data<Repo>,
    data: web::Json<UpdatePassword>,
    settings: web::Data<Settings>,
) -> actix_web::Result<HttpResponse> {
    let session = validate_session(request, &repo)?;
    if verify_password(&session.user.password, &data.existing_password, &settings)? {
        repo.change_password(session.user.id, &hash_password(&data.new_password, &settings)?)?;
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(error::ErrorBadRequest("invalid password"))
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_auth)
        .service(create_auth)
        .service(delete_auth)
        .service(update_password);
}

pub fn install(repo: &Repo, settings: &Settings) -> actix_web::Result<()> {
    if let (Some(username), Some(password)) = (
        settings.default_admin_username.as_ref(),
        settings.default_admin_password.as_ref(),
    ) {
        if repo.admin_exists()? {
            return Ok(());
        }
        info!("Creating default admin user");
        repo.create_user(NewUser {
            username: username.clone(),
            password: hash_password(password, &settings)?,
            name: username.clone(),
            email: "".to_owned(),
            website: "".to_owned(),
            trusted: true,
            admin: true,
        })?;
    }
    Ok(())
}
