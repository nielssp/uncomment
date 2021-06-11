use actix_web::{App, HttpResponse, HttpServer, Responder, error::InternalError, get, http::StatusCode, post, web};
use db::{PostComment, Repo, SqlitePool};
use r2d2_sqlite::SqliteConnectionManager;

mod db;

#[get("/")]
async fn get_comments(repo: web::Data<Repo>) -> actix_web::Result<HttpResponse> {
    let comments = repo.get_comments().map_err(|e| {
        InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    Ok(HttpResponse::Ok().json(comments))
}

#[post("/")]
async fn post_comment(data: web::Json<PostComment>, repo: web::Data<Repo>) -> actix_web::Result<HttpResponse> {
    let comment = repo.post_comment(data.into_inner()).map_err(|e| {
        InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    Ok(HttpResponse::Ok().json(comment))
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
