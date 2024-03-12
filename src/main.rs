#[macro_use]
extern crate rocket;

mod middleware {
    pub mod request_id;
}
mod config;
mod database {
    pub mod diesel_mysql;
    pub mod r2d2_redis;
}
mod model {
    pub mod url;
}
mod routes {
    pub mod get_short_url;
    pub mod post_url;
    pub mod response;
}
mod id_generator {
    pub mod base62;
    pub mod random;
}

#[get("/")]
fn index() -> &'static str {
    "Hello world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(database::diesel_mysql::stage())
        .attach(routes::post_url::stage())
        .attach(routes::get_short_url::stage())
        .mount("/", routes![index])
        .attach(middleware::request_id::RequestIdInjector::new())
        .manage(database::r2d2_redis::init_pool())
}
