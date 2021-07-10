/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DB queries related to users

use std::convert::TryInto;

use quaint::{pooled::PooledConnection, prelude::*};

use super::{DbError, Page, Pool, count_remaining, insert_id};

#[derive(serde::Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub name: String,
    pub email: String,
    pub website: String,
    pub trusted: bool,
    pub admin: bool,
}

pub struct Password {
    pub user_id: i64,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: String,
    pub website: String,
    pub trusted: bool,
    pub admin: bool,
}

#[derive(serde::Deserialize)]
pub struct UpdateUser {
    pub username: String,
    pub password: Option<String>,
    pub name: String,
    pub email: String,
    pub website: String,
    pub trusted: bool,
    pub admin: bool,
}

fn get_default_user_query<'a>() -> Select<'a> {
    Select::from_table("users".alias("u"))
        .columns(vec!["id", "username", "name", "email", "website", "trusted", "admin"])
}

async fn query_users<'a>(
    conn: &PooledConnection,
    select: Select<'a>,
) -> Result<Vec<User>, DbError> {
    let mut rows = conn.select(select).await?.into_iter();
    let mut content = Vec::new();
    while let Some(row) = rows.next() {
        content.push(User {
            id: row[0].clone().try_into()?,
            username: row[1].clone().try_into()?,
            name: row[2].clone().try_into()?,
            email: row[3].clone().try_into()?,
            website: row[4].clone().try_into()?,
            trusted: row[5].clone().try_into()?,
            admin: row[6].clone().try_into()?,
        });
    }
    Ok(content)
}

pub async fn get_password_by_username(pool: &Pool, username: &str) -> Result<Option<Password>, DbError> {
    let conn = pool.check_out().await?;
    let mut result = conn.select(Select::from_table("users")
        .columns(vec!["id", "password"])
        .so_that("username".equals(username))).await?.into_iter();
    if let Some(row) = result.next() {
        Ok(Some(Password {
            user_id: row[0].clone().try_into()?,
            password: row[1].clone().try_into()?,
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_password_by_user_id(pool: &Pool, user_id: i64) -> Result<Option<Password>, DbError> {
    let conn = pool.check_out().await?;
    let mut result = conn.select(Select::from_table("users")
        .columns(vec!["id", "password"])
        .so_that("id".equals(user_id))).await?.into_iter();
    if let Some(row) = result.next() {
        Ok(Some(Password {
            user_id: row[0].clone().try_into()?,
            password: row[1].clone().try_into()?,
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
            .value("email", new_user.email.as_str())
            .value("website", new_user.website.as_str())
            .value("trusted", new_user.trusted)
            .value("admin", new_user.admin)
            .build()).await?)?;
    Ok(User {
        id,
        username: new_user.username,
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

pub async fn get_user_by_id(pool: &Pool, id: i64) -> Result<Option<User>, DbError> {
    let conn = pool.check_out().await?;
    Ok(query_users(&conn, get_default_user_query().so_that("id".equals(id))).await?.into_iter().next())
}

pub async fn get_users(pool: &Pool, limit: usize, offset: usize) -> Result<Page<User>, DbError> {
    let conn = pool.check_out().await?;
    let query = get_default_user_query()
        .order_by("username")
        .limit(limit)
        .offset(offset);
    let content = query_users(&conn, query).await?;
    let remaining = count_remaining(&conn, content.len(), limit, offset,
        Select::from_table("users").value(count(asterisk()))).await?;
    Ok(Page { content, remaining, limit })
}

pub async fn update_user(pool: &Pool, id: i64, data: UpdateUser) -> Result<(), DbError> {
    let conn = pool.check_out().await?;
    let mut update = Update::table("users")
        .set("username", data.username.as_str())
        .set("name", data.name.as_str())
        .set("email", data.email.as_str())
        .set("website", data.website.as_str())
        .set("trusted", data.trusted)
        .set("admin", data.admin)
        .so_that("id".equals(id));
    if let Some(password) = data.password {
        update = update.set("password", password.clone());
    }
    conn.update(update).await?;
    Ok(())
}

pub async fn delete_user(pool: &Pool, id: i64) -> Result<(), DbError> {
    let conn = pool.check_out().await?;
    conn.delete(Delete::from_table("users").so_that("id".equals(id))).await?;
    Ok(())
}
