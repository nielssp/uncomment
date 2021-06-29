use actix_web::{HttpMessage, HttpResponse, cookie::Cookie, delete, error, get, put, post, web};
use pulldown_cmark::Parser;

use crate::{auth, db::{CommentStatus, Repo, UpdateComment}};

#[derive(serde::Deserialize)]
struct CommentQuery {
    offset: Option<usize>,
    status: Option<CommentStatus>,
    parent_id: Option<i64>,
    asc: Option<bool>,
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
    repo: web::Data<Repo>,
    query: web::Query<CommentQuery>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &repo)?;
    Ok(HttpResponse::Ok().json(repo.get_comments(query.status.unwrap_or(
                    CommentStatus::Pending), query.asc.unwrap_or(false), 10, query.offset.unwrap_or(0))?))
}

#[put("/admin/comments/{id:\\d+}")]
async fn update_comment(
    request: web::HttpRequest,
    repo: web::Data<Repo>,
    web::Path(id): web::Path<i64>,
    data: web::Json<UpdateCommentData>,
) -> actix_web::Result<HttpResponse> {
    let session = auth::validate_admin_session(request, &repo)?;
    let parser = Parser::new(data.markdown.as_str());
    let mut unsafe_html = String::new();
    pulldown_cmark::html::push_html(&mut unsafe_html, parser);
    let safe_html = ammonia::clean(&*unsafe_html);
    repo.update_comment(id, UpdateComment {
        name: data.name.clone(),
        email: data.email.clone(),
        website: data.website.clone(),
        markdown: data.markdown.clone(),
        html: safe_html,
        status: data.status,
    })?;
    Ok(HttpResponse::NoContent().body(""))
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

