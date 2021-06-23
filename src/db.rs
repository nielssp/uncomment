use std::{collections::{HashMap, HashSet}, fmt};

use chrono::{DateTime, FixedOffset, Local};
use log::info;
use rusqlite::{OptionalExtension, named_params, params};
use serde::Serialize;
use thiserror::Error;

use crate::migrations::SQLITE_MIGRATIONS;

#[derive(Serialize)]
pub struct Thread {
    pub id: i64,
    pub name: String,
    pub title: Option<String>,
}

#[derive(Debug)]
pub enum CommentStatus {
    Pending,
    Approved,
    Rejected,
}

impl fmt::Display for CommentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
    pub level1_id: Option<i64>,
    pub level2_id: Option<i64>,
    pub level3_id: Option<i64>,
    pub level4_id: Option<i64>,
    pub level5_id: Option<i64>,
    pub level6_id: Option<i64>,
}

pub struct NewComment {
    pub name: String,
    pub email: String,
    pub website: String,
    pub html: String,
    pub markdown: String,
}

pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: String,
    pub website: String,
    pub trusted: bool,
    pub admin: bool,
}

pub struct Session {
    pub id: String,
    pub valid_until: DateTime<FixedOffset>,
    pub user: User,
}

pub struct NewUser {
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: String,
    pub website: String,
    pub trusted: bool,
    pub admin: bool,
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

    fn get_comment_order(newest_first: bool) -> &'static str {
        if newest_first {
            "c.level1_id desc nulls first, \
            c.level2_id desc nulls first, \
            c.level3_id desc nulls first, \
            c.level4_id desc nulls first, \
            c.level5_id desc nulls first, \
            c.level6_id desc nulls first, \
            c.id desc"
        } else {
            "c.level1_id asc nulls first, \
            c.level2_id asc nulls first, \
            c.level3_id asc nulls first, \
            c.level4_id asc nulls first, \
            c.level5_id asc nulls first, \
            c.level6_id asc nulls first, \
            c.id asc"
        }
    }

    pub fn get_comments(&self, thread_name: &str, newest_first: bool) -> Result<Vec<Comment>, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                let mut stmt = conn.prepare(
                    &format!("select c.id, c.parent_id, c.name, c.website, c.html, c.created \
                    from comments c \
                    inner join threads t on t.id = c.thread_id \
                    where t.name = ? \
                    and c.status = 'Approved'
                    order by {}", Self::get_comment_order(newest_first)))?;
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
                conn.query_row("select c.id, c.thread_id, c.level1_id, c.level2_id, c.level3_id, c.level4_id, \
                    c.level5_id, c.level6_id \
                    from comments c \
                    where c.id = ?", [id], |row| {
                    Ok(CommentPosition {
                        id: row.get(0)?,
                        thread_id: row.get(1)?,
                        level1_id: row.get(2)?,
                        level2_id: row.get(3)?,
                        level3_id: row.get(4)?,
                        level4_id: row.get(5)?,
                        level5_id: row.get(6)?,
                        level6_id: row.get(7)?,
                    })
                }).optional().map_err(|e| e.into())
            },
        }
    }

    pub fn get_thread(&self, thread_name: &str) -> Result<Option<Thread>, RepoError> {
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

    pub fn create_thread(&self, thread_name: &str) -> Result<Thread, RepoError> {
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
                    name: thread_name.to_owned(),
                    title: None,
                })
            }
        }
    }

    pub fn post_comment(
        &self,
        thread_id: i64,
        parent: Option<&CommentPosition>,
        data: NewComment
    ) -> Result<Comment, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let now = Local::now();
                let conn = pool.get()?;
                let parent_id = parent.map(|p| p.level6_id.unwrap_or(p.id));
                let mut insert = conn.prepare(
                    "insert into comments (thread_id, parent_id, name, email, website, html, markdown, status, created) \
                    values (:thread_id, :parent_id, :name, :email, :website, :html, :markdown, :status, :created)")?;
                let id = insert.insert(named_params! {
                    ":thread_id": thread_id,
                    ":parent_id": parent_id,
                    ":name": data.name,
                    ":email": &data.email,
                    ":website": &data.website,
                    ":html": &data.html,
                    ":markdown": &data.markdown,
                    ":status": CommentStatus::Approved.to_string(),
                    ":created": now.to_rfc3339(),
                })?;
                let level1 = parent.map(|p| p.level1_id).flatten().unwrap_or(id);
                let level2 = parent.map(|p| p.level2_id
                    .or_else(|| if p.level1_id.is_some() { Some(id) } else { None })).flatten();
                let level3 = parent.map(|p| p.level3_id
                    .or_else(|| if p.level2_id.is_some() { Some(id) } else { None })).flatten();
                let level4 = parent.map(|p| p.level4_id
                    .or_else(|| if p.level3_id.is_some() { Some(id) } else { None })).flatten();
                let level5 = parent.map(|p| p.level5_id
                    .or_else(|| if p.level4_id.is_some() { Some(id) } else { None })).flatten();
                let level6 = parent.map(|p| p.level6_id
                    .or_else(|| if p.level5_id.is_some() { Some(id) } else { None })).flatten();
                conn.execute("update comments set level1_id = ?, level2_id = ?, level3_id = ?, \
                    level4_id = ?, level5_id = ?, level6_id = ? where id = ?",
                    params![level1, level2, level3, level4, level5, level6, id])?;
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

    pub fn get_session(&self, session_id: &str) -> Result<Option<Session>, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                let mut select = conn.prepare(
                    "select s.id, s.valid_until, u.id, u.username, u.password, u.name, u.email, \
                    u.website, u.trusted, u.admin from sessions s inner join users u on u.id = s.user_id \
                    where s.id = ?")?;
                let mut rows = select.query([session_id])?;
                if let Some(row) = rows.next()? {
                    let valid_until: String = row.get(1)?;
                    Ok(Some(Session {
                        id: row.get(0)?,
                        valid_until: DateTime::parse_from_rfc3339(valid_until.as_str())?,
                        user: User {
                            id: row.get(2)?,
                            username: row.get(3)?,
                            password: row.get(4)?,
                            name: row.get(5)?,
                            email: row.get(6)?,
                            website: row.get(7)?,
                            trusted: row.get(8)?,
                            admin: row.get(9)?,
                        },
                    }))
                } else {
                    Ok(None)
                }
            },
        }
    }

    pub fn get_user(&self, username: &str) -> Result<Option<User>, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                let mut select = conn.prepare(
                    "select u.id, u.username, u.password, u.name, u.email, \
                    u.website, u.trusted, u.admin from users u \
                    where u.username = ?")?;
                let mut rows = select.query([username])?;
                if let Some(row) = rows.next()? {
                    Ok(Some(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password: row.get(2)?,
                        name: row.get(3)?,
                        email: row.get(4)?,
                        website: row.get(5)?,
                        trusted: row.get(6)?,
                        admin: row.get(7)?,
                    }))
                } else {
                    Ok(None)
                }
            },
        }
    }

    pub fn admin_exists(&self) -> Result<bool, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                Ok(conn.query_row("select 1 from users where admin = 1", [], |_| Ok(true))
                    .optional().map(|o| o.is_some())?)
            },
        }
    }

    pub fn create_user(&self, new_user: NewUser) -> Result<User, RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                let mut insert = conn.prepare(
                    "insert into users (username, password, name, email, website, trusted, admin) \
                    values (:username, :password, :name, :email, :website, :trusted, :admin)")?;
                let id = insert.insert(named_params! {
                    ":username": new_user.username,
                    ":password": new_user.password,
                    ":name": new_user.name,
                    ":email": new_user.email,
                    ":website": new_user.website,
                    ":trusted": new_user.trusted,
                    ":admin": new_user.admin,
                })?;
                Ok(User {
                    id,
                    username: new_user.username,
                    password: new_user.password,
                    name: new_user.name,
                    email: new_user.email,
                    website: new_user.website,
                    trusted: new_user.trusted,
                    admin: new_user.admin,
                })
            },
        }
    }

    pub fn create_session(&self, session_id: &str, valid_until: DateTime<Local>, user_id: i64) -> Result<(), RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                conn.execute("insert into sessions (id, valid_until, user_id) VALUES (:id, :valid_until, :user_id)",
                    named_params! {
                        ":id": session_id,
                        ":valid_until": valid_until.to_rfc3339(),
                        ":user_id": user_id,
                    })?;
                Ok(())
            },
        }
    }

    pub fn delete_session(&self, session_id: &str) -> Result<(), RepoError> {
        match self {
            Repo::SqliteRepo(pool) => {
                let conn = pool.get()?;
                conn.execute("delete from sessions where id = ?", [session_id])?;
                Ok(())
            },
        }
    }
}
