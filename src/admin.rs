use actix_web::{HttpMessage, HttpResponse, cookie::Cookie, delete, error, get, put, post, web};

#[get("/admin/comments")]
async fn get_comments(
) -> actix_web::Result<HttpResponse> {
    Err(error::ErrorNotImplemented("not implemented"))
}

#[put("/admin/comments/{id:\\d+}")]
async fn update_comment(
    web::Path(id): web::Path<i64>,
) -> actix_web::Result<HttpResponse> {
    Err(error::ErrorNotImplemented("not implemented"))
}

#[delete("/admin/comments/{id:\\d+}")]
async fn delete_comment(
    web::Path(id): web::Path<i64>,
) -> actix_web::Result<HttpResponse> {
    Err(error::ErrorNotImplemented("not implemented"))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_comments)
        .service(update_comment)
        .service(delete_comment);
}

