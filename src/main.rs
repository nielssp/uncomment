use actix_web::{App, HttpResponse, HttpServer, ResponseError, error::{self}, get, post, web};
use db::{NewComment, Repo, RepoError, SqlitePool};
use log::{debug, info};
use pulldown_cmark::Parser;
use r2d2_sqlite::SqliteConnectionManager;
use serde::Deserialize;

mod db;
mod migrations;

#[derive(Deserialize)]
struct CommentRequest {
    t: String,
    parent_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct NewCommentData {
    pub name: String,
    pub content: String,
}

impl ResponseError for RepoError {
}

#[get("/")]
async fn get_comments(request: web::Query<CommentRequest>, repo: web::Data<Repo>) -> actix_web::Result<HttpResponse> {
    debug!("comments requested for {}", request.t);
    let comments = repo.get_comments(request.t.clone())?;
    Ok(HttpResponse::Ok().json(comments))
}

#[post("/")]
async fn post_comment(
    request: web::Query<CommentRequest>,
    data: web::Json<NewCommentData>,
    repo: web::Data<Repo>
) -> actix_web::Result<HttpResponse> {
    let thread = match repo.get_thread(request.t.clone())? {
        Some(t) => Ok(t),
        None => {
            // TODO: setting to disable automatic creation of threads
            // TODO: setting to validate thread name by making a GET request to the site
            repo.create_thread(request.t.clone()).map(|t| {
                info!("Created new thread: '{}' (id: {})", t.name, t.id);
                t
            })
        },
    }?;
    let parent = match request.parent_id {
        Some(id) => {
            repo.get_comment_position(id)?
                .filter(|pos| pos.thread_id == thread.id)
                .ok_or_else(|| error::ErrorBadRequest("invalid parent_id"))
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
    let parser = Parser::new(data.content.as_str());
    let mut unsafe_html = String::new();
    pulldown_cmark::html::push_html(&mut unsafe_html, parser);
    let safe_html = ammonia::clean(&*unsafe_html);
    let comment = repo.post_comment(thread.id, parent, NewComment {
        name: data.name.clone(),
        markdown: data.content.clone(),
        html: safe_html,
    })?;
    Ok(HttpResponse::Ok().json(comment))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("Starting uncomment server...");

    let manager = SqliteConnectionManager::file("data.db");
    let pool = SqlitePool::new(manager).unwrap();
    let repo: Repo = Repo::SqliteRepo(pool);

    repo.install().unwrap();

    HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();
        App::new()
            .wrap(cors)
            .data(repo.clone())
            .service(get_comments)
            .service(post_comment)
            .service(actix_files::Files::new("/", "dist"))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await
}
