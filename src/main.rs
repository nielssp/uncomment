/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment server

use actix_web::{App, HttpResponse, HttpServer, ResponseError, error, get, post, web};
use chrono::{Duration, Utc};
use db::{DbError, Pool, comments::{self, CommentStatus, NewComment}, threads::{self, NewThread}};
use dotenv::dotenv;
use log::{debug, info};
use pulldown_cmark::Parser;
use serde::Deserialize;

use crate::settings::Settings;

mod db;
mod auth;
mod admin;
mod settings;
mod import;

#[derive(Deserialize)]
struct CountQuery {
    t: String,
}

#[derive(Deserialize)]
struct CommentQuery {
    t: String,
    parent_id: Option<i64>,
    newest_first: Option<bool>,
}

#[derive(Deserialize)]
struct NewCommentData {
    name: String,
    email: String,
    website: String,
    content: String,
}

impl ResponseError for DbError {
}

#[get("/count")]
async fn count_comments(
    query: web::Query<CountQuery>,
    pool: web::Data<Pool>,
) -> actix_web::Result<HttpResponse> {
    let thread_names: Vec<&str> = query.t.split(",").collect();
    let counts = comments::count_comments_by_thread(&pool, thread_names).await?;
    Ok(HttpResponse::Ok().json(counts))
}

#[get("/comments")]
async fn get_comments(
    query: web::Query<CommentQuery>,
    pool: web::Data<Pool>,
    settings: web::Data<Settings>,
) -> actix_web::Result<HttpResponse> {
    debug!("comments requested for {}", query.t);
    let comments = comments::get_comment_thread(&pool, &query.t, query.newest_first.unwrap_or(false), settings.max_depth)
        .await?;
    Ok(HttpResponse::Ok().json(comments))
}

#[post("/comments")]
async fn post_comment(
    request: web::HttpRequest,
    query: web::Query<CommentQuery>,
    data: web::Json<NewCommentData>,
    pool: web::Data<Pool>,
    settings: web::Data<Settings>,
) -> actix_web::Result<HttpResponse> {
    let ip = request.peer_addr().map(|a| a.ip().to_string()).unwrap_or("".to_owned());
    if settings.rate_limit > 0 {
        let count = comments::count_comments_by_ip(&pool, &ip, Utc::now() - Duration::minutes(settings.rate_limit_interval)).await?;
        info!("rate limit: {} / {} comments in the past {} minutes", count, settings.rate_limit,
            settings.rate_limit_interval);
        if count >= settings.rate_limit {
            Err(error::ErrorTooManyRequests("TOO_MANY_COMMENTS"))?;
        }
    }
    let thread = match threads::get_thread_by_name(&pool, &query.t).await? {
        Some(t) => Ok(t),
        None => {
            if settings.auto_threads {
                // TODO: setting to validate thread name by making a GET request to the site
                Ok(threads::create_thread(&pool, NewThread {
                    name: query.t.clone(),
                    title: None,
                }).await.map(|t| {
                    info!("Created new thread: '{}' (id: {})", t.name, t.id);
                    t
                })?)
            } else {
                Err(error::ErrorBadRequest("THREAD_NOT_FOUND"))
            }
        },
    }?;
    let parent = match query.parent_id {
        Some(id) => {
            comments::get_comment_position(&pool, id).await?
                .filter(|pos| pos.thread_id == thread.id && pos.status == CommentStatus::Approved)
                .ok_or_else(|| error::ErrorBadRequest("PARENT_NOT_FOUND"))
                .map(|pos| {
                    debug!("Repying to comment {} in thread {}", pos.id, thread.id);
                    Some(pos)
                })
        }
        None => {
            debug!("Adding comment to thread {}", thread.id);
            Ok(None)
        }
    }?;
    if data.content.is_empty() {
        Err(error::ErrorBadRequest("MISSING_CONTENT"))?;
    }
    if settings.require_name && data.name.is_empty() {
        Err(error::ErrorBadRequest("MISSING_NAME"))?;
    }
    if settings.require_email && data.email.is_empty() {
        Err(error::ErrorBadRequest("MISSING_EMAIL"))?;
    }
    let parser = Parser::new(data.content.as_str());
    let mut unsafe_html = String::new();
    pulldown_cmark::html::push_html(&mut unsafe_html, parser);
    let safe_html = ammonia::clean(&unsafe_html);
    let comment = comments::post_comment(&pool, thread.id, parent.as_ref(), settings.max_depth, NewComment {
        name: data.name.clone(),
        email: data.email.clone(),
        website: data.website.clone(),
        ip,
        markdown: data.content.clone(),
        html: safe_html,
        status: if settings.moderate_all { CommentStatus::Pending } else { CommentStatus::Approved },
        created: Utc::now(),
    }).await?;
    Ok(HttpResponse::Ok().json(comment))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    info!("Starting uncomment server...");

    let settings = Settings::new().unwrap();

    let pool: Pool = db::install(&settings).await.unwrap();

    auth::install(&pool, &settings).await.unwrap();

    auth::cleanup(&pool).await.unwrap();

    let address = settings.listen.clone();

    HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();
        App::new()
            .wrap(cors)
            .data(pool.clone())
            .data(settings.clone())
            .service(count_comments)
            .service(get_comments)
            .service(post_comment)
            .configure(auth::config)
            .configure(admin::config)
            .service(actix_files::Files::new("/", "dist").index_file("index.html"))
    })
    .bind(address)?
    .run()
    .await
}
