use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Comment {
    pub id: i32,
    pub name: String,
    pub content: String,
}

pub type SqlitePool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

#[derive(Clone)]
pub enum Repo {
    SqliteRepo(SqlitePool),
}

impl Repo {
    pub fn get_comments(&self) -> actix_web::Result<Vec<Comment>> {
        match self {
            Repo::SqliteRepo(pool) => {
                actix_web::Result::Ok(vec![])
            },
        }
    }
}
