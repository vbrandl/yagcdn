use actix_web::{HttpResponse, ResponseError};

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("HttpClient({0})")]
    HttpClient(#[from] awc::error::SendRequestError),
    #[error("HttpPayload({0})")]
    HttpPayload(#[from] awc::error::PayloadError),
    #[error("HttpServer({0})")]
    HttpServer(#[from] actix_web::Error),
    #[error("Io({0})")]
    Io(#[from] std::io::Error),
    #[error("Json({0})")]
    Json(#[from] awc::error::JsonPayloadError),
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().finish()
    }
}
