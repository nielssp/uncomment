use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use db::{Repo, SqlitePool};
use r2d2_sqlite::SqliteConnectionManager;

mod db;

#[get("/")]
async fn hello(repo: web::Data<Repo>) -> actix_web::Result<HttpResponse> {
    let result = repo.get_comments();
    result.map(|comments| {
        HttpResponse::Ok().json(comments)
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let manager = SqliteConnectionManager::file("data.db");
    let pool = SqlitePool::new(manager).unwrap();
    let repo: Repo = Repo::SqliteRepo(pool);

    HttpServer::new(move || {
        App::new()
            .data(repo.clone())
            .service(hello)
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await
}
