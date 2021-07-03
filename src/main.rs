/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment server

use actix_web::{App, HttpResponse, HttpServer, ResponseError, error, get, post, web};
use chrono::{Duration, Local};
use db::{NewComment, Repo, RepoError, SqlitePool};
use dotenv::dotenv;
use log::{debug, info};
use pulldown_cmark::Parser;
use r2d2_sqlite::SqliteConnectionManager;
use serde::Deserialize;

use crate::{db::CommentStatus, settings::Settings};

mod db;
mod migrations;
mod auth;
mod admin;
mod settings;

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

impl ResponseError for RepoError {
}

#[get("/comments")]
async fn get_comments(
    query: web::Query<CommentQuery>,
    repo: web::Data<Repo>,
) -> actix_web::Result<HttpResponse> {
    debug!("comments requested for {}", query.t);
    let comments = repo.get_comment_thread(&query.t, query.newest_first.unwrap_or(false))?;
    Ok(HttpResponse::Ok().json(comments))
}

#[post("/comments")]
async fn post_comment(
    request: web::HttpRequest,
    query: web::Query<CommentQuery>,
    data: web::Json<NewCommentData>,
    repo: web::Data<Repo>,
    settings: web::Data<Settings>,
) -> actix_web::Result<HttpResponse> {
    let ip = request.peer_addr().map(|a| a.ip().to_string()).unwrap_or("".to_owned());
    let count = repo.count_comments_by_ip(&ip, Local::now() - Duration::minutes(10))?;
    info!("{} comments", count);
    if count >= 10 {
        Err(error::ErrorTooManyRequests("TOO_MANY_COMMENTS"))?;
    }
    let thread = match repo.get_thread(&query.t)? {
        Some(t) => Ok(t),
        None => {
            if settings.auto_threads {
                // TODO: setting to validate thread name by making a GET request to the site
                Ok(repo.create_thread(&query.t).map(|t| {
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
            repo.get_comment_position(id)?
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
    let safe_html = ammonia::clean(&*unsafe_html);
    let comment = repo.post_comment(thread.id, parent.as_ref(), NewComment {
        name: data.name.clone(),
        email: data.email.clone(),
        website: data.website.clone(),
        ip,
        markdown: data.content.clone(),
        html: safe_html,
        status: if settings.moderate_all { CommentStatus::Pending } else { CommentStatus::Approved },
    })?;
    Ok(HttpResponse::Ok().json(comment))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    info!("Starting uncomment server...");

    let settings = Settings::new().unwrap();

    let manager = SqliteConnectionManager::file(&settings.sqlite_database);
    let pool = SqlitePool::new(manager).unwrap();
    let repo: Repo = Repo::SqliteRepo(pool);

    repo.install().unwrap();
    auth::install(&repo, &settings).unwrap();

    let address = settings.listen.clone();

    HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();
        App::new()
            .wrap(cors)
            .data(repo.clone())
            .data(settings.clone())
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
