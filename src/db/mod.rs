/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment database abstraction

use std::{collections::HashSet, path::Path};

use async_std::fs::OpenOptions;
use log::info;
use sea_query::{DeleteStatement, InsertStatement, SelectStatement, SqliteQueryBuilder, UpdateStatement};
use sqlx::{Row, sqlite::SqliteRow};
use crate::settings::Settings;
use thiserror::Error;

use migrations::SQLITE_MIGRATIONS;

sea_query::sea_query_driver_postgres!();
sea_query::sea_query_driver_sqlite!();

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

#[derive(Clone)]
pub enum Pool {
    Sqlite(sqlx::SqlitePool),
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQL error")]
    SqlxError(#[from] sqlx::error::Error),
    #[error("IO error")]
    IoError(#[from] async_std::io::Error),
    #[error("Date parsing error")]
    ChronoError(#[from] chrono::ParseError),
    #[error("Unsupported database connection string")]
    UnsupportedConnectionString,
    #[error("Invalid value in column")]
    ColumnTypeError,
}

impl Pool {
    pub async fn connect(connection_string: &str) -> Result<Pool, DbError> {
        if connection_string.starts_with("sqlite:") {
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(Path::new(connection_string.strip_prefix("sqlite:").unwrap()))
                .await?;
            Ok(Pool::Sqlite(sqlx::SqlitePool::connect(connection_string).await?))
        } else {
            Err(DbError::UnsupportedConnectionString)
        }
    }

    pub async fn select(&self, query: &SelectStatement) -> Result<Vec<SqliteRow>, DbError> {
        match self {
            Pool::Sqlite(pool) => {
                let (sql, values) = query.build(SqliteQueryBuilder);
                Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).fetch_all(pool).await?)
            },
        }
    }

    pub async fn select_one(&self, query: &SelectStatement) -> Result<SqliteRow, DbError> {
        match self {
            Pool::Sqlite(pool) => {
                let (sql, values) = query.build(SqliteQueryBuilder);
                Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).fetch_one(pool).await?)
            },
        }
    }

    pub async fn select_optional(&self, query: &SelectStatement) -> Result<Option<SqliteRow>, DbError> {
        match self {
            Pool::Sqlite(pool) => {
                let (sql, values) = query.build(SqliteQueryBuilder);
                Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).fetch_optional(pool).await?)
            },
        }
    }

    pub async fn insert(&self, query: &InsertStatement) -> Result<i32, DbError> {
        match self {
            Pool::Sqlite(pool) => {
                let (sql, values) = query.build(SqliteQueryBuilder);
                Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).execute(pool).await?.last_insert_rowid() as i32)
            },
        }
    }

    pub async fn update(&self, query: &UpdateStatement) -> Result<u64, DbError> {
        match self {
            Pool::Sqlite(pool) => {
                let (sql, values) = query.build(SqliteQueryBuilder);
                Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).execute(pool).await?.rows_affected())
            },
        }
    }

    pub async fn delete(&self, query: &DeleteStatement) -> Result<u64, DbError> {
        match self {
            Pool::Sqlite(pool) => {
                let (sql, values) = query.build(SqliteQueryBuilder);
                Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).execute(pool).await?.rows_affected())
            },
        }
    }
}

pub async fn count_remaining(
    pool: &Pool,
    length: usize,
    limit: usize,
    offset: usize,
    select: &SelectStatement,
) -> Result<usize, DbError> {
    let mut remaining = 0;
    if length == limit {
        let size = pool.select(select).await?
            .first()
            .map(|row| row.get(0))
            .flatten()
            .unwrap_or(0) as i64;
        remaining = (size as usize) - offset - limit;
    }
    Ok(remaining)
}

pub async fn install(settings: &Settings) -> Result<Pool, DbError> {
    let pool = Pool::connect(&settings.database).await?;
    match pool {
        Pool::Sqlite(pool) => {
            let row = sqlx::query("pragma table_info('versions')").fetch_optional(&pool).await?;
            let mut versions: HashSet<String> = HashSet::new();
            if row.is_none() {
                info!("Installing new SQLite3 database...");
                sqlx::query("create table versions (version text not null)").execute(&pool).await?;
            } else {
                let rows = sqlx::query("select version from versions").fetch_all(&pool).await?;
                for row in rows {
                    versions.insert(row.try_get("version")?);
                }
            }
            for (name, statements) in SQLITE_MIGRATIONS {
                if versions.contains(name.to_owned()) {
                    continue;
                }
                info!("Running migration: {}", name);
                let mut tx = pool.begin().await?;
                for statement in statements.iter() {
                    sqlx::query(statement).execute(&mut tx).await?;
                }
                sqlx::query("insert into versions values ($1)").bind(name).execute(&mut tx).await?;
                tx.commit().await?;
            }
            Ok(Pool::Sqlite(pool))
        },
    }
}
