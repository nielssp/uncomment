// Minimal migration system

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
         hierarchy text(100) not null,
         name text(100) not null,
         email text(100) not null,
         website text(100) not null,
         html text not null,
         markdown text not null,
         status text(50) not null,
         created text not null
     )",
     "create table users (
         id integer primary key autoincrement,
         username text(100) not null,
         password text(200) not null,
         name text(100) not null,
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
