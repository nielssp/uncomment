/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment database abstraction

use std::collections::HashSet;

use log::info;
use quaint::{pooled::{PooledConnection, Quaint}, prelude::*};
use rusqlite::params;
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
    #[error("sqlite error")]
    SqliteError(#[from] rusqlite::Error),
    #[error("date parsing error")]
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
    match pool.connection_info() {
        ConnectionInfo::Sqlite { file_path, .. } => {
            let conn = rusqlite::Connection::open(&file_path)?;
            let mut stmt = conn.prepare("pragma table_info('versions')")?;
            let mut rows = stmt.query(params![])?;
            let mut versions: HashSet<String> = HashSet::new();
            if rows.next()?.is_none() {
                info!("Installing new SQLite3 database...");
                conn.execute("create table versions (version text not null)", params![])?;
            } else {
                let mut get_versions = conn.prepare("select version from versions")?;
                versions = get_versions.query_map(params![], |row| row.get(0)).and_then(Iterator::collect)?;
            }
            for (name, statements) in SQLITE_MIGRATIONS {
                if versions.contains(name.to_owned()) {
                    continue;
                }
                info!("Running migration: {}", name);
                for statement in statements.iter() {
                    conn.execute(statement, params![])?;
                }
                conn.execute("insert into versions values (?1)", [name])?;
            }
        },
    }
    Ok(pool)
}
