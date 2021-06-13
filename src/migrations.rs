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
         html text not null,
         markdown text not null,
         created text not null
     )",
    ]),
];
