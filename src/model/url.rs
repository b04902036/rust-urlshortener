use crate::config;
use crate::database::r2d2_redis::Conn;
use anyhow::Result as AnyResult;
use anyhow::{anyhow, format_err};
use r2d2_redis::redis::Commands;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::diesel::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug, Queryable, Deserialize, Serialize, Insertable)]
#[diesel(table_name = url)]
pub struct Url {
    pub id: Option<i64>,
    pub origin: String,
    pub short: Option<String>,
    pub expire_at_secs: Option<i64>,
    pub created_at_secs: Option<i64>,
}

diesel::table! {
    url(id) {
        id -> Nullable<BigInt>,
        origin -> Text,
        short -> Text,
        expire_at_secs -> BigInt,
        created_at_secs -> BigInt,
    }
}

impl Url {
    pub fn to_redis(&self, connection: &mut Conn) -> AnyResult<()> {
        if connection.is_none() {
            return Err(anyhow!("cache service not available"));
        }
        if let Some(conn) = connection.as_mut() {
            let expires = self
                .expire_at_secs
                .unwrap_or_else(|| config::MAX_URL_EXPIRE_SECS)
                .to_string();
            let cache_url = [("origin", &self.origin), ("expires_at_secs", &expires)];
            if let Some(key) = self.short.as_ref() {
                conn.hset_multiple(key, &cache_url)?;
            } else {
                return Err(anyhow!("short url not set"));
            }
        }
        Ok(())
    }

    pub fn from_redis(connection: &mut Conn, key: &str) -> AnyResult<Self> {
        if connection.is_none() {
            return Err(format_err!("cache service not available"));
        }
        if let Some(conn) = connection.as_mut() {
            let cache_url: HashMap<String, String> = conn.hgetall(key)?;

            let origin = cache_url
                .get("origin")
                .ok_or(anyhow!("origin field not exist"))?;
            let expires_at_secs = cache_url
                .get("expires_at_secs")
                .ok_or(anyhow!("expires_at_secs not exist"))?
                .parse::<i64>()?;

            Ok(Url {
                id: None,
                origin: origin.to_owned(),
                short: Some(key.to_owned()),
                expire_at_secs: Some(expires_at_secs),
                created_at_secs: None,
            })
        } else {
            Err(anyhow!(""))
        }
    }
}
