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
    service::{GitHubApiResponse, Github, Service},
};
use actix_web::{
    http::header::{self, CacheControl, CacheDirective, Expires, LOCATION},
    middleware, web, App, Error as AError, HttpResponse, HttpServer,
};
use awc::{http::StatusCode, Client};
use futures::Future;
use std::time::{Duration, SystemTime};

fn proxy_file<T: Service>(
    client: web::Data<Client>,
    data: web::Path<FilePath>,
) -> Box<dyn Future<Item = HttpResponse, Error = AError>> {
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
) -> Box<dyn Future<Item = HttpResponse, Error = AError>> {
    Box::new(
        client
            .get(&T::api_url(&data))
            .header(header::USER_AGENT, statics::USER_AGENT.as_str())
            .send()
            .from_err()
            .and_then(move |mut response| match response.status() {
                StatusCode::OK => Box::new(
                    response
                        .json::<GitHubApiResponse>()
                        .map(move |resp| {
                            HttpResponse::SeeOther()
                                .header(
                                    LOCATION,
                                    T::redirect_url(&data.user, &data.repo, &resp.sha, &data.file)
                                        .as_str(),
                                )
                                .finish()
                        })
                        .from_err(),
                )
                    as Box<dyn Future<Item = HttpResponse, Error = AError>>,
                code => Box::new(futures::future::ok(HttpResponse::build(code).finish()))
                    as Box<dyn Future<Item = HttpResponse, Error = AError>>,
            }),
    )
}

fn handle_request<T: Service>(
    client: web::Data<Client>,
    data: web::Path<FilePath>,
) -> Box<dyn Future<Item = HttpResponse, Error = AError>> {
    if data.commit.len() == 40 {
        proxy_file::<T>(client, data)
    } else {
        redirect::<T>(client, data)
    }
}

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=trace");
    env_logger::init();

    Ok(HttpServer::new(move || {
        App::new()
            .data(Client::new())
            .wrap(middleware::Logger::default())
            .route(
                "/github/{user}/{repo}/{commit}/{file}",
                web::get().to_async(handle_request::<Github>),
            )
        // .default_service(web::resource("").route(web::get().to_async(p404)))
    })
    // .workers(OPT.workers)
    .bind("127.0.0.1:8080")?
    .run()?)
}
