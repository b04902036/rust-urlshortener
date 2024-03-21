use crate::config;
use crate::database::{diesel_mysql::Db, r2d2_redis::Conn};
use crate::id_generator::random;
use crate::model::url::{url as myurl, Url as MyUrl};
use crate::routes::response::ApiResponse;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::serde::json::{self, Json};
use rocket_db_pools::diesel::prelude::*;
use rocket_db_pools::Connection;
use std::time::{SystemTime, UNIX_EPOCH};
use url::{ParseError, Url};

#[post("/", data = "<req>")]
async fn create(
    mut db: Connection<Db>,
    mut connection: Conn,
    mut req: Json<MyUrl>,
) -> Result<ApiResponse, ApiResponse> {
    diesel::sql_function!(fn last_insert_id() -> BigInt);
    if let Err(err) = check_url(&req.origin) {
        return Err(ApiResponse::err(
            Status::BadRequest,
            format!("invalid url: {:?}", err),
        ));
    }

    req = check_and_set_expire(req)?;
    req.created_at_secs = Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64);

    // generate random short url
    let mut msg = diesel::result::Error::RollbackTransaction;
    for _ in 0..config::ID_REGEN_CNT {
        let short = random::gen_short_url();
        let mut body = req.clone();
        body.short = Some(short);

        let object = db
            .transaction(|mut conn| {
                Box::pin(async move {
                    diesel::insert_into(myurl::table)
                        .values(&*body)
                        .execute(&mut conn)
                        .await?;

                    body.id = Some(
                        myurl::table
                            .select(last_insert_id())
                            .first(&mut conn)
                            .await?,
                    );

                    Ok::<_, diesel::result::Error>(body)
                })
            })
            .await;
        let result = match object {
            Ok(Json(object)) => {
                if let Err(msg) = object.to_redis(&mut connection) {
                    println!("fail to upload to redis: {:?}", msg);
                } else {
                    println!("uploaded to redis");
                }
                Ok(ApiResponse::ok(json::to_string(&object)?))
            }
            Err(err) => {
                msg = err;
                continue;
            }
        };
        return result;
    }
    Err(ApiResponse::err(
        Status::InternalServerError,
        msg.to_string(),
    ))
}

#[get("/")]
async fn list(mut db: Connection<Db>, mut connection: Conn) -> Result<ApiResponse, ApiResponse> {
    let shorts: Vec<String> = myurl::table.select(myurl::short).load(&mut db).await?;
    let mut urls: Vec<MyUrl> = vec![];
    for short in &shorts {
        let current_url = MyUrl::from_redis(&mut connection, short).unwrap_or_else(|err| {
            println!("fail to get value from cache {:?}", err);
            MyUrl {
                id: None,
                origin: String::from("unknown"),
                short: Some(short.to_owned()),
                expire_at_secs: Some(0),
                created_at_secs: None,
            }
        });

        urls.push(current_url);
    }
    Ok(ApiResponse::ok(json::to_string(&urls)?))
}

fn check_url(to_check: &str) -> Result<(), ParseError> {
    let _result = Url::parse(to_check)?;
    Ok(())
}

fn check_and_set_expire(Json(mut body): Json<MyUrl>) -> Result<Json<MyUrl>, ApiResponse> {
    let expire = body
        .expire_at_secs
        .unwrap_or_else(|| config::MAX_URL_EXPIRE_SECS);
    if expire > config::MAX_URL_EXPIRE_SECS {
        return Err(ApiResponse::err(
            Status::BadRequest,
            format!(
                "max url expire secs is {}, found {}",
                config::MAX_URL_EXPIRE_SECS,
                expire
            ),
        ));
    } else if expire < config::MIN_URL_EXPIRE_SECS {
        return Err(ApiResponse::err(
            Status::BadRequest,
            format!(
                "min url expire secs is {}, found {}",
                config::MIN_URL_EXPIRE_SECS,
                expire
            ),
        ));
    } else {
        body.expire_at_secs = Some(expire);
    }
    Ok(Json(body))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("adding url endpoint", |rocket| async {
        rocket.mount("/url", routes![create, list])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expire_not_set_in_url() {
        let req = Json(MyUrl {
            id: None,
            origin: String::from("abc"),
            short: None,
            expire_at_secs: None,
            created_at_secs: None,
        });
        let req = check_and_set_expire(req);
        let target = config::MAX_URL_EXPIRE_SECS;
        assert_eq!(req.is_ok(), true);
        let expire_at = req.ok().unwrap().expire_at_secs;
        assert!(match expire_at {
            Some(x) if x == target => true,
            _ => false,
        });
    }

    #[test]
    fn expire_too_long() {
        let target = config::MAX_URL_EXPIRE_SECS * 2;
        let req = Json(MyUrl {
            id: None,
            origin: String::from("abc"),
            short: None,
            expire_at_secs: Some(target),
            created_at_secs: None,
        });
        let req = check_and_set_expire(req);
        assert_eq!(req.is_ok(), false);
    }

    #[test]
    fn expire_too_short() {
        let target = config::MIN_URL_EXPIRE_SECS - 1;
        let req = Json(MyUrl {
            id: None,
            origin: String::from("abc"),
            short: None,
            expire_at_secs: Some(target),
            created_at_secs: None,
        });
        let req = check_and_set_expire(req);
        assert_eq!(req.is_ok(), false);
    }

    #[test]
    fn unknown_url_format() {
        let urls = [
            "asdkjask",
            "https://",
            ":https://asd",
            "ht tps://google.com",
            "",
            "google.com",
            "https://",
        ];
        for url in urls {
            let result = check_url(url);
            assert!(result.is_err());
        }
    }
}
