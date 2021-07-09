/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DB queries related to threads

use std::convert::TryInto;

use quaint::{pooled::PooledConnection, prelude::*};

use crate::db::{Pool, DbError, insert_id};

use super::{Page, count_remaining};

#[derive(serde::Serialize)]
pub struct Thread {
    pub id: i64,
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

fn get_default_thread_query<'a>() -> Select<'a> {
    Select::from_table("threads".alias("t"))
        .columns(vec!("id", "name", "title"))
        .value(Select::from_table("comments")
            .value(count(asterisk()))
            .so_that("thread_id".equals(Column::from(("t", "id")))))
}

async fn query_threads<'a>(
    conn: &PooledConnection,
    select: Select<'a>,
) -> Result<Vec<Thread>, DbError> {
    let mut rows = conn.select(select).await?.into_iter();
    let mut content = Vec::new();
    while let Some(row) = rows.next() {
        content.push(Thread {
            id: row[0].clone().try_into()?,
            name: row[1].clone().try_into()?,
            title: row[2].to_string(),
            comments: row[3].clone().try_into()?,
        });
    }
    Ok(content)
}

pub async fn get_thread(pool: &Pool, thread_name: &str) -> Result<Option<Thread>, DbError> {
    let conn = pool.check_out().await?;
    Ok(query_threads(&conn, get_default_thread_query().so_that("name".equals(thread_name))).await?.into_iter().next())
}

pub async fn get_thread_by_id(pool: &Pool, id: i64) -> Result<Option<Thread>, DbError> {
    let conn = pool.check_out().await?;
    Ok(query_threads(&conn, get_default_thread_query().so_that("id".equals(id))).await?.into_iter().next())
}

pub async fn create_thread(pool: &Pool, data: NewThread) -> Result<Thread, DbError> {
    let conn = pool.check_out().await?;
    let id = insert_id(conn.insert(Insert::single_into("threads").value("name", data.name.as_str()).build()).await?)?;
    Ok(Thread {
        id: id as i64,
        name: data.name,
        title: data.title,
        comments: 0,
    })
}

pub async fn get_threads(pool: &Pool, limit: usize, offset: usize) -> Result<Page<Thread>, DbError> {
    let conn = pool.check_out().await?;
    let query = get_default_thread_query()
        .order_by("name")
        .limit(limit)
        .offset(offset);
    let content = query_threads(&conn, query).await?;
    let remaining = count_remaining(&conn, content.len(), limit, offset,
        Select::from_table("threads").value(count(asterisk()))).await?;
    Ok(Page { content, remaining, limit })
}

pub async fn update_thread(pool: &Pool, id: i64, data: UpdateThread) -> Result<(), DbError> {
    let conn = pool.check_out().await?;
    conn.update(Update::table("threads")
        .set("title", data.title.map(ParameterizedValue::from).unwrap_or(ParameterizedValue::Null))
        .so_that("id".equals(id))).await?;
    Ok(())
}

pub async fn delete_thread(pool: &Pool, id: i64) -> Result<(), DbError> {
    let conn = pool.check_out().await?;
    conn.delete(Delete::from_table("comments").so_that("thread_id".equals(id))).await?;
    conn.delete(Delete::from_table("threads").so_that("id".equals(id))).await?;
    Ok(())
}

