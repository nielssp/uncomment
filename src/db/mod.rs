/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment database abstraction

use std::collections::HashSet;

use log::info;
use sea_query::{DeleteStatement, InsertStatement, SelectStatement, UpdateStatement};
use sqlx::Row;
use crate::settings::Settings;
use thiserror::Error;

#[cfg(not(feature = "postgres"))]
use sea_query::SqliteQueryBuilder;

#[cfg(not(feature = "postgres"))]
sea_query::sea_query_driver_sqlite!();

#[cfg(feature = "postgres")]
use sea_query::PostgresQueryBuilder;

#[cfg(feature = "postgres")]
sea_query::sea_query_driver_postgres!();

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
    #[cfg(not(feature = "postgres"))]
    Pool(sqlx::SqlitePool),
    #[cfg(feature = "postgres")]
    Pool(sqlx::PgPool),
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
    #[cfg(not(feature = "postgres"))]
    pub async fn connect(connection_string: &str) -> Result<Pool, DbError> {
        if connection_string.starts_with("sqlite:") {
            async_std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(std::path::Path::new(connection_string.strip_prefix("sqlite:").unwrap()))
                .await?;
            Ok(Pool::Pool(sqlx::SqlitePool::connect(connection_string).await?))
        } else {
            Err(DbError::UnsupportedConnectionString)
        }
    }

    #[cfg(feature = "postgres")]
    pub async fn connect(connection_string: &str) -> Result<Pool, DbError> {
        if connection_string.starts_with("postgresql:") {
            Ok(Pool::Pool(sqlx::PgPool::connect(connection_string).await?))
        } else {
            Err(DbError::UnsupportedConnectionString)
        }
    }

    #[cfg(not(feature = "postgres"))]
    pub async fn select(&self, query: &SelectStatement) -> Result<Vec<sqlx::sqlite::SqliteRow>, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(SqliteQueryBuilder);
        Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).fetch_all(pool).await?)
    }

    #[cfg(not(feature = "postgres"))]
    pub async fn select_one(&self, query: &SelectStatement) -> Result<sqlx::sqlite::SqliteRow, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(SqliteQueryBuilder);
        Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).fetch_one(pool).await?)
    }

    #[cfg(not(feature = "postgres"))]
    pub async fn select_optional(&self, query: &SelectStatement) -> Result<Option<sqlx::sqlite::SqliteRow>, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(SqliteQueryBuilder);
        Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).fetch_optional(pool).await?)
    }

    #[cfg(not(feature = "postgres"))]
    pub async fn insert(&self, query: &InsertStatement) -> Result<(), DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(SqliteQueryBuilder);
        sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).execute(pool).await?;
        Ok(())
    }

    #[cfg(not(feature = "postgres"))]
    pub async fn insert_returning(&self, query: &InsertStatement) -> Result<i32, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(SqliteQueryBuilder);
        Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).execute(pool).await?.last_insert_rowid() as i32)
    }

    #[cfg(not(feature = "postgres"))]
    pub async fn update(&self, query: &UpdateStatement) -> Result<u64, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(SqliteQueryBuilder);
        Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).execute(pool).await?.rows_affected())
    }

    #[cfg(not(feature = "postgres"))]
    pub async fn delete(&self, query: &DeleteStatement) -> Result<u64, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(SqliteQueryBuilder);
        Ok(sea_query_driver_sqlite::bind_query(sqlx::query(&sql), &values).execute(pool).await?.rows_affected())
    }

    #[cfg(feature = "postgres")]
    pub async fn select(&self, query: &SelectStatement) -> Result<Vec<sqlx::postgres::PgRow>, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(PostgresQueryBuilder);
        Ok(sea_query_driver_postgres::bind_query(sqlx::query(&sql), &values).fetch_all(pool).await?)
    }

    #[cfg(feature = "postgres")]
    pub async fn select_one(&self, query: &SelectStatement) -> Result<sqlx::postgres::PgRow, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(PostgresQueryBuilder);
        Ok(sea_query_driver_postgres::bind_query(sqlx::query(&sql), &values).fetch_one(pool).await?)
    }

    #[cfg(feature = "postgres")]
    pub async fn select_optional(&self, query: &SelectStatement) -> Result<Option<sqlx::postgres::PgRow>, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(PostgresQueryBuilder);
        Ok(sea_query_driver_postgres::bind_query(sqlx::query(&sql), &values).fetch_optional(pool).await?)
    }

    #[cfg(feature = "postgres")]
    pub async fn insert(&self, query: &InsertStatement) -> Result<(), DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(PostgresQueryBuilder);
        sea_query_driver_postgres::bind_query(sqlx::query(&sql), &values).execute(pool).await?;
        Ok(())
    }

    #[cfg(feature = "postgres")]
    pub async fn insert_returning(&self, query: &InsertStatement) -> Result<i32, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(PostgresQueryBuilder);
        let row = sea_query_driver_postgres::bind_query(sqlx::query(&sql), &values).fetch_one(pool).await?;
        Ok(row.get(0))
    }

    #[cfg(feature = "postgres")]
    pub async fn update(&self, query: &UpdateStatement) -> Result<u64, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(PostgresQueryBuilder);
        Ok(sea_query_driver_postgres::bind_query(sqlx::query(&sql), &values).execute(pool).await?.rows_affected())
    }

    #[cfg(feature = "postgres")]
    pub async fn delete(&self, query: &DeleteStatement) -> Result<u64, DbError> {
        let Pool::Pool(pool) = self;
        let (sql, values) = query.build(PostgresQueryBuilder);
        Ok(sea_query_driver_postgres::bind_query(sqlx::query(&sql), &values).execute(pool).await?.rows_affected())
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
            .unwrap_or(0 as i64);
        remaining = (size as usize) - offset - limit;
    }
    Ok(remaining)
}

#[cfg(not(feature = "postgres"))]
pub async fn install(settings: &Settings) -> Result<Pool, DbError> {
    let Pool::Pool(pool) = Pool::connect(&settings.database).await?;
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
    for (name, migration) in migrations::MIGRATIONS {
        if versions.contains(name.to_owned()) {
            continue;
        }
        info!("Running migration: {}", name);
        let mut tx = pool.begin().await?;
        for statement in migration(&SqliteQueryBuilder) {
            sqlx::query(&statement).execute(&mut tx).await?;
        }
        sqlx::query("insert into versions values (?)").bind(name).execute(&mut tx).await?;
        tx.commit().await?;
    }
    Ok(Pool::Pool(pool))
}

#[cfg(feature = "postgres")]
pub async fn install(settings: &Settings) -> Result<Pool, DbError> {
    let Pool::Pool(pool) = Pool::connect(&settings.database).await?;
    let row = sqlx::query("SELECT 1 FROM pg_catalog.pg_tables WHERE schemaname = 'public' AND tablename = 'versions'")
        .fetch_optional(&pool).await?;
    let mut versions: HashSet<String> = HashSet::new();
    if row.is_none() {
        info!("Installing new PostgreSQL database...");
        sqlx::query("create table versions (version varchar(100) not null)").execute(&pool).await?;
    } else {
        let rows = sqlx::query("select version from versions").fetch_all(&pool).await?;
        for row in rows {
            versions.insert(row.try_get("version")?);
        }
    }
    for (name, migration) in migrations::MIGRATIONS {
        if versions.contains(name.to_owned()) {
            continue;
        }
        info!("Running migration: {}", name);
        let mut tx = pool.begin().await?;
        for statement in migration(&PostgresQueryBuilder) {
            sqlx::query(&statement).execute(&mut tx).await?;
        }
        sqlx::query("insert into versions values ($1)").bind(name).execute(&mut tx).await?;
        tx.commit().await?;
    }
    Ok(Pool::Pool(pool))
}
