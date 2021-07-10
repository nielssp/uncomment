/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DB queries related to sessions

use std::convert::TryInto;

use quaint::prelude::*;
use chrono::{DateTime, FixedOffset, Local};

use crate::db::{Pool, DbError};

use super::users::User;

pub struct Session {
    pub id: String,
    pub valid_until: DateTime<FixedOffset>,
    pub user: User,
}

pub async fn get_session(pool: &Pool, session_id: &str) -> Result<Option<Session>, DbError> {
    let conn = pool.check_out().await?;
    let mut result = conn.select(Select::from_table("sessions".alias("s"))
        .columns(vec![
            ("s", "id"),
            ("s", "valid_until"),
            ("u", "id"),
            ("u", "username"),
            ("u", "name"),
            ("u", "email"),
            ("u", "website"),
            ("u", "trusted"),
            ("u", "admin")
        ])
        .inner_join("users".alias("u").on(("u", "id").equals(Column::from(("s", "user_id")))))
        .so_that(("s", "id").equals(session_id))).await?.into_iter();
    if let Some(row) = result.next() {
        let valid_until: String = row[1].clone().try_into()?;
        Ok(Some(Session {
            id: row[0].clone().try_into()?,
            valid_until: DateTime::parse_from_rfc3339(valid_until.as_str())?,
            user: User {
                id: row[2].clone().try_into()?,
                username: row[3].clone().try_into()?,
                name: row[4].clone().try_into()?,
                email: row[5].clone().try_into()?,
                website: row[6].clone().try_into()?,
                trusted: row[7].clone().try_into()?,
                admin: row[8].clone().try_into()?,
            },
        }))
    } else {
        Ok(None)
    }
}

pub async fn create_session(pool: &Pool, session_id: &str, valid_until: DateTime<Local>, user_id: i64) -> Result<(), DbError> {
    let conn = pool.check_out().await?;
    conn.insert(Insert::single_into("sessions")
        .value("id", session_id)
        .value("valid_until", valid_until.to_rfc3339())
        .value("user_id", user_id)
        .build()).await?;
    Ok(())
}

pub async fn delete_session(pool: &Pool, session_id: &str) -> Result<(), DbError> {
    let conn = pool.check_out().await?;
    conn.delete(Delete::from_table("sessions").so_that("id".equals(session_id))).await?;
    Ok(())
}
