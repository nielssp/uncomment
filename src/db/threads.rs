/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DB queries related to threads

use sea_query::{Expr, Func, Iden, Query, SelectStatement, SimpleExpr};
use sqlx::Row;

use crate::db::{DbError, Pool, comments::Comments};

use super::{Page, count_remaining};

#[derive(Iden)]
pub enum Threads {
    Table,
    Id,
    Name,
    Title,
}

#[derive(serde::Serialize)]
pub struct Thread {
    pub id: i32,
    pub name: String,
    pub title: Option<String>,
    pub comments: i64,
}

#[derive(serde::Deserialize)]
pub struct NewThread {
    pub name: String,
    pub title: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateThread {
    pub title: Option<String>,
}

fn get_default_thread_query() -> SelectStatement {
    Query::select()
        .columns(vec!(Threads::Id, Threads::Name, Threads::Title))
        .from(Threads::Table)
        .expr(SimpleExpr::SubQuery(Box::new(Query::select()
                    .expr(Func::count(Expr::col(Comments::Id)))
                    .from(Comments::Table)
                    .and_where(Expr::tbl(Threads::Table, Threads::Id)
                        .equals(Comments::Table, Comments::ThreadId))
                    .to_owned())))
        .to_owned()
}

async fn query_threads(
    pool: &Pool,
    select: &SelectStatement,
) -> Result<Vec<Thread>, DbError> {
    let mut rows = pool.select(select).await?.into_iter();
    let mut content = Vec::new();
    while let Some(row) = rows.next() {
        content.push(Thread {
            id: row.try_get(0)?,
            name: row.try_get(1)?,
            title: row.try_get(2)?,
            comments: row.try_get(3)?,
        });
    }
    Ok(content)
}

pub async fn get_thread_by_name(pool: &Pool, thread_name: &str) -> Result<Option<Thread>, DbError> {
    Ok(query_threads(pool, get_default_thread_query()
            .and_where(Expr::col(Threads::Name).eq(thread_name)))
        .await?.into_iter().next())
}

pub async fn get_thread_by_id(pool: &Pool, id: i32) -> Result<Option<Thread>, DbError> {
    Ok(query_threads(&pool, get_default_thread_query()
            .and_where(Expr::col(Threads::Id).eq(id)))
        .await?.into_iter().next())
}

pub async fn create_thread(pool: &Pool, data: NewThread) -> Result<Thread, DbError> {
    let id = pool.insert(Query::insert()
        .into_table(Threads::Table)
        .columns(vec![Threads::Name, Threads::Title])
        .values_panic(vec![data.name.as_str().into(), data.title.clone().into()])
        .returning_col(Threads::Id)).await?;
    Ok(Thread {
        id: id as i32,
        name: data.name,
        title: data.title,
        comments: 0,
    })
}

pub async fn get_threads(pool: &Pool, limit: usize, offset: usize) -> Result<Page<Thread>, DbError> {
    let mut query = get_default_thread_query();
    query.order_by(Threads::Name, sea_query::Order::Asc)
        .limit(limit as u64)
        .offset(offset as u64);
    let content = query_threads(pool, &query).await?;
    let remaining = count_remaining(pool, content.len(), limit, offset,
        Query::select().from(Threads::Table)
            .expr(Expr::count(Expr::col(Threads::Id)))).await?;
    Ok(Page { content, remaining, limit })
}

pub async fn update_thread(pool: &Pool, id: i32, data: UpdateThread) -> Result<(), DbError> {
    pool.update(Query::update()
        .table(Threads::Table)
        .value(Threads::Title, data.title.into())
        .and_where(Expr::col(Threads::Id).eq(id))).await?;
    Ok(())
}

pub async fn delete_thread(pool: &Pool, id: i32) -> Result<(), DbError> {
    pool.delete(Query::delete()
        .from_table(Comments::Table)
        .and_where(Expr::col(Comments::ThreadId).eq(id))).await?;
    pool.delete(Query::delete()
        .from_table(Threads::Table)
        .and_where(Expr::col(Threads::Id).eq(id))).await?;
    Ok(())
}

