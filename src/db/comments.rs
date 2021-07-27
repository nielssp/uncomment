/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DB queries related to comments and threads

use sea_query::{Alias, DynIden, Expr, Func, Iden, Order, Query, SelectStatement, SimpleExpr, Value};
use sqlx::Row;

use std::{cmp, collections::HashMap, fmt};

use chrono::{DateTime, Utc};

use crate::db::{Page, Pool, DbError};

use super::{count_remaining, threads::Threads};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq)]
pub enum CommentStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Iden)]
pub enum Comments {
    Table,
    Id,
    ThreadId,
    ParentId,
    Level1Id,
    Level2Id,
    Level3Id,
    Level4Id,
    Level5Id,
    Level6Id,
    #[allow(dead_code)]
    UserId,
    Name,
    Email,
    Website,
    Ip,
    Html,
    Markdown,
    Status,
    Created,
}

fn convert_comment_status(value: &str) -> Result<CommentStatus, DbError> {
    match value {
        "Pending" => Ok(CommentStatus::Pending),
        "Approved" => Ok(CommentStatus::Approved),
        "Rejected" => Ok(CommentStatus::Rejected),
        _ => Err(DbError::ColumnTypeError),
    }
}

impl Into<Value> for CommentStatus {
    fn into(self) -> Value {
        self.to_string().into()
    }
}

impl fmt::Display for CommentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(serde::Serialize, Clone)]
pub struct PublicComment {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub website: String,
    pub html: String,
    pub created: String,
    pub created_timestamp: i64,
    pub approved: bool,
    pub replies: Vec<PublicComment>,
}

#[derive(Clone, Copy)]
pub struct CommentPosition {
    pub id: i32,
    pub thread_id: i32,
    pub level1_id: Option<i32>,
    pub level2_id: Option<i32>,
    pub level3_id: Option<i32>,
    pub level4_id: Option<i32>,
    pub level5_id: Option<i32>,
    pub level6_id: Option<i32>,
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
    pub created: DateTime<Utc>,
}

#[derive(serde::Serialize)]
pub struct PrivateComment {
    pub id: i32,
    pub thread_id: i32,
    pub thread_name: String,
    pub parent_id: Option<i32>,
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
    Parent(i32),
    Thread(i32),
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
    let mut rows = pool.select(Query::select()
        .column((Threads::Table, Threads::Name))
        .expr(Expr::count(Expr::tbl(Comments::Table, Comments::Id)))
        .from(Comments::Table)
        .inner_join(Threads::Table, Expr::tbl(Threads::Table, Threads::Id).equals(Comments::Table, Comments::ThreadId))
        .and_where(Expr::tbl(Threads::Table, Threads::Name).is_in(thread_names))
        .and_where(Expr::tbl(Comments::Table, Comments::Status).eq(CommentStatus::Approved))
        .group_by_col((Threads::Table, Threads::Name))).await?.into_iter();
    let mut result = HashMap::new();
    while let Some(row) = rows.next() {
        result.insert(row.try_get(0)?, row.try_get(1)?);
    }
    Ok(result)
}

fn build_comment_tree(comment: &mut PublicComment, replies: &HashMap<i32, Vec<PublicComment>>) {
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

fn get_comment_order(newest_first: bool, max_depth: u8) -> Vec<(String, Order)> {
    let mut order = Vec::new();
    if newest_first {
        if max_depth > 0 {
            order.push(("comments.level1_id".to_owned(), Order::Desc));
        }
        for i in 2..max_depth {
            order.push((format!("case when comments.level{}_id is null then 0 else 1 end", i), Order::Desc));
            order.push((format!("comments.level{}_id", i), Order::Desc));
        }
        order.push(("comments.id".to_owned(), Order::Desc));
    } else {
        if max_depth > 0 {
            order.push(("comments.level1_id".to_owned(), Order::Asc));
        }
        for i in 2..=max_depth {
            order.push((format!("comments.level{}_id", i), Order::Asc));
        }
        order.push(("comments.id".to_owned(), Order::Asc));
    }
    order
}

fn get_parent_id(
    id: i32,
    ids: [Option<i32>; 6],
    max_depth: u8,
) -> Option<i32> {
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
    let rows = pool.select(Query::select().from(Comments::Table)
        .columns(vec![
            (Comments::Table, Comments::Id),
            (Comments::Table, Comments::ParentId),
            (Comments::Table, Comments::Level1Id),
            (Comments::Table, Comments::Level2Id),
            (Comments::Table, Comments::Level3Id),
            (Comments::Table, Comments::Level4Id),
            (Comments::Table, Comments::Level5Id),
            (Comments::Table, Comments::Level6Id),
            (Comments::Table, Comments::Name),
            (Comments::Table, Comments::Website),
            (Comments::Table, Comments::Html),
            (Comments::Table, Comments::Created),
        ])
        .inner_join(Threads::Table, Expr::tbl(Threads::Table, Threads::Id).equals(Comments::Table, Comments::ThreadId))
        .and_where(Expr::tbl(Threads::Table, Threads::Name).eq(thread_name))
        .and_where(Expr::tbl(Comments::Table, Comments::Status).eq(CommentStatus::Approved))
        .order_by_customs(get_comment_order(newest_first, max_depth)))
        .await?;
    let mut root = Vec::new();
    let mut replies: HashMap<i32, Vec<PublicComment>> = HashMap::new();
    for row in rows {
        let created: DateTime<Utc> = row.try_get(11)?;
        let id: i32 = row.try_get(0)?;
        let level1_id = row.try_get(2)?;
        let level2_id = row.try_get(3)?;
        let level3_id = row.try_get(4)?;
        let level4_id = row.try_get(5)?;
        let level5_id = row.try_get(6)?;
        let level6_id = row.try_get(7)?;
        let parent_id = get_parent_id(id, [level1_id, level2_id, level3_id, level4_id, level5_id, level6_id], max_depth);
        let comment = PublicComment {
            id,
            parent_id,
            name: row.try_get(8)?,
            website: row.try_get(9)?,
            html: row.try_get(10)?,
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

pub async fn get_comment_position(pool: &Pool, id: i32) -> Result<Option<CommentPosition>, DbError> {
    let result = pool.select_optional(Query::select().from(Comments::Table)
        .columns(vec![
            Comments::Id,
            Comments::ThreadId,
            Comments::Level1Id,
            Comments::Level2Id,
            Comments::Level3Id,
            Comments::Level4Id,
            Comments::Level5Id,
            Comments::Level6Id,
            Comments::Status,
        ])
        .and_where(Expr::col(Comments::Id).eq(id)))
        .await?;
    if let Some(row) = result {
        Ok(Some(CommentPosition {
            id: row.try_get(0)?,
            thread_id: row.try_get(1)?,
            level1_id: row.try_get(2)?,
            level2_id: row.try_get(3)?,
            level3_id: row.try_get(4)?,
            level4_id: row.try_get(5)?,
            level5_id: row.try_get(6)?,
            level6_id: row.try_get(7)?,
            status: convert_comment_status(row.try_get(8)?)?,
        }))
    } else {
        Ok(None)
    }
}

pub async fn count_comments_by_ip(pool: &Pool, ip: &str, since: DateTime<Utc>) -> Result<i64, DbError> {
    let result = pool.select_one(Query::select().from(Comments::Table)
        .expr(Expr::col(Comments::Id).count())
        .and_where(Expr::col(Comments::Ip).eq(ip))
        .and_where(Expr::col(Comments::Created).gte(since.to_rfc3339())))
        .await?;
    Ok(result.try_get(0)?)
}

pub async fn insert_comment(
    pool: &Pool,
    thread_id: i32,
    parent: Option<&CommentPosition>,
    data: &NewComment,
) -> Result<CommentPosition, DbError> {
    let id = pool.insert(Query::insert().into_table(Comments::Table)
        .columns(vec![
            Comments::ThreadId,
            Comments::Name,
            Comments::Email,
            Comments::Website,
            Comments::Ip,
            Comments::Html,
            Comments::Markdown,
            Comments::Status,
            Comments::Created,
        ])
        .values_panic(vec![
            thread_id.into(),
            data.name.as_str().into(),
            data.email.as_str().into(),
            data.website.as_str().into(),
            data.ip.as_str().into(),
            data.html.as_str().into(),
            data.markdown.as_str().into(),
            data.status.into(),
            data.created.to_rfc3339().into(),
        ])
        .returning_col(Comments::Id)).await?;
    let parent_id = parent.map(|p| p.level6_id.unwrap_or(p.id));
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
    pool.update(Query::update().table(Comments::Table)
        .value(Comments::ParentId, parent_id.into())
        .value(Comments::Level1Id, level1.into())
        .value(Comments::Level2Id, level2.into())
        .value(Comments::Level3Id, level3.into())
        .value(Comments::Level4Id, level4.into())
        .value(Comments::Level5Id, level5.into())
        .value(Comments::Level6Id, level6.into())
        .and_where(Expr::col(Comments::Id).eq(id)))
        .await?;
    Ok(CommentPosition {
        id,
        thread_id,
        level1_id: Some(level1),
        level2_id: level2,
        level3_id: level3,
        level4_id: level4,
        level5_id: level5,
        level6_id: level6,
        status: data.status,
    })
}

pub async fn post_comment(
    pool: &Pool,
    thread_id: i32,
    parent: Option<&CommentPosition>,
    max_depth: u8,
    data: NewComment,
) -> Result<PublicComment, DbError> {
    let position = insert_comment(pool, thread_id, parent, &data).await?;
    let visible_parent_id = get_parent_id(position.id, [
        position.level1_id, position.level2_id, position.level3_id, position.level4_id, position.level5_id,
        position.level6_id], max_depth);
    Ok(PublicComment {
        id: position.id,
        parent_id: visible_parent_id,
        name: data.name,
        website: data.website,
        html: data.html,
        created: data.created.to_rfc3339(),
        created_timestamp: data.created.timestamp(),
        approved: data.status == CommentStatus::Approved,
        replies: vec![],
    })
}

fn get_default_comment_query() -> SelectStatement {
    let nested: DynIden = sea_query::SeaRc::new(Alias::new("c"));
    Query::select().from(Comments::Table)
        .columns(vec![
            (Comments::Table, Comments::Id),
            (Comments::Table, Comments::ThreadId),
        ])
        .column((Threads::Table, Threads::Name))
        .columns(vec![
            (Comments::Table, Comments::ParentId),
            (Comments::Table, Comments::Status),
            (Comments::Table, Comments::Name),
            (Comments::Table, Comments::Email),
            (Comments::Table, Comments::Website),
            (Comments::Table, Comments::Ip),
            (Comments::Table, Comments::Markdown),
            (Comments::Table, Comments::Html),
            (Comments::Table, Comments::Created),
        ])
        .expr(SimpleExpr::SubQuery(Box::new(Query::select()
                    .expr(Func::count(Expr::col(Comments::Id)))
                    .from_as(Comments::Table, nested.clone())
                    .and_where(Expr::tbl(Comments::Table, Comments::Id)
                        .equals(nested.clone(), Comments::ParentId))
                    .to_owned())))
        .inner_join(Threads::Table, Expr::tbl(Threads::Table, Threads::Id).equals(Comments::Table, Comments::ThreadId))
        .to_owned()
}

async fn query_comments(
    pool: &Pool,
    select: &SelectStatement,
) -> Result<Vec<PrivateComment>, DbError> {
    let mut rows = pool.select(select).await?.into_iter();
    let mut content = Vec::new();
    while let Some(row) = rows.next() {
        let created: DateTime<Utc> = row.try_get(11)?;
        content.push(PrivateComment {
            id: row.try_get(0)?,
            thread_id: row.try_get(1)?,
            thread_name: row.try_get(2)?,
            parent_id: row.try_get(3)?,
            status: convert_comment_status(row.try_get(4)?)?,
            name: row.try_get(5)?,
            email: row.try_get(6)?,
            website: row.try_get(7)?,
            ip: row.try_get(8)?,
            markdown: row.try_get(9)?,
            html: row.try_get(10)?,
            created: created.to_rfc3339(),
            created_timestamp: created.timestamp(),
            replies: row.try_get(12)?,
        });
    }
    Ok(content)
}

pub async fn get_comments(pool: &Pool, filter: CommentFilter, asc: bool, limit: usize, offset: usize) -> Result<Page<PrivateComment>, DbError> {
    let mut query = get_default_comment_query();
    match filter {
        CommentFilter::Status(status) => {
            query.and_where(Expr::tbl(Comments::Table, Comments::Status).eq(status));
        },
        CommentFilter::Parent(parent_id) => {
            query.and_where(Expr::tbl(Comments::Table, Comments::ParentId).eq(parent_id));
        },
        CommentFilter::Thread(thread_id) => {
            query.and_where(Expr::tbl(Comments::Table, Comments::ThreadId).eq(thread_id));
        },
    };
    if asc {
        query.order_by((Comments::Table, Comments::Created), sea_query::Order::Asc);
    } else {
        query.order_by((Comments::Table, Comments::Created), sea_query::Order::Desc);
    };
    query.limit(limit as u64).offset(offset as u64);
    let content = query_comments(pool, &query).await?;
    let remaining = count_remaining(pool, content.len(), limit, offset, Query::select()
        .from(Comments::Table)
        .expr(Expr::col(Comments::Id).count())
        .and_where(match filter {
            CommentFilter::Status(status) => Expr::col(Comments::Status).eq(status),
            CommentFilter::Parent(parent_id) => Expr::col(Comments::ParentId).eq(parent_id),
            CommentFilter::Thread(thread_id) => Expr::col(Comments::ThreadId).eq(thread_id),
        })).await?;
    Ok(Page { content, remaining, limit })
}

pub async fn get_comment(pool: &Pool, id: i32) -> Result<Option<PrivateComment>, DbError> {
    Ok(query_comments(pool, get_default_comment_query()
            .and_where(Expr::tbl(Comments::Table, Comments::Id).eq(id))).await?.into_iter().next())
}

pub async fn update_comment(pool: &Pool, id: i32, data: UpdateComment) -> Result<(), DbError> {
    pool.update(Query::update().table(Comments::Table)
        .value(Comments::Name, data.name.into())
        .value(Comments::Website, data.website.into())
        .value(Comments::Email, data.email.into())
        .value(Comments::Markdown, data.markdown.into())
        .value(Comments::Html, data.html.into())
        .value(Comments::Status, data.status.into())
        .and_where(Expr::col(Comments::Id).eq(id))).await?;
    Ok(())
}

pub async fn delete_comment(pool: &Pool, id: i32) -> Result<(), DbError> {
    pool.delete(Query::delete().from_table(Comments::Table)
        .and_where(Expr::col(Comments::Id).eq(id))).await?;
    Ok(())
}

