use actix_web::{App, HttpResponse, HttpServer, ResponseError, error::{self}, get, post, web};
use db::{PostComment, Repo, RepoError, SqlitePool};
use log::{debug, info};
use r2d2_sqlite::SqliteConnectionManager;
use serde::Deserialize;

mod db;

#[derive(Deserialize)]
struct CommentRequest {
    t: String,
    parent_id: Option<i64>,
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
    data: web::Json<PostComment>,
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
    let comment = repo.post_comment(thread.id, parent, data.into_inner())?;
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
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await
}
