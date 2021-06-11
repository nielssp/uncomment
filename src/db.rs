use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize)]
pub struct Comment {
    pub id: i64,
    pub name: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct PostComment {
    pub name: String,
    pub content: String,
}

pub type SqlitePool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

#[derive(Clone)]
pub enum Repo {
    SqliteRepo(SqlitePool),
}

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("r2d2 error")]
    R2d2Error(#[from] r2d2::Error),
    #[error("sqlite error")]
    SqliteError(#[from] rusqlite::Error),
}

impl Repo {
    pub fn install(&self) -> Result<(), RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                let mut stmt = conn.prepare("pragma table_info('comments')")?;
                let mut rows = stmt.query([])?;
                if rows.next()?.is_none() {
                    println!("installing sqlite3 database...");
                    let mut create = conn.prepare(
                        "create table comments (
                            id integer primary key autoincrement,
                            name text(100) not null,
                            content text not null
                        )"
                    )?;
                    create.execute([])?;
                }
                Ok(())
            },
        }
    }

    pub fn get_comments(&self) -> Result<Vec<Comment>, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                let mut stmt = conn.prepare("select id, name, content from comments")?;
                let mut rows = stmt.query([])?;
                let mut comments = Vec::new();
                while let Some(row) = rows.next()? {
                    comments.push(Comment {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        content: row.get(2)?,
                    });
                }
                actix_web::Result::Ok(comments)
            },
        }
    }

    pub fn post_comment(&self, data: PostComment) -> Result<Comment, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                let mut stmt = conn.prepare("insert into comments (name, content) values (?1, ?2)")?;
                let id = stmt.insert([data.name.clone(), data.content.clone()])?;
                Ok(Comment {
                    id,
                    name: data.name,
                    content: data.content,
                })
            }
        }
    }
}
