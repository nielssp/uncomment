/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment database abstraction

use std::{collections::HashSet, path::Path};

use async_std::fs::OpenOptions;
use chrono::{TimeZone, Utc};
use log::info;
use sea_query::{DeleteStatement, InsertStatement, PostgresQueryBuilder, SelectStatement, SqliteQueryBuilder, UpdateStatement, Value, Values};
use sqlx::{AnyPool, Row, any::AnyRow};
use crate::settings::Settings;
use thiserror::Error;

use migrations::SQLITE_MIGRATIONS;

use self::migrations::POSTGRES_MIGRATIONS;

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

#[derive(Clone, Copy)]
pub enum PoolKind {
    Sqlite,
    Postgres,
}

#[derive(Clone)]
pub struct Pool {
    pub kind: PoolKind,
    pub pool: AnyPool,
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
    #[error("No insert id returned")]
    NoInsertId,
    #[error("Invalid value in column")]
    ColumnTypeError,
}

fn bind_query<'a>(
    query: sqlx::query::Query<'a, sqlx::Any, <sqlx::Any as sqlx::database::HasArguments<'a>>::Arguments>,
    params: &'a Values
) -> sqlx::query::Query<'a, sqlx::Any, <sqlx::Any as sqlx::database::HasArguments<'a>>::Arguments> {
    let mut query = query;
    info!("params: {:?}", params);
    for value in params.iter() {
        query = match value {
            Value::Null => query.bind(None::<bool>),
            Value::Bool(v) => query.bind(v),
            Value::TinyInt(v) => query.bind(*v as i64),
            Value::SmallInt(v) => query.bind(*v as i64),
            Value::Int(v) => query.bind(*v),
            Value::BigInt(v) => query.bind(*v),
            Value::TinyUnsigned(v) => query.bind(*v as i64),
            Value::SmallUnsigned(v) => query.bind(*v as i64),
            Value::Unsigned(v) => query.bind(*v as i64),
            Value::BigUnsigned(v) => query.bind(*v as i64),
            Value::Float(v) => query.bind(*v),
            Value::Double(v) => query.bind(*v),
            Value::String(v) => query.bind(v.as_str()),
            Value::Bytes(v) => query.bind(v.as_ref()),
            _ => {
                if value.is_json() {
                    query.bind(value.as_ref_json())
                } else if value.is_date_time() {
                    query.bind(Utc.from_utc_datetime(value.as_ref_date_time()))
                } else if value.is_decimal() {
                    query.bind(value.decimal_to_f64())
                } else if value.is_uuid() {
                    query.bind(value.as_ref_uuid())
                } else {
                    unimplemented!();
                }
            }
        };
    }
    query
}

impl Pool {
    pub async fn connect(connection_string: &str) -> Result<Pool, DbError> {
        if connection_string.starts_with("sqlite:") {
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(Path::new(connection_string.strip_prefix("sqlite:").unwrap()))
                .await?;
            Ok(Pool {
                kind: PoolKind::Sqlite,
                pool: sqlx::AnyPool::connect(connection_string).await?,
            })
        } else if connection_string.starts_with("postgresql:") {
            Ok(Pool {
                kind: PoolKind::Postgres,
                pool: sqlx::AnyPool::connect(connection_string).await?,
            })
        } else {
            Err(DbError::UnsupportedConnectionString)
        }
    }

    fn build_select_query(&self, query: &SelectStatement) -> (String, Values) {
        match self.kind {
            PoolKind::Sqlite => {
                query.build(SqliteQueryBuilder)
            },
            PoolKind::Postgres => {
                query.build(PostgresQueryBuilder)
            },
        }
    }

    fn build_insert_query(&self, query: &InsertStatement) -> (String, Values) {
        match self.kind {
            PoolKind::Sqlite => {
                query.build(SqliteQueryBuilder)
            },
            PoolKind::Postgres => {
                query.build(PostgresQueryBuilder)
            },
        }
    }

    fn build_update_query(&self, query: &UpdateStatement) -> (String, Values) {
        match self.kind {
            PoolKind::Sqlite => {
                query.build(SqliteQueryBuilder)
            },
            PoolKind::Postgres => {
                query.build(PostgresQueryBuilder)
            },
        }
    }

    fn build_delete_query(&self, query: &DeleteStatement) -> (String, Values) {
        match self.kind {
            PoolKind::Sqlite => {
                query.build(SqliteQueryBuilder)
            },
            PoolKind::Postgres => {
                query.build(PostgresQueryBuilder)
            },
        }
    }

    fn fix_broken_null_behavior(&self, (sql, Values(values)): (String, Values)) -> (String, Values) {
        if values.is_empty() {
            return (sql, Values(values));
        }
        let mut rest = sql.clone();
        let mut result = "".to_owned();
        let mut copy = Vec::new();
        let mut counter = 1;
        for value in values.iter() {
            let mut splits = rest.splitn(2, match self.kind {
                PoolKind::Sqlite => "?",
                PoolKind::Postgres => "$",
            }).into_iter();
            if let Some(start) = splits.next() {
                result.push_str(start);
            }
            match value {
                Value::Null => result.push_str("null"),
                _ => {
                    match self.kind {
                        PoolKind::Sqlite => result.push('?'),
                        PoolKind::Postgres => result.push_str(&format!("${}", counter)),
                    }
                    counter += 1;
                    copy.push(value.clone());
                },
            }
            if let Some(end) = splits.next() {
                rest = match self.kind {
                    PoolKind::Sqlite => end.to_owned(),
                    PoolKind::Postgres => end.to_owned().trim_start_matches(char::is_numeric).to_owned(),
                }
            } else {
                rest = "".to_owned();
            }
        }
        result.push_str(&rest);
        (result, Values(copy))
    }

    pub async fn select(&self, query: &SelectStatement) -> Result<Vec<AnyRow>, DbError> {
        let (sql, values) = self.fix_broken_null_behavior(self.build_select_query(query));
        let query = bind_query(sqlx::query(&sql), &values);
        Ok(query.fetch_all(&self.pool).await?)
    }

    pub async fn select_one(&self, query: &SelectStatement) -> Result<AnyRow, DbError> {
        let (sql, values) = self.fix_broken_null_behavior(self.build_select_query(query));
        let query = bind_query(sqlx::query(&sql), &values);
        Ok(query.fetch_one(&self.pool).await?)
    }

    pub async fn select_optional(&self, query: &SelectStatement) -> Result<Option<AnyRow>, DbError> {
        let (sql, values) = self.fix_broken_null_behavior(self.build_select_query(query));
        let query = bind_query(sqlx::query(&sql), &values);
        Ok(query.fetch_optional(&self.pool).await?)
    }

    pub async fn insert(&self, query: &InsertStatement) -> Result<i64, DbError> {
        let (sql, values) = self.fix_broken_null_behavior(self.build_insert_query(query));
        let query = bind_query(sqlx::query(&sql), &values);
        let result = query.execute(&self.pool).await?;
        result.last_insert_id().ok_or(DbError::NoInsertId)
    }

    pub async fn update(&self, query: &UpdateStatement) -> Result<u64, DbError> {
        let (sql, values) = self.fix_broken_null_behavior(self.build_update_query(query));
        let query = bind_query(sqlx::query(&sql), &values);
        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected())
    }

    pub async fn delete(&self, query: &DeleteStatement) -> Result<u64, DbError> {
        let (sql, values) = self.fix_broken_null_behavior(self.build_delete_query(query));
        let query = bind_query(sqlx::query(&sql), &values);
        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected())
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
    match pool.kind {
        PoolKind::Sqlite => {
            let row = sqlx::query("pragma table_info('versions')").fetch_optional(&pool.pool).await?;
            let mut versions: HashSet<String> = HashSet::new();
            if row.is_none() {
                info!("Installing new SQLite3 database...");
                sqlx::query("create table versions (version text not null)").execute(&pool.pool).await?;
            } else {
                let rows = sqlx::query("select version from versions").fetch_all(&pool.pool).await?;
                for row in rows {
                    versions.insert(row.try_get("version")?);
                }
            }
            for (name, statements) in SQLITE_MIGRATIONS {
                if versions.contains(name.to_owned()) {
                    continue;
                }
                info!("Running migration: {}", name);
                let mut tx = pool.pool.begin().await?;
                for statement in statements.iter() {
                    sqlx::query(statement).execute(&mut tx).await?;
                }
                sqlx::query("insert into versions values ($1)").bind(name).execute(&mut tx).await?;
                tx.commit().await?;
            }
        },
        PoolKind::Postgres => {
            let row = sqlx::query("SELECT 1 FROM pg_catalog.pg_tables WHERE schemaname = 'public' AND tablename = 'versions'")
                .fetch_optional(&pool.pool).await?;
            let mut versions: HashSet<String> = HashSet::new();
            if row.is_none() {
                info!("Installing new PostgreSQL database...");
                sqlx::query("create table versions (version varchar(100) not null)").execute(&pool.pool).await?;
            } else {
                let rows = sqlx::query("select version from versions").fetch_all(&pool.pool).await?;
                for row in rows {
                    versions.insert(row.try_get("version")?);
                }
            }
            for (name, statements) in POSTGRES_MIGRATIONS {
                if versions.contains(name.to_owned()) {
                    continue;
                }
                info!("Running migration: {}", name);
                let mut tx = pool.pool.begin().await?;
                for statement in statements.iter() {
                    sqlx::query(statement).execute(&mut tx).await?;
                }
                sqlx::query("insert into versions values ($1)").bind(name).execute(&mut tx).await?;
                tx.commit().await?;
            }
        },
    }
    Ok(pool)
}
