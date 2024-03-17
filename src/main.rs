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

#[cfg(test)]
mod tests {
    use super::model::url;
    use super::*;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn create_and_access_url() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let body = url::Url {
            id: None,
            origin: String::from("https://google.com"),
            short: None,
            expire_at_secs: None,
            created_at_secs: None,
        };
        let response = client.post("/url").json(&body).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let response: Option<url::Url> = response.into_json();
        assert!(!response.is_none());
        let response = response.unwrap();
        assert_eq!(response.origin, body.origin);

        // send back short url
        let response = client
            .get(String::from("/") + response.short.as_ref().unwrap().as_str())
            .dispatch();
        assert_eq!(response.status(), Status::Found);
    }

    #[test]
    fn access_invalid_url() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/invalidurl").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }
}
