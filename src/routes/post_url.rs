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
async fn create(mut db: Connection<Db>, mut connection: Conn, req: Json<MyUrl>) -> ApiResponse {
    diesel::sql_function!(fn last_insert_id() -> BigInt);
    if let Err(err) = check_url(&req.origin) {
        return ApiResponse::err(Status::BadRequest, format!("invalid url: {:?}", err));
    }

    let body = check_and_set_expire(req);
    if let Err(msg) = body {
        return ApiResponse::err(Status::BadRequest, msg);
    }
    let mut req = body.ok().unwrap();
    req.created_at_secs = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backward")
            .as_secs() as i64,
    );

    // generate random short url
    let mut msg = String::new();
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
                    println!("uploaed to redis");
                }
                ApiResponse::ok(json::to_string(&object).unwrap())
            }
            Err(err) => {
                msg = err.to_string();
                continue;
            }
        };
        return result;
    }
    ApiResponse::err(Status::InternalServerError, msg)
}

#[get("/")]
async fn list(mut db: Connection<Db>, mut connection: Conn) -> ApiResponse {
    let shorts: Result<Vec<String>, _> = myurl::table.select(myurl::short).load(&mut db).await;
    if let Err(msg) = shorts {
        return ApiResponse::err(Status::InternalServerError, msg.to_string());
    }
    let shorts = shorts.ok().unwrap();
    let mut urls: Vec<MyUrl> = vec![];
    for short in &shorts {
        let current_url = match MyUrl::from_redis(&mut connection, short) {
            Ok(val) => val,
            Err(msg) => {
                println!("fail to get value from cache {:?}", msg);
                MyUrl {
                    id: None,
                    origin: String::from("unknown"),
                    short: Some(short.to_owned()),
                    expire_at_secs: Some(0),
                    created_at_secs: None,
                }
            }
        };
        urls.push(current_url);
    }
    ApiResponse::ok(json::to_string(&urls).unwrap())
}

fn check_url(to_check: &String) -> Result<bool, ParseError> {
    let result = Url::parse(to_check)?;
    if result.scheme().len() == 0 {
        return Ok(false);
    } else if let None = result.host() {
        return Ok(false);
    }
    Ok(true)
}

fn check_and_set_expire(Json(mut body): Json<MyUrl>) -> Result<Json<MyUrl>, String> {
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time goes backward")
        .as_secs() as i64;
    let expire = body
        .expire_at_secs
        .unwrap_or_else(|| config::MAX_URL_EXPIRE_SECS);
    if expire > config::MAX_URL_EXPIRE_SECS {
        return Err(format!(
            "max url expire secs is {}, found {}",
            config::MAX_URL_EXPIRE_SECS,
            expire
        ));
    } else if expire < config::MIN_URL_EXPIRE_SECS {
        return Err(format!(
            "min url expire secs is {}, found {}",
            config::MIN_URL_EXPIRE_SECS,
            expire
        ));
    } else {
        body.expire_at_secs = Some(expire + now_secs);
    }
    Ok(Json(body))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("adding url endpoint", |rocket| async {
        rocket.mount("/url", routes![create, list])
    })
}
