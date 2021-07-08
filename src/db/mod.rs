/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment database abstraction

use std::collections::HashSet;

use log::info;
use quaint::{pooled::{PooledConnection, Quaint}, prelude::*};
use crate::settings::Settings;
use thiserror::Error;

use migrations::SQLITE_MIGRATIONS;

pub mod comments;
pub mod threads;
pub mod users;
pub mod sessions;
pub mod migrations;

#[derive(serde::Serialize)]
pub struct Page<T> {
    pub content: Vec<T>,
    pub remaining: usize,
    pub limit: usize,
}

pub type Pool = Quaint;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("quaint error")]
    QuaintError(#[from] quaint::error::Error),
    #[error("chrono error")]
    ChronoError(#[from] chrono::ParseError),
    #[error("column type error")]
    ColumnTypeError,
}

pub fn insert_id(id_opt: Option<Id>) -> Result<i64, DbError> {
    match id_opt {
        Some(Id::Int(id)) => Ok(id as i64),
        _ => Err(DbError::ColumnTypeError),
    }
}

pub async fn count_remaining<'a>(
    conn: &PooledConnection,
    length: usize,
    limit: usize,
    offset: usize,
    select: Select<'a>,
) -> Result<usize, DbError> {
    let mut remaining = 0;
    if length == limit {
        let size = conn.select(select).await?
        .first()
            .map(|row| row[0].as_i64())
            .flatten()
            .unwrap_or(0) as usize;
        remaining = size - offset - limit;
    }
    Ok(remaining)
}

pub async fn install(settings: &Settings) -> Result<Pool, DbError> {
    let pool = Quaint::new(&format!("file:{}", settings.sqlite_database)).await?;
    let conn = pool.check_out().await?;
    match pool.connection_info() {
        ConnectionInfo::Sqlite { .. } => {
            let result = conn.query_raw("pragma table_info('versions')", &[]).await?;
            let mut versions: HashSet<String> = HashSet::new();
            if result.is_empty() {
                info!("Installing new SQLite3 database...");
                conn.execute_raw(
                    "create table versions (
                        version text not null
                    )", &[]
                ).await?;
            } else {
                versions = conn.select(Select::from_table("versions").column("version")).await?
                    .into_iter()
                    .map(|row| row[0].as_str().map(String::from))
                    .flatten()
                    .collect();
            }
            for (name, statements) in SQLITE_MIGRATIONS {
                if versions.contains(name.to_owned()) {
                    continue;
                }
                info!("Running migration: {}", name);
                let tx = conn.start_transaction().await?;
                for statement in statements.iter() {
                    tx.execute_raw(statement, &[]).await?;
                }
                tx.insert(Insert::single_into("versions").value("version", name.to_owned()).into()).await?;
                tx.commit().await?;
            }
        },
    }
    Ok(pool)
}
