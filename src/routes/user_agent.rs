use actix_web::{
    HttpRequest, HttpResponse, Responder, get,
    http::header::USER_AGENT,
    web::{Either, Json},
};

use crate::models::user_agent::UserAgent;

#[get("/user-agent")]
async fn user_agent_get(req: HttpRequest) -> Either<Json<UserAgent>, impl Responder> {
    match req.headers().get(USER_AGENT) {
        Some(user_agent) => match user_agent.to_str() {
            Err(err) => Either::Right(
                HttpResponse::BadRequest()
                    .body(format!("Could not parse the User-Agent header: {}", err)),
            ),
            Ok(user_agent) => Either::Left(Json(UserAgent::new(user_agent.to_string()))),
        },
        None => Either::Right(
            HttpResponse::BadRequest()
                .body("The incoming request does not have a User-Agent header"),
        ),
    }
}
