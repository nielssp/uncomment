/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! XML comment import

use std::{collections::HashMap, fs::File, io::BufReader};

use crate::db::{DbError, Pool, comments::{self, CommentPosition}, threads};
use chrono::{DateTime, Utc};
use log::info;
use minidom::{Element, NSChoice};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("XML error")]
    XmlError(#[from] minidom::Error),
    #[error("date parsing error")]
    ChronoError(#[from] chrono::ParseError),
}

pub struct ImportThread {
    name: String,
    title: String,
    comments: Vec<ImportComment>,
 }

#[derive(Clone)]
pub struct ImportComment {
    id: String,
    name: String,
    website: String,
    message: String,
    created: DateTime<Utc>,
    replies: Vec<ImportComment>,
}

fn build_comment_tree(comment: &mut ImportComment, replies: &HashMap<String, Vec<ImportComment>>) {
    match replies.get(&comment.id) {
        Some(comment_replies) => {
            for reply in comment_replies {
                let mut clone = reply.clone();
                build_comment_tree(&mut clone, replies);
                comment.replies.push(clone);
            }
        },
        None => {
        },
    }
}

pub fn read_xml_comments(
    f: File,
) -> Result<Vec<ImportThread>, ImportError> {
    let f = BufReader::new(f);
    let mut reader = minidom::quick_xml::Reader::from_reader(f);
    let root = Element::from_reader(&mut reader)?;
    let mut threads = HashMap::new();
    let mut replies: HashMap<String, Vec<ImportComment>> = HashMap::new();
    for child in root.children() {
        if child.is("thread", NSChoice::Any) {
            if let Some(id) = child.attr("dsq:id") {
                match (
                    child.get_child("id", NSChoice::Any).map(|e| e.text()).filter(|id| !id.is_empty()),
                    child.get_child("title", NSChoice::Any).map(|e| e.text()),
                ) {
                    (Some(name), Some(title)) => {
                        info!("Importing thread {}", id);
                        threads.insert(id, ImportThread {
                            name,
                            title,
                            comments: Vec::new(),
                        });
                    },
                    _ => {},
                }
            }
        } else if child.is("post", NSChoice::Any) {
            if let Some(id) = child.attr("dsq:id") {
                match (
                    child.get_child("message", NSChoice::Any).map(|e| e.text()).filter(|message| !message.is_empty()),
                    child.get_child("createdAt", NSChoice::Any).map(|e| e.text()),
                    child.get_child("author", NSChoice::Any).map(|author| (
                            author.get_child("name", NSChoice::Any).map(|e| e.text()).filter(|name| !name.is_empty()),
                            author.get_child("username", NSChoice::Any).map(|e| e.text()).filter(|username| !username.is_empty()),
                    )),
                    child.get_child("thread", NSChoice::Any).map(|e| e.attr("dsq:id")).flatten(),
                ) {
                    (Some(message), Some(created_at), Some((Some(name), Some(username))), Some(thread_id)) => {
                        let website = format!("https://disqus.com/by/{}/", username);
                        let created = DateTime::parse_from_rfc3339(&created_at)?
                            .with_timezone(&Utc);
                        let comment = ImportComment {
                            id: id.to_owned(),
                            name,
                            website,
                            message,
                            created,
                            replies: Vec::new(),
                        };
                        info!("Importing comment {}", id);
                        match child.get_child("parent", NSChoice::Any).map(|e| e.attr("dsq:id")).flatten() {
                            Some(parent_id) => {
                                let parent_id = parent_id.to_owned();
                                match replies.get_mut(&parent_id) {
                                    Some(parent_replies) => parent_replies.push(comment),
                                    None => {
                                        let mut parent_replies = Vec::new();
                                        parent_replies.push(comment);
                                        replies.insert(parent_id, parent_replies);
                                    }
                                }
                            },
                            None => match threads.get_mut(&thread_id) {
                                Some(thread) => thread.comments.push(comment),
                                None => {},
                            },
                        };
                    },
                    _ => {},
                }
            }
        }
    }
    Ok(threads.into_iter().map(|(_, mut thread)| {
        for mut comment in thread.comments.iter_mut() {
            build_comment_tree(&mut comment, &replies);
        }
        thread
    }).collect())
}

async fn insert_imported_comment(
    pool: &Pool,
    thread_id: i32,
    parent: Option<&CommentPosition>,
    comment: &ImportComment,
) -> Result<CommentPosition, DbError> {
    let safe_html = ammonia::clean(&comment.message);
    comments::insert_comment(pool, thread_id, parent, &comments::NewComment {
        name: comment.name.clone(),
        email: "".to_owned(),
        website: comment.website.clone(),
        ip: "".to_owned(),
        html: safe_html,
        markdown: comment.message.clone(),
        status: comments::CommentStatus::Approved,
        created: comment.created,
    }).await
}

pub async fn insert_imported_comments(
    pool: &Pool,
    threads: Vec<ImportThread>,
) -> Result<(), DbError> {
    for thread in threads {
        let thread_id = match threads::get_thread_by_name(pool, &thread.name).await? {
            Some(t) => Ok(t),
            None => threads::create_thread(pool, threads::NewThread {
                name: thread.name,
                title: Some(thread.title),
            }).await,
        }?.id;
        let mut queue: Vec<(&ImportComment, Option<CommentPosition>)> = Vec::new();
        for comment in thread.comments.iter() {
            queue.push((comment, None));
        }
        while let Some((comment, parent)) = queue.pop() {
            let position = insert_imported_comment(pool, thread_id, parent.as_ref(), &comment).await?;
            for reply in comment.replies.iter() {
                queue.push((reply, Some(position)));
            }
        }
    }
    Ok(())
}
