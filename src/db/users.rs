/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DB queries related to users

use std::convert::TryInto;

use quaint::prelude::*;
use crate::db::{DbError, Pool, insert_id};

pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: String,
    pub website: String,
    pub trusted: bool,
    pub admin: bool,
}

pub struct NewUser {
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: String,
    pub website: String,
    pub trusted: bool,
    pub admin: bool,
}

pub async fn get_user(pool: &Pool, username: &str) -> Result<Option<User>, DbError> {
    let conn = pool.check_out().await?;
    let mut result = conn.select(Select::from_table("users")
        .columns(vec!["id", "username", "password", "name", "email", "website", "trusted", "admin"])
        .so_that("username".equals(username))).await?.into_iter();
    if let Some(row) = result.next() {
        Ok(Some(User {
            id: row[0].clone().try_into()?,
            username: row[1].clone().try_into()?,
            password: row[2].clone().try_into()?,
            name: row[3].clone().try_into()?,
            email: row[4].clone().try_into()?,
            website: row[5].clone().try_into()?,
            trusted: row[6].clone().try_into()?,
            admin: row[7].clone().try_into()?,
        }))
    } else {
        Ok(None)
    }
}

pub async fn admin_exists(pool: &Pool) -> Result<bool, DbError> {
    let conn = pool.check_out().await?;
    Ok(conn.select(Select::from_table("users")
            .value(1)
            .so_that("admin".equals(true))).await?.first().is_some())
}

pub async fn create_user(pool: &Pool, new_user: NewUser) -> Result<User, DbError> {
    let conn = pool.check_out().await?;
    let id = insert_id(conn.insert(Insert::single_into("users")
            .value("username", new_user.username.as_str())
            .value("password", new_user.password.as_str())
            .value("name", new_user.name.as_str())
            .value("website", new_user.website.as_str())
            .value("trusted", new_user.trusted)
            .value("admin", new_user.admin)
            .build()).await?)?;
    Ok(User {
        id,
        username: new_user.username,
        password: new_user.password,
        name: new_user.name,
        email: new_user.email,
        website: new_user.website,
        trusted: new_user.trusted,
        admin: new_user.admin,
    })
}

pub async fn change_password(pool: &Pool, user_id: i64, password: &str) -> Result<(), DbError> {
    let conn = pool.check_out().await?;
    conn.update(Update::table("users")
        .set("password", password)
        .so_that("id".equals(user_id))).await?;
    Ok(())
}
