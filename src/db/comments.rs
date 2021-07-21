/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DB queries related to comments and threads

use std::{cmp, collections::HashMap, convert::{TryFrom, TryInto}, fmt};

use chrono::{DateTime, Utc};
use quaint::{pooled::PooledConnection, prelude::*};

use crate::db::{Page, Pool, DbError, insert_id};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq)]
pub enum CommentStatus {
    Pending,
    Approved,
    Rejected,
}

impl<'a> TryFrom<ParameterizedValue<'a>> for CommentStatus {
    type Error = DbError;

    fn try_from(value: ParameterizedValue) -> Result<Self, Self::Error> {
        let s = value.as_str().ok_or(DbError::ColumnTypeError)?;
        match s {
            "Pending" => Ok(CommentStatus::Pending),
            "Approved" => Ok(CommentStatus::Approved),
            "Rejected" => Ok(CommentStatus::Rejected),
            _ => Err(DbError::ColumnTypeError),
        }
    }
}

impl fmt::Display for CommentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(serde::Serialize, Clone)]
pub struct PublicComment {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub website: String,
    pub html: String,
    pub created: String,
    pub created_timestamp: i64,
    pub approved: bool,
    pub replies: Vec<PublicComment>,
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
    pub status: CommentStatus,
}

pub struct NewComment {
    pub name: String,
    pub email: String,
    pub website: String,
    pub ip: String,
    pub html: String,
    pub markdown: String,
    pub status: CommentStatus,
}

#[derive(serde::Serialize)]
pub struct PrivateComment {
    pub id: i64,
    pub thread_id: i64,
    pub thread_name: String,
    pub parent_id: Option<i64>,
    pub status: CommentStatus,
    pub name: String,
    pub email: String,
    pub website: String,
    pub ip: String,
    pub markdown: String,
    pub html: String,
    pub created: String,
    pub created_timestamp: i64,
    pub replies: i64,
}

pub enum CommentFilter {
    Status(CommentStatus),
    Parent(i64),
    Thread(i64),
}

pub struct UpdateComment {
    pub name: String,
    pub email: String,
    pub website: String,
    pub html: String,
    pub markdown: String,
    pub status: CommentStatus,
}

pub async fn count_comments_by_thread(pool: &Pool, thread_names: Vec<&str>) -> Result<HashMap<String, i64>, DbError> {
    let conn = pool.check_out().await?;
    let mut rows = conn.select(Select::from_table("comments".alias("c"))
        .value(Column::from(("t", "name")))
        .value(count(asterisk()))
        .inner_join("threads".alias("t").on(("t", "id").equals(Column::from(("c", "thread_id")))))
        .so_that(("t", "name").in_selection(thread_names))
        .and_where(("c", "status").equals("Approved"))
        .group_by(Column::from(("t", "name")))).await?.into_iter();
    let mut result = HashMap::new();
    while let Some(row) = rows.next() {
        result.insert(row[0].clone().try_into()?, row[1].clone().try_into()?);
    }
    Ok(result)
}

fn build_comment_tree(comment: &mut PublicComment, replies: &HashMap<i64, Vec<PublicComment>>) {
    match replies.get(&comment.id) {
        Some(comment_replies) => {
            for reply in comment_replies {
                let mut clone = reply.clone();
                build_comment_tree(&mut clone, replies);
                comment.replies.push(clone);
            }
        },
        None => {
        },
    }
}

fn get_comment_order(newest_first: bool, max_depth: u8) -> String {
    let mut order: String = "".to_owned();
    if newest_first {
        if max_depth > 0 {
            order.push_str("c.level1_id desc,");
        }
        for i in 1..max_depth {
            order.push_str(&format!("case when c.level{}_id is null then 0 else 1 end asc, c.level{}_id desc,",
                    i, i));
        }
        order.push_str("c.id desc");
    } else {
        if max_depth > 0 {
            order.push_str("c.level1_id asc,");
        }
        for i in 1..=max_depth {
            order.push_str(&format!("c.level{}_id asc,", i));
        }
        order.push_str("c.id asc");
    }
    order
}

fn get_parent_id(
    id: i64,
    ids: [Option<i64>; 6],
    max_depth: u8,
) -> Option<i64> {
    for i in (1..=max_depth).rev() {
        if i == 6 {
            if let Some(parent_id) = ids[5] {
                if parent_id != id {
                    return Some(parent_id);
                }
            }
        } else if ids[i as usize].is_some() {
            return ids[(i - 1) as usize];
        }
    }
    None
}

pub async fn get_comment_thread(
    pool: &Pool,
    thread_name: &str,
    newest_first: bool,
    mut max_depth: u8,
) -> Result<Vec<PublicComment>, DbError> {
    max_depth = cmp::max(0, cmp::min(6, max_depth));
    let conn = pool.check_out().await?;
    let mut rows = conn.query_raw(
        &format!("select c.id, c.parent_id, c.level1_id, c.level2_id, c.level3_id, c.level4_id, c.level5_id, \
                    c.level6_id, c.name, c.website, c.html, c.created \
                    from comments c \
                    inner join threads t on t.id = c.thread_id \
                    where t.name = ? \
                    and c.status = 'Approved'
                    order by {}", get_comment_order(newest_first, max_depth)), &[ParameterizedValue::from(thread_name)])
        .await?
        .into_iter();
    let mut root = Vec::new();
    let mut replies: HashMap<i64, Vec<PublicComment>> = HashMap::new();
    while let Some(row) = rows.next() {
        let created_string: String = row[11].clone().try_into()?;
        let created = DateTime::parse_from_rfc3339(created_string.as_str())?;
        let id: i64 = row[0].clone().try_into()?;
        let level1_id = row[2].as_i64();
        let level2_id = row[3].as_i64();
        let level3_id = row[4].as_i64();
        let level4_id = row[5].as_i64();
        let level5_id = row[6].as_i64();
        let level6_id = row[7].as_i64();
        let parent_id = get_parent_id(id, [level1_id, level2_id, level3_id, level4_id, level5_id, level6_id], max_depth);
        let comment = PublicComment {
            id,
            parent_id,
            name: row[8].clone().try_into()?,
            website: row[9].clone().try_into()?,
            html: row[10].clone().try_into()?,
            created: created.to_rfc3339(),
            created_timestamp: created.timestamp(),
            approved: true,
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
        build_comment_tree(&mut comment, &replies);
        result.push(comment);
    }
    Ok(result)
}

pub async fn get_comment_position(pool: &Pool, id: i64) -> Result<Option<CommentPosition>, DbError> {
    let conn = pool.check_out().await?;
    let result = conn.select(Select::from_table("comments")
        .columns(vec!["id", "thread_id", "level1_id", "level2_id", "level3_id", "level4_id", "level5_id", "level6_id",
            "status"])
        .so_that("id".equals(id))).await?;
    if let Some(row) = result.first() {
        Ok(Some(CommentPosition {
            id: row[0].clone().try_into()?,
            thread_id: row[1].clone().try_into()?,
            level1_id: row[2].as_i64(),
            level2_id: row[3].as_i64(),
            level3_id: row[4].as_i64(),
            level4_id: row[5].as_i64(),
            level5_id: row[6].as_i64(),
            level6_id: row[7].as_i64(),
            status: row[8].clone().try_into()?,
        }))
    } else {
        Ok(None)
    }
}

pub async fn count_comments_by_ip(pool: &Pool, ip: &str, since: DateTime<Utc>) -> Result<i64, DbError> {
    let conn = pool.check_out().await?;
    let result = conn.select(Select::from_table("comments")
        .value(count(asterisk()))
        .so_that("ip".equals(ip).and("created".greater_than_or_equals(since.to_rfc3339())))).await?;
    Ok(result.first().and_then(|row| row[0].as_i64()).unwrap_or(0))
}

pub async fn post_comment(
    pool: &Pool,
    thread_id: i64,
    parent: Option<&CommentPosition>,
    max_depth: u8,
    data: NewComment,
) -> Result<PublicComment, DbError> {
    let now = Utc::now();
    let conn = pool.check_out().await?;
    let parent_id = parent.map(|p| p.level6_id.unwrap_or(p.id));
    let id = insert_id(conn.insert(Insert::single_into("comments")
        .value("thread_id", thread_id)
        .value("parent_id", parent_id.map(ParameterizedValue::from).unwrap_or(ParameterizedValue::Null))
        .value("name", data.name.as_str())
        .value("email", data.email)
        .value("website", data.website.as_str())
        .value("ip", data.ip)
        .value("html", data.html.as_str())
        .value("markdown", data.markdown)
        .value("status", data.status.to_string())
        .value("created", now.to_rfc3339())
        .build()).await?)?;
    let level1 = parent.map(|p| p.level1_id).flatten().unwrap_or(id);
    let level2 = parent.map(|p| p.level2_id
        .or_else(|| p.level1_id.map(|_| id))).flatten();
    let level3 = parent.map(|p| p.level3_id
        .or_else(|| p.level2_id.map(|_| id))).flatten();
    let level4 = parent.map(|p| p.level4_id
        .or_else(|| p.level3_id.map(|_| id))).flatten();
    let level5 = parent.map(|p| p.level5_id
        .or_else(|| p.level4_id.map(|_| id))).flatten();
    let level6 = parent.map(|p| p.level6_id
        .or_else(|| p.level5_id.map(|_| id))).flatten();
    let visible_parent_id = get_parent_id(id, [Some(level1), level2, level3, level4, level5, level6], max_depth);
    conn.update(Update::table("comments")
        .set("level1_id", level1)
        .set("level2_id", level2.map(ParameterizedValue::from).unwrap_or(ParameterizedValue::Null))
        .set("level3_id", level3.map(ParameterizedValue::from).unwrap_or(ParameterizedValue::Null))
        .set("level4_id", level4.map(ParameterizedValue::from).unwrap_or(ParameterizedValue::Null))
        .set("level5_id", level5.map(ParameterizedValue::from).unwrap_or(ParameterizedValue::Null))
        .set("level6_id", level6.map(ParameterizedValue::from).unwrap_or(ParameterizedValue::Null))
        .so_that("id".equals(id))).await?;
    Ok(PublicComment {
        id,
        parent_id: visible_parent_id,
        name: data.name,
        website: data.website,
        html: data.html,
        created: now.to_rfc3339(),
        created_timestamp: now.timestamp(),
        approved: data.status == CommentStatus::Approved,
        replies: vec![],
    })
}

fn get_default_comment_query<'a>() -> Select<'a> {
    Select::from_table("comments".alias("c"))
        .columns(vec![
            ("c", "id"), ("c", "thread_id"), ("t", "name"), ("c", "parent_id"), ("c", "status"),
            ("c", "name"), ("c", "email"), ("c", "website"), ("c", "ip"), ("c", "markdown"), ("c", "html"),
            ("c", "created")
        ])
        .value(Select::from_table("comments")
            .value(count(asterisk()))
            .so_that("parent_id".equals(Column::from(("c", "id")))))
        .inner_join("threads".alias("t").on(("t", "id").equals(Column::from(("c", "thread_id")))))
}

async fn query_comments<'a>(
    conn: &PooledConnection,
    select: Select<'a>,
) -> Result<Vec<PrivateComment>, DbError> {
    let mut rows = conn.select(select).await?.into_iter();
    let mut content = Vec::new();
    while let Some(row) = rows.next() {
        let created_string: String = row[11].clone().try_into()?;
        let created = DateTime::parse_from_rfc3339(created_string.as_str())?;
        content.push(PrivateComment {
            id: row[0].clone().try_into()?,
            thread_id: row[1].clone().try_into()?,
            thread_name: row[2].clone().try_into()?,
            parent_id: row[3].as_i64(),
            status: row[4].clone().try_into()?,
            name: row[5].clone().try_into()?,
            email: row[6].clone().try_into()?,
            website: row[7].clone().try_into()?,
            ip: row[8].clone().try_into()?,
            markdown: row[9].clone().try_into()?,
            html: row[10].clone().try_into()?,
            created: created.to_rfc3339(),
            created_timestamp: created.timestamp(),
            replies: row[12].clone().try_into()?,
        });
    }
    Ok(content)
}

pub async fn get_comments(pool: &Pool, filter: CommentFilter, asc: bool, limit: usize, offset: usize) -> Result<Page<PrivateComment>, DbError> {
    let conn = pool.check_out().await?;
    let mut query = get_default_comment_query();
    match filter {
        CommentFilter::Status(status) => {
            query = query.so_that(("c", "status").equals(status.to_string()));
        },
        CommentFilter::Parent(parent_id) => {
            query = query.so_that(("c", "parent_id").equals(parent_id));
        },
        CommentFilter::Thread(thread_id) => {
            query = query.so_that(("c", "thread_id").equals(thread_id));
        },
    };
    if asc {
        query = query.order_by("created".ascend());
    } else {
        query = query.order_by("created".descend());
    };
    query = query.limit(limit).offset(offset);
    let content = query_comments(&conn, query).await?;
    let mut remaining = 0;
    if content.len() == limit {
        let size = conn.select(Select::from_table("comments")
            .value(count(asterisk()))
            .so_that(match filter {
                CommentFilter::Status(status) => "status".equals(status.to_string()),
                CommentFilter::Parent(parent_id) => "parent_id".equals(parent_id),
                CommentFilter::Thread(thread_id) => "thread_id".equals(thread_id),
            })).await?
        .first()
            .map(|row| row[0].as_i64())
            .flatten()
            .unwrap_or(0) as usize;
        remaining = size - offset - limit;
    }
    Ok(Page { content, remaining, limit })
}

pub async fn get_comment(pool: &Pool, id: i64) -> Result<Option<PrivateComment>, DbError> {
    let conn = pool.check_out().await?;
    Ok(query_comments(&conn, get_default_comment_query().so_that(("c", "id").equals(id))).await?.into_iter().next())
}

pub async fn update_comment(pool: &Pool, id: i64, data: UpdateComment) -> Result<(), DbError> {
    let conn = pool.check_out().await?;
    conn.update(Update::table("comments")
        .set("name", data.name)
        .set("website", data.website)
        .set("email", data.email)
        .set("markdown", data.markdown)
        .set("html", data.html)
        .set("status", data.status.to_string())
        .so_that("id".equals(id))).await?;
    Ok(())
}

pub async fn delete_comment(pool: &Pool, id: i64) -> Result<(), DbError> {
    let conn = pool.check_out().await?;
    conn.delete(Delete::from_table("comments").so_that("id".equals(id))).await?;
    Ok(())
}

