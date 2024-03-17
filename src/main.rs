#[macro_use]
extern crate rocket;

mod config;
mod database;
mod id_generator;
mod middleware;
mod model;
mod routes;

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
