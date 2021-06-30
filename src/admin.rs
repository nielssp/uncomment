use actix_web::{HttpMessage, HttpResponse, cookie::Cookie, delete, error, get, put, post, web};
use pulldown_cmark::Parser;

use crate::{auth, db::{CommentFilter, CommentStatus, Repo, UpdateComment}};

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
    let filter = query.parent_id.map(|id| CommentFilter::Parent(id))
        .unwrap_or_else(|| CommentFilter::Status(query.status.unwrap_or(CommentStatus::Pending)));
    Ok(HttpResponse::Ok().json(repo.get_comments(filter,
                query.asc.unwrap_or(false), 10, query.offset.unwrap_or(0))?))
}

#[get("/admin/comments/{id:\\d+}")]
async fn get_comment(
    request: web::HttpRequest,
    repo: web::Data<Repo>,
    web::Path(id): web::Path<i64>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &repo)?;
    let comment = repo.get_comment(id)?.ok_or_else(|| error::ErrorNotFound("comment not found"))?;
    Ok(HttpResponse::Ok().json(comment))
}

#[put("/admin/comments/{id:\\d+}")]
async fn update_comment(
    request: web::HttpRequest,
    repo: web::Data<Repo>,
    web::Path(id): web::Path<i64>,
    data: web::Json<UpdateCommentData>,
) -> actix_web::Result<HttpResponse> {
    auth::validate_admin_session(request, &repo)?;
    let mut comment = repo.get_comment(id)?.ok_or_else(|| error::ErrorNotFound("comment not found"))?;
    let parser = Parser::new(data.markdown.as_str());
    let mut unsafe_html = String::new();
    pulldown_cmark::html::push_html(&mut unsafe_html, parser);
    comment.name = data.name.clone();
    comment.email = data.email.clone();
    comment.website = data.website.clone();
    comment.markdown = data.markdown.clone();
    comment.html = ammonia::clean(&*unsafe_html);
    comment.status = data.status;
    repo.update_comment(id, UpdateComment {
        name: data.name.clone(),
        email: data.email.clone(),
        website: data.website.clone(),
        markdown: data.markdown.clone(),
        html: comment.html.clone(),
        status: data.status,
    })?;
    Ok(HttpResponse::Ok().json(comment))
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
        .service(get_comment)
        .service(update_comment)
        .service(delete_comment);
}

