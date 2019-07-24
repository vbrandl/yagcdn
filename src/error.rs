use actix_web::{HttpResponse, ResponseError};
use std::fmt;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) enum Error {
    // HttpClient(reqwest::Error),
    HttpClient(awc::error::SendRequestError),
    HttpPayload(awc::error::PayloadError),
    HttpServer(actix_web::Error),
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::error::Error::*;
        match self {
            HttpClient(err) => write!(f, "HttpClient({})", err),
            HttpPayload(err) => write!(f, "HttpPayload({})", err),
            HttpServer(err) => write!(f, "HttpServer({})", err),
            Io(err) => write!(f, "Io({})", err),
        }
    }
}

impl std::error::Error for Error {}

// impl From<reqwest::Error> for Error {
//     fn from(err: reqwest::Error) -> Self {
//         Error::HttpClient(err)
//     }
// }

impl From<actix_web::Error> for Error {
    fn from(err: actix_web::Error) -> Self {
        Error::HttpServer(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<awc::error::SendRequestError> for Error {
    fn from(err: awc::error::SendRequestError) -> Self {
        Error::HttpClient(err)
    }
}

impl From<awc::error::PayloadError> for Error {
    fn from(err: awc::error::PayloadError) -> Self {
        Error::HttpPayload(err)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().finish()
    }

    fn render_response(&self) -> HttpResponse {
        self.error_response()
    }
}
