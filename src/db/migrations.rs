/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Minimal migration system

pub static SQLITE_MIGRATIONS: &'static [(&'static str, &'static [&'static str])] = &[
    ("V1_Init", &[
     "create table threads (
         id integer primary key autoincrement,
         name text(100) unique not null,
         title text(100) null
     )",
     "create table comments (
         id integer primary key autoincrement,
         thread_id integer not null,
         parent_id integer null,
         level1_id integer null,
         level2_id integer null,
         level3_id integer null,
         level4_id integer null,
         level5_id integer null,
         level6_id integer null,
         user_id integer null,
         name text(100) not null,
         email text(100) not null,
         website text(100) not null,
         ip text(100) not null,
         html text not null,
         markdown text not null,
         status text(50) not null,
         created text not null
     )",
     "create table users (
         id integer primary key autoincrement,
         username text(100) not null,
         password text(200) not null,
         name text(100) not null unique,
         email text(100) not null,
         website text(100) not null,
         trusted boolean not null default 0,
         admin boolean not null default 0
     )",
     "create table sessions (
        id text(100) primary key,
        user_id integer not null,
        valid_until text not null
     )",
    ]),
];

pub static POSTGRES_MIGRATIONS: &'static [(&'static str, &'static [&'static str])] = &[
    ("V1_Init", &[
     "create table threads (
         id serial primary key,
         name varchar(100) unique not null,
         title varchar(100) null
     )",
     "create table comments (
         id serial primary key,
         thread_id integer not null,
         parent_id integer null,
         level1_id integer null,
         level2_id integer null,
         level3_id integer null,
         level4_id integer null,
         level5_id integer null,
         level6_id integer null,
         user_id integer null,
         name varchar(100) not null,
         email varchar(100) not null,
         website varchar(100) not null,
         ip varchar(100) not null,
         html text not null,
         markdown text not null,
         status varchar(50) not null,
         created timestamp not null
     )",
     "create table users (
         id serial primary key,
         username varchar(100) not null,
         password varchar(200) not null,
         name varchar(100) not null unique,
         email varchar(100) not null,
         website varchar(100) not null,
         trusted boolean not null default false,
         admin boolean not null default false
     )",
     "create table sessions (
        id varchar(100) primary key,
        user_id integer not null,
        valid_until timestamp not null
     )",
    ]),
];
