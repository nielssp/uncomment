/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Minimal migration system

use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, SchemaBuilder, Table};

use crate::db::{comments::Comments, sessions::Sessions, users::Users};

use super::threads::Threads;

pub static MIGRATIONS: &'static [(&'static str, fn(&dyn SchemaBuilder) -> Vec<String>)] = &[
    ("V1_Init", |builder| {
        vec![
            Table::create()
                .table(Threads::Table)
                .col(ColumnDef::new(Threads::Id).integer().auto_increment().primary_key())
                .col(ColumnDef::new(Threads::Name).string().not_null().unique_key())
                .col(ColumnDef::new(Threads::Title).string())
                .build_any(builder),
            Table::create()
                .table(Users::Table)
                .col(ColumnDef::new(Users::Id).integer().auto_increment().primary_key())
                .col(ColumnDef::new(Users::Username).string().not_null().unique_key())
                .col(ColumnDef::new(Users::Password).string().not_null())
                .col(ColumnDef::new(Users::Name).string().not_null())
                .col(ColumnDef::new(Users::Email).string().not_null())
                .col(ColumnDef::new(Users::Website).string().not_null())
                .col(ColumnDef::new(Users::Trusted).boolean().not_null().default(false))
                .col(ColumnDef::new(Users::Admin).boolean().not_null().default(false))
                .build_any(builder),
            Table::create()
                .table(Sessions::Table)
                .col(ColumnDef::new(Sessions::Id).string().primary_key())
                .col(ColumnDef::new(Sessions::UserId).integer().not_null())
                .col(ColumnDef::new(Sessions::ValidUntil).timestamp().not_null())
                .foreign_key(ForeignKey::create()
                    .name("FK_sessions_user_id")
                    .from(Sessions::Table, Sessions::UserId)
                    .to(Users::Table, Threads::Id)
                    .on_delete(ForeignKeyAction::Cascade))
                .build_any(builder),
            Table::create()
                .table(Comments::Table)
                .col(ColumnDef::new(Comments::Id).integer().auto_increment().primary_key())
                .col(ColumnDef::new(Comments::ThreadId).integer().not_null())
                .col(ColumnDef::new(Comments::ParentId).integer())
                .col(ColumnDef::new(Comments::Level1Id).integer())
                .col(ColumnDef::new(Comments::Level2Id).integer())
                .col(ColumnDef::new(Comments::Level3Id).integer())
                .col(ColumnDef::new(Comments::Level4Id).integer())
                .col(ColumnDef::new(Comments::Level5Id).integer())
                .col(ColumnDef::new(Comments::Level6Id).integer())
                .col(ColumnDef::new(Comments::UserId).integer())
                .col(ColumnDef::new(Comments::Name).string().not_null())
                .col(ColumnDef::new(Comments::Email).string().not_null())
                .col(ColumnDef::new(Comments::Website).string().not_null())
                .col(ColumnDef::new(Comments::Ip).string().not_null())
                .col(ColumnDef::new(Comments::Html).text().not_null())
                .col(ColumnDef::new(Comments::Markdown).text().not_null())
                .col(ColumnDef::new(Comments::Status).string().not_null())
                .col(ColumnDef::new(Comments::Created).timestamp().not_null())
                .foreign_key(ForeignKey::create()
                    .name("FK_comments_thread_id")
                    .from(Comments::Table, Comments::ThreadId)
                    .to(Threads::Table, Threads::Id)
                    .on_delete(ForeignKeyAction::Cascade))
                .foreign_key(ForeignKey::create()
                    .name("FK_comments_user_id")
                    .from(Comments::Table, Comments::UserId)
                    .to(Users::Table, Threads::Id)
                    .on_delete(ForeignKeyAction::SetNull))
                .build_any(builder),
        ]
    }),
];
