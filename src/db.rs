use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Local};
use log::{debug, info};
use rusqlite::{OptionalExtension, named_params, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::migrations::SQLITE_MIGRATIONS;

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
    pub website: String,
    pub html: String,
    pub created: String,
    pub created_timestamp: i64,
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
    pub email: String,
    pub website: String,
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
    #[error("chrono error")]
    ChronoError(#[from] chrono::ParseError),
}

impl Repo {
    pub fn install(&self) -> Result<(), RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                let mut stmt = conn.prepare("pragma table_info('versions')")?;
                let mut rows = stmt.query([])?;
                let mut versions: HashSet<String> = HashSet::new();
                if rows.next()?.is_none() {
                    info!("Installing new SQLite3 database...");
                    conn.execute(
                        "create table versions (
                            version text not null
                        )", []
                    )?;
                } else {
                    let mut get_versions = conn.prepare("select version from versions")?;
                    versions = get_versions.query_map([], |row| row.get(0)).and_then(Iterator::collect)?;
                }
                for (name, statements) in SQLITE_MIGRATIONS {
                    if versions.contains(name.to_owned()) {
                        continue;
                    }
                    info!("Running migration: {}", name);
                    let mut conn = pool.get()?;
                    let tx = conn.transaction()?;
                    for statement in statements.iter() {
                        tx.execute(statement, [])?;
                    }
                    tx.execute(
                        "insert into versions values (?1)", [name]
                    )?;
                    tx.commit()?;
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
                    "select c.id, c.parent_id, c.name, c.website, c.html, c.created \
                    from comments c \
                    inner join threads t on t.id = c.thread_id
                    where t.name = ?
                    order by c.hierarchy asc")?;
                let mut rows = stmt.query([thread_name])?;
                let mut root = Vec::new();
                let mut replies: HashMap<i64, Vec<Comment>> = HashMap::new();
                while let Some(row) = rows.next()? {
                    let created_string: String = row.get(5)?;
                    let created = DateTime::parse_from_rfc3339(created_string.as_str())?;
                    let comment = Comment {
                        id: row.get(0)?,
                        parent_id: row.get(1)?,
                        name: row.get(2)?,
                        website: row.get(3)?,
                        html: row.get(4)?,
                        created: created.to_rfc3339(),
                        created_timestamp: created.timestamp(),
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
                    "insert into comments (thread_id, parent_id, hierarchy, name, email, website, html, markdown, status, created) \
                    values (:thread_id, :parent_id, :hierarchy, :name, :email, :website, :html, :markdown, 'approved', :created)")?;
                let id = insert.insert(named_params! {
                    ":thread_id": thread_id,
                    ":parent_id": parent_id,
                    ":hierarchy": hierarchy.unwrap_or("".to_owned()),
                    ":name": data.name.clone(),
                    ":email": data.email.clone(),
                    ":website": data.website.clone(),
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
                    website: data.website,
                    html: data.html,
                    created: now.to_rfc3339(),
                    created_timestamp: now.timestamp(),
                    replies: vec![],
                })
            }
        }
    }
}
