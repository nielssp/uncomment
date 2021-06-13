use std::collections::HashMap;

use chrono::Local;
use log::{debug, info};
use rusqlite::{OptionalExtension, named_params, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize)]
pub struct Thread {
    pub id: i64,
    pub name: String,
    pub title: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Comment {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub html: String,
    pub created: String,
    pub replies: Vec<Comment>,
}

pub struct CommentPosition {
    pub id: i64,
    pub thread_id: i64,
    pub parent_id: Option<i64>,
    pub hierarchy: String,
}

pub struct NewComment {
    pub name: String,
    pub html: String,
    pub markdown: String,
}

#[derive(Serialize)]
pub struct Page<T> {
    pub content: Vec<T>,
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
                let mut stmt = conn.prepare("pragma table_info('versions')")?;
                let mut rows = stmt.query([])?;
                if rows.next()?.is_none() {
                    info!("Installing SQLite3 database...");
                    conn.execute(
                        "create table versions (
                            version text not null
                        )", []
                    )?;
                    conn.execute(
                        "create table threads (
                            id integer primary key autoincrement,
                            name text(100) unique not null,
                            title text(100) null
                        )", []
                    )?;
                    conn.execute(
                        "create table comments (
                            id integer primary key autoincrement,
                            thread_id integer not null,
                            parent_id integer null,
                            hierarchy text(100) not null,
                            name text(100) not null,
                            html text not null,
                            markdown text not null,
                            created text not null
                        )", []
                    )?;
                    conn.execute(
                        "insert into versions values (?1)", ["1"]
                    )?;
                }
                Ok(())
            },
        }
    }

    fn build_comment_tree(comment: &mut Comment, replies: &HashMap<i64, Vec<Comment>>) {
        match replies.get(&comment.id) {
            Some(comment_replies) => {
                for reply in comment_replies {
                    let mut clone = reply.clone();
                    Self::build_comment_tree(&mut clone, replies);
                    comment.replies.push(clone);
                }
            },
            None => {
            },
        }
    }

    pub fn get_comments(&self, thread_name: String) -> Result<Vec<Comment>, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                let mut stmt = conn.prepare(
                    "select c.id, c.parent_id, c.name, c.html, c.created \
                    from comments c \
                    inner join threads t on t.id = c.thread_id
                    where t.name = ?
                    order by c.hierarchy asc")?;
                let mut rows = stmt.query([thread_name])?;
                let mut root = Vec::new();
                let mut replies: HashMap<i64, Vec<Comment>> = HashMap::new();
                while let Some(row) = rows.next()? {
                    let comment = Comment {
                        id: row.get(0)?,
                        parent_id: row.get(1)?,
                        name: row.get(2)?,
                        html: row.get(3)?,
                        created: row.get(4)?,
                        replies: vec![],
                    };
                    match comment.parent_id {
                        Some(parent_id) => match replies.get_mut(&parent_id) {
                            Some(parent_replies) => parent_replies.push(comment),
                            None => {
                                let mut parent_replies = Vec::new();
                                parent_replies.push(comment);
                                replies.insert(parent_id, parent_replies);
                            }
                        },
                        None => root.push(comment),
                    }
                }
                let mut result = Vec::new();
                for mut comment in root {
                    Self::build_comment_tree(&mut comment, &replies);
                    result.push(comment);
                }
                Ok(result)
            },
        }
    }

    pub fn get_comment_position(&self, id: i64) -> Result<Option<CommentPosition>, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                conn.query_row("select id, thread_id, parent_id, hierarchy from comments where id = ?", [id], |row| {
                    Ok(CommentPosition {
                        id: row.get(0)?,
                        thread_id: row.get(1)?,
                        parent_id: row.get(2)?,
                        hierarchy: row.get(3)?,
                    })
                }).optional().map_err(|e| e.into())
            },
        }
    }

    pub fn get_thread(&self, thread_name: String) -> Result<Option<Thread>, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                conn.query_row("select id, name, title from threads where name = ?", [thread_name], |row| {
                    Ok(Thread {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        title: row.get(2)?,
                    })
                }).optional().map_err(|e| e.into())
            },
        }
    }

    pub fn create_thread(&self, thread_name: String) -> Result<Thread, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                let mut insert = conn.prepare("insert into threads (name) \
                    values (:name)")?;
                let id = insert.insert(named_params! {
                    ":name": thread_name.clone(),
                })?;
                Ok(Thread {
                    id,
                    name: thread_name,
                    title: None,
                })
            }
        }
    }

    pub fn post_comment(
        &self,
        thread_id: i64,
        parent: Option<CommentPosition>,
        data: NewComment
    ) -> Result<Comment, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let now = Local::now();
                let conn = pool.get()?;
                let parent_id = parent.as_ref().map(|p| p.id);
                let hierarchy = parent.as_ref().map(|p| format!("{}/{}", p.hierarchy, p.id));
                let mut insert = conn.prepare(
                    "insert into comments (thread_id, parent_id, hierarchy, name, html, markdown, created) \
                    values (:thread_id, :parent_id, :hierarchy, :name, :html, :markdown, :created)")?;
                let id = insert.insert(named_params! {
                    ":thread_id": thread_id,
                    ":parent_id": parent_id,
                    ":hierarchy": hierarchy.unwrap_or("".to_owned()),
                    ":name": data.name.clone(),
                    ":html": data.html.clone(),
                    ":markdown": data.markdown.clone(),
                    ":created": now.to_rfc3339(),
                })?;
                if parent.is_none() {
                    conn.execute("update comments set hierarchy = ? where id = ?", params![
                        format!("/{}", id),
                        id
                    ])?;
                }
                Ok(Comment {
                    id,
                    parent_id,
                    name: data.name,
                    html: data.html,
                    created: now.to_rfc3339(),
                    replies: vec![],
                })
            }
        }
    }
}
