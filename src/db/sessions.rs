/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DB queries related to sessions

use chrono::{DateTime, FixedOffset, Utc};
use sea_query::{Expr, Iden, Query};
use sqlx::Row;

use crate::db::{DbError, Pool, users::Users};

use super::users::User;

#[derive(Iden)]
pub enum Sessions {
    Table,
    Id,
    ValidUntil,
    UserId,
}

pub struct Session {
    pub id: String,
    pub valid_until: DateTime<FixedOffset>,
    pub user: User,
}

pub async fn get_session(pool: &Pool, session_id: &str) -> Result<Option<Session>, DbError> {
    let result = pool.select_optional(Query::select()
        .columns(vec![
            (Sessions::Table, Sessions::Id),
            (Sessions::Table, Sessions::ValidUntil),
        ])
        .columns(vec![
            (Users::Table, Users::Id),
            (Users::Table, Users::Username),
            (Users::Table, Users::Name),
            (Users::Table, Users::Email),
            (Users::Table, Users::Website),
            (Users::Table, Users::Trusted),
            (Users::Table, Users::Admin),
        ])
        .from(Sessions::Table)
        .inner_join(Users::Table, Expr::tbl(Users::Table, Users::Id).equals(Sessions::Table, Sessions::UserId))
        .and_where(Expr::tbl(Sessions::Table, Sessions::Id).eq(session_id)))
        .await?;
    if let Some(row) = result {
        let valid_until: String = row.try_get(1)?;
        Ok(Some(Session {
            id: row.try_get(0)?,
            valid_until: DateTime::parse_from_rfc3339(valid_until.as_str())?,
            user: User {
                id: row.try_get(2)?,
                username: row.try_get(3)?,
                name: row.try_get(4)?,
                email: row.try_get(5)?,
                website: row.try_get(6)?,
                trusted: row.try_get(7)?,
                admin: row.try_get(8)?,
            },
        }))
    } else {
        Ok(None)
    }
}

pub async fn create_session(pool: &Pool, session_id: &str, valid_until: DateTime<Utc>, user_id: i64) -> Result<(), DbError> {
    pool.insert(Query::insert().into_table(Sessions::Table)
        .columns(vec![Sessions::Id, Sessions::ValidUntil, Sessions::UserId])
        .values_panic(vec![
            session_id.into(),
            valid_until.to_rfc3339().into(),
            user_id.into(),
        ])).await?;
    Ok(())
}

pub async fn delete_session(pool: &Pool, session_id: &str) -> Result<(), DbError> {
    pool.delete(Query::delete().from_table(Sessions::Table)
        .and_where(Expr::col(Sessions::Id).eq(session_id)))
        .await?;
    Ok(())
}

pub async fn delete_expired_sessions(pool: &Pool) -> Result<(), DbError> {
    pool.delete(Query::delete().from_table(Sessions::Table)
        .and_where(Expr::col(Sessions::ValidUntil).lt(Utc::now().naive_utc())))
        .await?;
    Ok(())
}
