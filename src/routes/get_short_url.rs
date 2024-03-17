use crate::database::{diesel_mysql::Db, r2d2_redis::Conn};
use crate::model::url::url::expire_at_secs;
use crate::model::url::{url as myurl, Url as MyUrl};
use crate::routes::response::ApiResponse;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket_db_pools::diesel::prelude::*;
use rocket_db_pools::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

#[get("/<url_id>")]
async fn get_short_url(
    url_id: &str,
    mut db: Connection<Db>,
    mut connection: Conn,
) -> Result<Redirect, ApiResponse> {
    // check from redis first
    let now_sec = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
    match MyUrl::from_redis(&mut connection, &url_id.to_string()) {
        Ok(val) => {
            let expire_at = val
                .expire_at_secs
                .map_or_else(|| 0, |v| v + val.created_at_secs.map_or_else(|| 0, |v| v));
            if expire_at > now_sec {
                return Ok(Redirect::found(val.origin));
            } else {
                return Err(ApiResponse::err(Status::NotFound, format!("url not found")));
            }
        }
        Err(msg) => {
            println!("fail to get from cache: {:?}", msg);
        }
    };

    let predicate = myurl::short.eq(&url_id).and(expire_at_secs.gt(&now_sec));
    let shorts: Vec<(String, i64)> = myurl::table
        .select((myurl::origin, myurl::expire_at_secs))
        .filter(predicate)
        .load(&mut db)
        .await?;
    if shorts.len() > 1 {
        return Err(ApiResponse::err(
            Status::InternalServerError,
            format!("more than 1 record found"),
        ));
    } else if shorts.len() == 0 {
        return Err(ApiResponse::err(Status::NotFound, format!("url not found")));
    }
    let (origin_url, expire) = shorts.get(0).unwrap();
    // upload to cache
    let result = MyUrl {
        id: None,
        origin: origin_url.clone(),
        short: None,
        expire_at_secs: Some(*expire),
        created_at_secs: None,
    }
    .to_redis(&mut connection);
    if let Err(msg) = result {
        println!("fail to upload to cache: {:?}", msg);
    }
    if *expire > now_sec {
        Ok(Redirect::found(origin_url.to_owned()))
    } else {
        Err(ApiResponse::err(Status::NotFound, format!("url not found")))
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("adding get short url endpoint", |rocket| async {
        rocket.mount("/", routes![get_short_url])
    })
}
