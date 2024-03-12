use crate::config;
use rand::{thread_rng, Rng};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{http::Header, Data, Request, Response};
pub struct RequestIdInjector {}

impl RequestIdInjector {
    pub fn new() -> RequestIdInjector {
        RequestIdInjector {}
    }
}

#[rocket::async_trait]
impl Fairing for RequestIdInjector {
    fn info(&self) -> Info {
        Info {
            name: "Request ID Injector",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        if let Some(_) = request.headers().get(config::REQUEST_ID_HEADER).next() {
            return;
        }
        let request_id = format!("{:x}", thread_rng().gen::<u64>());
        request.add_header(Header::new(config::REQUEST_ID_HEADER, request_id));
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let request_id = request
            .headers()
            .get(config::REQUEST_ID_HEADER)
            .next()
            .unwrap()
            .to_string();
        response.set_header(Header::new(config::REQUEST_ID_HEADER, request_id));
    }
}
