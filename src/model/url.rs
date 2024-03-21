use crate::config;
use crate::database::r2d2_redis::Conn;
use anyhow::Result as AnyResult;
use anyhow::{anyhow, format_err};
use r2d2_redis::redis::Commands;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::diesel::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug, Queryable, Deserialize, Serialize, Insertable, PartialEq)]
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
        id -> BigInt,
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

            // these 'let' are to let the corresponding String
            // representation live long enough till we send it to redis
            let id_str: String;
            let created_at_str: String;
            let mut cache_url: Vec<(&str, &String)> =
                vec![("origin", &self.origin), ("expires_at_secs", &expires)];
            if let Some(id) = self.id {
                id_str = id.to_string();
                cache_url.push(("id", &id_str));
            }
            if let Some(created_at) = self.created_at_secs {
                created_at_str = created_at.to_string();
                cache_url.push(("created_at_secs", &created_at_str));
            }

            if let Some(ref key) = self.short {
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
            println!("got value: {:?}", cache_url);
            let id = cache_url
                .get("id")
                .and_then(|val| val.parse::<i64>().map_or_else(|_e| None, |v| Some(v)));
            let origin = cache_url
                .get("origin")
                .ok_or(anyhow!("origin field not exist"))?;
            let expires_at = cache_url
                .get("expires_at_secs")
                .ok_or(anyhow!("expires_at_secs not exist"))?
                .parse::<i64>()?;
            let created_at = cache_url
                .get("created_at_secs")
                .and_then(|val| val.parse::<i64>().map_or_else(|_e| None, |v| Some(v)));

            Ok(Url {
                id,
                origin: origin.to_owned(),
                short: Some(key.to_owned()),
                expire_at_secs: Some(expires_at),
                created_at_secs: created_at,
            })
        } else {
            Err(anyhow!(""))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Url;
    use crate::database::r2d2_redis::{init_pool, Conn};

    #[test]
    fn upload_and_download_from_redis() {
        let pool = init_pool();
        let mut connection = match pool.get() {
            Ok(database) => Conn(Some(database)),
            Err(e) => {
                panic!("fail to get redis connection {:?}", e);
            }
        };

        let url = Url {
            id: Some(123),
            origin: String::from("http://google.com"),
            short: Some(String::from("abcde12")),
            expire_at_secs: Some(12345),
            created_at_secs: Some(34567),
        };
        let result = url.to_redis(&mut connection);
        assert_eq!(result.is_ok(), true);

        // get back from redis
        let cached_url = Url::from_redis(&mut connection, url.short.as_ref().unwrap().as_str());
        assert_eq!(cached_url.is_ok(), true);
        let cached_url = cached_url.ok().unwrap();
        assert_eq!(url, cached_url);
    }
}
