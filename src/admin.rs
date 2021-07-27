/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment dashboard API

use actix_multipart::Multipart;
use actix_web::{HttpResponse, delete, error, get, put, post, web};
use log::info;
use pulldown_cmark::Parser;
use futures::{TryStreamExt, StreamExt};
use std::io::{Seek, SeekFrom, Write};

use crate::{auth::{self, hash_password}, db::{Pool, comments::{self, CommentFilter, CommentStatus, UpdateComment}, threads::{self, NewThread, UpdateThread}, users::{self, NewUser, UpdateUser}}, import, settings::Settings};

#[derive(serde::Deserialize)]
struct CommentQuery {
    offset: Option<usize>,
    status: Option<CommentStatus>,
    parent_id: Option<i32>,
    thread_id: Option<i32>,
    asc: Option<bool>,
}

#[derive(serde::Deserialize)]
struct ThreadQuery {
    offset: Option<usize>,
}

#[derive(serde::Deserialize)]
struct UserQuery {
    offset: Option<usize>,
}

#[derive(serde::Deserialize)]
struct UpdateCommentData {
    name: String,
    email: String,
    website: String,
    markdown: String,
    status: CommentStatus,
}

#[get("/admin/comments")]
async fn get_comments(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    query: web::Query<CommentQuery>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    let filter = query.parent_id.map(|id| CommentFilter::Parent(id))
        .unwrap_or_else(|| query.thread_id.map(|id| CommentFilter::Thread(id))
            .unwrap_or_else(|| CommentFilter::Status(query.status.unwrap_or(CommentStatus::Pending))));
    Ok(HttpResponse::Ok().json(comments::get_comments(&pool, filter,
                query.asc.unwrap_or(false), 10, query.offset.unwrap_or(0)).await?))
}

#[get("/admin/comments/{id:\\d+}")]
async fn get_comment(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i32>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    let comment = comments::get_comment(&pool, id).await?.ok_or_else(|| error::ErrorNotFound("NOT_FOUND"))?;
    Ok(HttpResponse::Ok().json(comment))
}

#[put("/admin/comments/{id:\\d+}")]
async fn update_comment(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i32>,
    data: web::Json<UpdateCommentData>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    let mut comment = comments::get_comment(&pool, id).await?.ok_or_else(|| error::ErrorNotFound("NOT_FOUND"))?;
    let parser = Parser::new(data.markdown.as_str());
    let mut unsafe_html = String::new();
    pulldown_cmark::html::push_html(&mut unsafe_html, parser);
    comment.name = data.name.clone();
    comment.email = data.email.clone();
    comment.website = data.website.clone();
    comment.markdown = data.markdown.clone();
    comment.html = ammonia::clean(&*unsafe_html);
    comment.status = data.status;
    comments::update_comment(&pool, id, UpdateComment {
        name: data.name.clone(),
        email: data.email.clone(),
        website: data.website.clone(),
        markdown: data.markdown.clone(),
        html: comment.html.clone(),
        status: data.status,
    }).await?;
    Ok(HttpResponse::Ok().json(comment))
}

#[delete("/admin/comments/{id:\\d+}")]
async fn delete_comment(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i32>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    comments::delete_comment(&pool, id).await?;
    Ok(HttpResponse::NoContent().body(""))
}

#[get("/admin/threads")]
async fn get_threads(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    query: web::Query<ThreadQuery>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    Ok(HttpResponse::Ok().json(threads::get_threads(&pool, 30, query.offset.unwrap_or(0)).await?))
}

#[post("/admin/threads")]
async fn create_thread(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    data: web::Json<NewThread>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    Ok(HttpResponse::Ok().json(threads::create_thread(&pool, data.into_inner()).await?))
}

#[get("/admin/threads/{id:\\d+}")]
async fn get_thread(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i32>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    let thread = threads::get_thread_by_id(&pool, id).await?.ok_or_else(|| error::ErrorNotFound("NOT_FOUND"))?;
    Ok(HttpResponse::Ok().json(thread))
}

#[put("/admin/threads/{id:\\d+}")]
async fn update_thread(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i32>,
    data: web::Json<UpdateThread>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    let mut thread = threads::get_thread_by_id(&pool, id).await?.ok_or_else(|| error::ErrorNotFound("NOT_FOUND"))?;
    thread.title = data.title.clone();
    threads::update_thread(&pool, id, data.into_inner()).await?;
    Ok(HttpResponse::Ok().json(thread))
}

#[delete("/admin/threads/{id:\\d+}")]
async fn delete_thread(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i32>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    threads::delete_thread(&pool, id).await?;
    Ok(HttpResponse::NoContent().body(""))
}

#[get("/admin/users")]
async fn get_users(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    query: web::Query<UserQuery>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    Ok(HttpResponse::Ok().json(users::get_users(&pool, 30, query.offset.unwrap_or(0)).await?))
}

#[post("/admin/users")]
async fn create_user(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    data: web::Json<NewUser>,
    settings: web::Data<Settings>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    let mut user = data.into_inner();
    user.password = hash_password(&user.password, &settings)?;
    Ok(HttpResponse::Ok().json(users::create_user(&pool, user).await?))
}

#[get("/admin/ussers/{id:\\d+}")]
async fn get_user(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i32>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    let user = users::get_user_by_id(&pool, id).await?.ok_or_else(|| error::ErrorNotFound("NOT_FOUND"))?;
    Ok(HttpResponse::Ok().json(user))
}

#[put("/admin/users/{id:\\d+}")]
async fn update_user(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i32>,
    data: web::Json<UpdateUser>,
    settings: web::Data<Settings>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    let mut user = users::get_user_by_id(&pool, id).await?.ok_or_else(|| error::ErrorNotFound("NOT_FOUND"))?;
    user.username = data.username.clone();
    user.name = data.name.clone();
    user.email = data.email.clone();
    user.website = data.website.clone();
    user.trusted = data.trusted;
    user.admin = data.admin;
    let mut update = data.into_inner();
    if let Some(password) = update.password {
        update.password = Some(hash_password(&password, &settings)?);
    }
    users::update_user(&pool, id, update).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[delete("/admin/users/{id:\\d+}")]
async fn delete_user(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i32>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    users::delete_user(&pool, id).await?;
    Ok(HttpResponse::NoContent().body(""))
}

#[post("/admin/import")]
async fn import_comments(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    mut payload: Multipart,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    while let Some(mut field) = payload.try_next().await? {
        let mut f = web::block(|| tempfile::tempfile()).await?;
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
        f = web::block(move || f.seek(SeekFrom::Start(0)).map(|_| f)).await?;
        info!("Importing comments from XML file");
        let comments = web::block(move || import::read_xml_comments(f)).await?;
        import::insert_imported_comments(&pool, comments).await?;
    }
    Ok(HttpResponse::NoContent().body(""))
}


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_comments)
        .service(get_comment)
        .service(update_comment)
        .service(delete_comment)
        .service(get_threads)
        .service(create_thread)
        .service(get_thread)
        .service(update_thread)
        .service(delete_thread)
        .service(get_users)
        .service(create_user)
        .service(get_user)
        .service(update_user)
        .service(delete_user)
        .service(import_comments);
}

