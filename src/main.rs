use actix_web::{App, HttpResponse, HttpServer, Responder, error::InternalError, get, http::StatusCode, post, web};
use db::{Repo, SqlitePool};
use r2d2_sqlite::SqliteConnectionManager;

mod db;

#[get("/")]
async fn get_comments(repo: web::Data<Repo>) -> actix_web::Result<HttpResponse> {
    let comments = repo.get_comments().map_err(|e| {
        InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    Ok(HttpResponse::Ok().json(comments))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let manager = SqliteConnectionManager::file("data.db");
    let pool = SqlitePool::new(manager).unwrap();
    let repo: Repo = Repo::SqliteRepo(pool);

    repo.install().unwrap();

    HttpServer::new(move || {
        App::new()
            .data(repo.clone())
            .service(get_comments)
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await
}
