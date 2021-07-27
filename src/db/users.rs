/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DB queries related to users

use super::{DbError, Page, Pool, count_remaining};

use sea_query::{Expr, Iden, Query, SelectStatement};
use sqlx::Row;

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Username,
    Password,
    Name,
    Email,
    Website,
    Trusted,
    Admin,
}

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

fn get_default_user_query() -> SelectStatement {
    Query::select().from(Users::Table)
        .columns(vec![
            Users::Id,
            Users::Username,
            Users::Name,
            Users::Email,
            Users::Website,
            Users::Trusted,
            Users::Admin,
        ])
        .to_owned()
}

async fn query_users(
    pool: &Pool,
    select: &SelectStatement,
) -> Result<Vec<User>, DbError> {
    let mut rows = pool.select(select).await?.into_iter();
    let mut content = Vec::new();
    while let Some(row) = rows.next() {
        content.push(User {
            id: row.try_get(0)?,
            username: row.try_get(1)?,
            name: row.try_get(2)?,
            email: row.try_get(3)?,
            website: row.try_get(4)?,
            trusted: row.try_get(5)?,
            admin: row.try_get(6)?,
        });
    }
    Ok(content)
}

pub async fn get_password_by_username(pool: &Pool, username: &str) -> Result<Option<Password>, DbError> {
    let result = pool.select_optional(Query::select().from(Users::Table)
        .columns(vec![Users::Id, Users::Password])
        .and_where(Expr::col(Users::Username).eq(username)))
        .await?;
    if let Some(row) = result {
        Ok(Some(Password {
            user_id: row.try_get(0)?,
            password: row.try_get(1)?,
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_password_by_user_id(pool: &Pool, user_id: i64) -> Result<Option<Password>, DbError> {
    let result = pool.select_optional(Query::select().from(Users::Table)
        .columns(vec![Users::Id, Users::Password])
        .and_where(Expr::col(Users::Id).eq(user_id)))
        .await?;
    if let Some(row) = result {
        Ok(Some(Password {
            user_id: row.try_get(0)?,
            password: row.try_get(1)?,
        }))
    } else {
        Ok(None)
    }
}

pub async fn admin_exists(pool: &Pool) -> Result<bool, DbError> {
    Ok(pool.select_optional(Query::select().from(Users::Table)
            .expr(Expr::value(1))
            .and_where(Expr::col(Users::Admin).eq(true)))
        .await?
        .is_some())
}

pub async fn create_user(pool: &Pool, new_user: NewUser) -> Result<User, DbError> {
    let id = pool.insert(Query::insert()
        .into_table(Users::Table)
        .columns(vec![
            Users::Username,
            Users::Password,
            Users::Name,
            Users::Email,
            Users::Website,
            Users::Trusted,
            Users::Admin,
        ])
        .values_panic(vec![
            new_user.username.as_str().into(),
            new_user.password.as_str().into(),
            new_user.name.as_str().into(),
            new_user.email.as_str().into(),
            new_user.website.as_str().into(),
            new_user.trusted.into(),
            new_user.admin.into(),
        ])
        .returning_col(Users::Id)).await?;
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
    pool.update(Query::update().table(Users::Table)
        .value(Users::Password, password.into())
        .and_where(Expr::col(Users::Id).eq(user_id)))
        .await?;
    Ok(())
}

pub async fn get_user_by_id(pool: &Pool, id: i64) -> Result<Option<User>, DbError> {
    Ok(query_users(pool, get_default_user_query().and_where(Expr::col(Users::Id).eq(id))).await?.into_iter().next())
}

pub async fn get_users(pool: &Pool, limit: usize, offset: usize) -> Result<Page<User>, DbError> {
    let mut query = get_default_user_query();
    query.order_by(Users::Username, sea_query::Order::Asc)
        .limit(limit as u64)
        .offset(offset as u64);
    let content = query_users(pool, &query).await?;
    let remaining = count_remaining(pool, content.len(), limit, offset,
        Query::select().from(Users::Table).expr(Expr::count(Expr::col(Users::Id)))).await?;
    Ok(Page { content, remaining, limit })
}

pub async fn update_user(pool: &Pool, id: i64, data: UpdateUser) -> Result<(), DbError> {
    let mut update = Query::update().table(Users::Table)
        .value(Users::Username, data.username.into())
        .value(Users::Name, data.name.into())
        .value(Users::Email, data.email.into())
        .value(Users::Website, data.website.into())
        .value(Users::Trusted, data.trusted.into())
        .value(Users::Admin, data.admin.into())
        .and_where(Expr::col(Users::Id).eq(id))
        .to_owned();
    if let Some(password) = data.password {
        update = update.value(Users::Password, password.into()).to_owned();
    }
    pool.update(&update).await?;
    Ok(())
}

pub async fn delete_user(pool: &Pool, id: i64) -> Result<(), DbError> {
    pool.delete(Query::delete().from_table(Users::Table)
        .and_where(Expr::col(Users::Id).eq(id)))
        .await?;
    Ok(())
}
