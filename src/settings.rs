/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Uncomment server settings

use config::{Config, ConfigError, Environment};

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub listen: String,
    pub sqlite_database: String,
    pub secret_key: String,
    pub rate_limit: i64,
    pub rate_limit_interval: i64,
    pub auto_threads: bool,
    pub require_name: bool,
    pub require_email: bool,
    pub moderate_all: bool,
    pub default_admin_username: Option<String>,
    pub default_admin_password: Option<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();
        s.set_default("listen", "127.0.0.1:5000")?;
        s.set_default("sqlite_database", "data.db")?;
        s.set_default("rate_limit", 10)?;
        s.set_default("rate_limit_interval", 10)?;
        s.set_default("auto_threads", true)?;
        s.set_default("require_name", false)?;
        s.set_default("require_email", false)?;
        s.set_default("moderate_all", false)?;
        s.merge(Environment::with_prefix("UNCOMMENT"))?;
        s.try_into()
    }
}
