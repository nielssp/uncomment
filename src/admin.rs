/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment dashboard API

use actix_web::{HttpResponse, delete, error, get, put, post, web};
use pulldown_cmark::Parser;

use crate::{auth, db::{Pool, comments::{self, CommentFilter, CommentStatus, UpdateComment}, threads::{self, NewThread, UpdateThread}}};

#[derive(serde::Deserialize)]
struct CommentQuery {
    offset: Option<usize>,
    status: Option<CommentStatus>,
    parent_id: Option<i64>,
    thread_id: Option<i64>,
    asc: Option<bool>,
}

#[derive(serde::Deserialize)]
struct ThreadQuery {
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
    web::Path(id): web::Path<i64>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    let comment = comments::get_comment(&pool, id).await?.ok_or_else(|| error::ErrorNotFound("NOT_FOUND"))?;
    Ok(HttpResponse::Ok().json(comment))
}

#[put("/admin/comments/{id:\\d+}")]
async fn update_comment(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i64>,
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
    web::Path(id): web::Path<i64>,
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
    web::Path(id): web::Path<i64>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    let thread = threads::get_thread_by_id(&pool, id).await?.ok_or_else(|| error::ErrorNotFound("NOT_FOUND"))?;
    Ok(HttpResponse::Ok().json(thread))
}

#[put("/admin/threads/{id:\\d+}")]
async fn update_thread(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i64>,
    data: web::Json<UpdateThread>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    let mut thread = threads::get_thread_by_id(&pool, id).await?.ok_or_else(|| error::ErrorNotFound("thread not found"))?;
    thread.title = data.title.clone();
    threads::update_thread(&pool, id, data.into_inner()).await?;
    Ok(HttpResponse::Ok().json(thread))
}

#[delete("/admin/threads/{id:\\d+}")]
async fn delete_thread(
    request: web::HttpRequest,
    pool: web::Data<Pool>,
    web::Path(id): web::Path<i64>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &pool).await?;
    threads::delete_thread(&pool, id).await?;
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
        .service(delete_thread);
}

