#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod data;
mod error;
mod service;
mod statics;

use crate::{
    data::FilePath,
    error::Result,
    service::{Bitbucket, GitLab, Github, Service},
    statics::FAVICON,
};
use actix_web::{
    http::header::{self, CacheControl, CacheDirective, Expires},
    middleware, web, App, Error, HttpResponse, HttpServer,
};
use awc::{http::StatusCode, Client};
use futures::Future;
use std::time::{Duration, SystemTime};

fn proxy_file<T: Service>(
    client: web::Data<Client>,
    data: web::Path<FilePath>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        client
            .get(&T::raw_url(
                &data.user,
                &data.repo,
                &data.commit,
                &data.file,
            ))
            .header(header::USER_AGENT, statics::USER_AGENT.as_str())
            .send()
            .from_err()
            .and_then(move |response| match response.status() {
                StatusCode::OK => {
                    let mime = mime_guess::guess_mime_type(&data.file);
                    let expiration = SystemTime::now() + Duration::from_secs(2_592_000_000);
                    Ok(HttpResponse::Ok()
                        .content_type(mime.to_string().as_str())
                        .set(Expires(expiration.into()))
                        .set(CacheControl(vec![
                            CacheDirective::MaxAge(2_592_000_000),
                            CacheDirective::Public,
                        ]))
                        .streaming(response))
                }
                code => Ok(HttpResponse::build(code).finish()),
            }),
    )
}

fn redirect<T: Service>(
    client: web::Data<Client>,
    data: web::Path<FilePath>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        client
            .get(&T::api_url(&data))
            .header(header::USER_AGENT, statics::USER_AGENT.as_str())
            .send()
            .from_err()
            .and_then(move |response| T::request_head(response, data, client)),
    )
}

fn handle_request<T: Service>(
    client: web::Data<Client>,
    data: web::Path<FilePath>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    if data.commit.len() == 40 {
        proxy_file::<T>(client, data)
    } else {
        redirect::<T>(client, data)
    }
}

#[get("/favicon.ico")]
fn favicon32() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/png")
        .set(CacheControl(vec![
            CacheDirective::Public,
            CacheDirective::MaxAge(2_592_000_000),
        ]))
        .body(FAVICON)
}

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=trace");
    pretty_env_logger::init();
    openssl_probe::init_ssl_cert_env_vars();

    Ok(HttpServer::new(move || {
        App::new()
            .data(Client::new())
            .wrap(middleware::Logger::default())
            .service(favicon32)
            .route(
                "/github/{user}/{repo}/{commit}/{file:.*}",
                web::get().to_async(handle_request::<Github>),
            )
            .route(
                "/bitbucket/{user}/{repo}/{commit}/{file:.*}",
                web::get().to_async(handle_request::<Bitbucket>),
            )
            .route(
                "/gitlab/{user}/{repo}/{commit}/{file:.*}",
                web::get().to_async(handle_request::<GitLab>),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()?)
}
