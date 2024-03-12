use rocket::http::{ContentType, Status};
use rocket::response;
use rocket::response::{Responder, Response};
use rocket::*;

pub struct ApiResponse {
    status: Status,
    message: String,
}

impl ApiResponse {
    pub fn ok(message: String) -> Self {
        ApiResponse {
            status: Status::Ok,
            message,
        }
    }
    pub fn err(status: Status, message: String) -> Self {
        ApiResponse { status, message }
    }
}

impl<'r> Responder<'r, 'static> for ApiResponse {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(self.message.respond_to(req)?)
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}
