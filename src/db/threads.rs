/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DB queries related to threads

use std::convert::TryInto;

use quaint::prelude::*;

use crate::db::{Pool, DbError, insert_id};

#[derive(serde::Serialize)]
pub struct Thread {
    pub id: i64,
    pub name: String,
    pub title: Option<String>,
}

pub async fn get_thread(pool: &Pool, thread_name: &str) -> Result<Option<Thread>, DbError> {
    let conn = pool.check_out().await?;
    let result = conn.select(Select::from_table("threads")
        .columns(vec!("id", "name", "title"))
        .so_that("name".equals(thread_name))).await?;
    if let Some(row) = result.first() {
        Ok(Some(Thread {
            id: row[0].clone().try_into()?,
            name: row[1].clone().try_into()?,
            title: row[2].to_string(),
        }))
    } else {
        Ok(None)
    }
}

pub async fn create_thread(pool: &Pool, thread_name: &str) -> Result<Thread, DbError> {
    let conn = pool.check_out().await?;
    let id = insert_id(conn.insert(Insert::single_into("threads").value("name", thread_name).build()).await?)?;
    Ok(Thread {
        id: id as i64,
        name: thread_name.to_owned(),
        title: None,
    })
}
