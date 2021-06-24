use actix_web::{HttpMessage, HttpResponse, cookie::Cookie, delete, error, get, put, post, web};

use crate::{auth, db::Repo};

#[derive(serde::Deserialize)]
struct CommentQuery {
    offset: Option<i64>,
}

#[get("/admin/comments")]
async fn get_comments(
    request: web::HttpRequest,
    repo: web::Data<Repo>,
    query: web::Query<CommentQuery>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &repo)?;
    Ok(HttpResponse::Ok().json(repo.get_comments(query.offset.unwrap_or(0))?))
}

#[put("/admin/comments/{id:\\d+}")]
async fn update_comment(
    request: web::HttpRequest,
    repo: web::Data<Repo>,
    web::Path(id): web::Path<i64>,
) -> actix_web::Result<HttpResponse> {
    let session = auth::validate_admin_session(request, &repo)?;
    Err(error::ErrorNotImplemented("not implemented"))
}

#[delete("/admin/comments/{id:\\d+}")]
async fn delete_comment(
    request: web::HttpRequest,
    repo: web::Data<Repo>,
    web::Path(id): web::Path<i64>,
) -> actix_web::Result<HttpResponse> {
    let session = auth::validate_admin_session(request, &repo)?;
    Err(error::ErrorNotImplemented("not implemented"))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_comments)
        .service(update_comment)
        .service(delete_comment);
}

