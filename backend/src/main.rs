#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

mod cache;
mod cdn;
mod config;
mod data;
mod error;
mod service;
mod statics;

use crate::{
    cdn::Cloudflare,
    data::{FilePath, State},
    error::Result,
    service::{Bitbucket, GitLab, Github, Service},
    statics::{FAVICON, OPT, REDIRECT_AGE},
};
use actix_files;
use actix_web::{
    http::header::{self, CacheControl, CacheDirective, LOCATION},
    middleware, web, App, Error, HttpResponse, HttpServer,
};
use awc::{http::StatusCode, Client};
use futures::Future;
use std::sync::{Arc, RwLock};
use time_cache::{Cache, CacheResult};

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
                    let mime = mime_guess::guess_mime_type(&*data.file);
                    Ok(HttpResponse::Ok()
                        .content_type(mime.to_string().as_str())
                        .set(CacheControl(vec![
                            CacheDirective::Public,
                            CacheDirective::MaxAge(2_592_000_000),
                        ]))
                        .streaming(response))
                }
                code => Ok(HttpResponse::build(code).finish()),
            }),
    )
}

fn redirect<T: Service>(
    client: web::Data<Client>,
    cache: web::Data<State>,
    data: web::Path<FilePath>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let invalid = {
        if let Ok(cache) = cache.read() {
            let key = data.to_key::<T>();
            match cache.get(&key) {
                CacheResult::Cached(head) => {
                    let head = head.clone();
                    return Box::new(futures::future::ok(()).map(move |_| {
                        HttpResponse::SeeOther()
                            .header(
                                LOCATION,
                                T::redirect_url(&data.user, &data.repo, &head, &data.file).as_str(),
                            )
                            .set(CacheControl(vec![
                                CacheDirective::Public,
                                CacheDirective::MaxAge(REDIRECT_AGE.as_secs() as u32),
                            ]))
                            .finish()
                    }));
                }
                CacheResult::Invalid => true,
                CacheResult::Empty => false,
            }
        } else {
            false
        }
    };
    if invalid {
        if let Ok(mut cache) = cache.write() {
            cache.clear();
        }
    }
    Box::new(
        client
            .get(&T::api_url(&data))
            .header(header::USER_AGENT, statics::USER_AGENT.as_str())
            .send()
            .from_err()
            .and_then(move |response| T::request_head(response, data, client, Arc::clone(&cache))),
    )
}

fn handle_request<T: Service>(
    client: web::Data<Client>,
    cache: web::Data<State>,
    data: web::Path<FilePath>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    if data.commit.len() == 40 {
        proxy_file::<T>(client, data)
    } else {
        redirect::<T>(client, cache, data)
    }
}

fn serve_gist(
    client: web::Data<Client>,
    data: web::Path<FilePath>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let url = format!(
        "https://gist.github.com/{}/{}/raw/{}/{}",
        data.user, data.repo, data.commit, data.file
    );
    client
        .get(url)
        .header(header::USER_AGENT, statics::USER_AGENT.as_str())
        .send()
        .from_err()
        .and_then(move |response| match response.status() {
            StatusCode::OK => {
                let mime = mime_guess::guess_mime_type(&*data.file);
                Ok(HttpResponse::Ok()
                    .content_type(mime.to_string().as_str())
                    .set(CacheControl(vec![
                        CacheDirective::Public,
                        CacheDirective::MaxAge(2_592_000_000),
                    ]))
                    .streaming(response))
            }
            code => Ok(HttpResponse::build(code).finish()),
        })
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

fn purge_cache<T: 'static + Service>(
    client: web::Data<Client>,
    cache: web::Data<State>,
    data: web::Path<FilePath>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    if data.commit.len() == 40 {
        Box::new(
            Cloudflare::purge_cache::<T>(&client, &data.path())
                .map(|success| HttpResponse::Ok().body(success.to_string())),
        )
    } else {
        let cache = cache.clone();
        Box::new(futures::future::ok(()).map(move |_| {
            if let Ok(mut cache) = cache.write() {
                let key = data.to_key::<T>();
                cache.invalidate(&key);
                HttpResponse::Ok().finish()
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }))
    }
}

fn dbg<T: 'static + Service>(
    client: web::Data<Client>,
    cache: web::Data<State>,
    data: web::Path<FilePath>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    if data.commit.len() == 40 {
        Box::new(Cloudflare::dbg::<T>(&client, &data.path()))
    } else {
        let cache = cache.clone();
        Box::new(futures::future::ok(()).map(move |_| {
            if let Ok(mut cache) = cache.write() {
                let key = data.to_key::<T>();
                cache.invalidate(&key);
                HttpResponse::Ok().finish()
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }))
    }
}

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=trace");
    pretty_env_logger::init();
    openssl_probe::init_ssl_cert_env_vars();

    let state: State = Arc::new(RwLock::new(Cache::new(REDIRECT_AGE)));
    Ok(HttpServer::new(move || {
        App::new()
            .data(Client::new())
            .data(state.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath)
            .service(favicon32)
            .route(
                "/github/{user}/{repo}/{commit}/{file:.*}",
                web::get().to_async(handle_request::<Github>),
            )
            .route(
                "/github/{user}/{repo}/{commit}/{file:.*}",
                web::delete().to_async(dbg::<Github>),
            )
            .route(
                "/bitbucket/{user}/{repo}/{commit}/{file:.*}",
                web::get().to_async(handle_request::<Bitbucket>),
            )
            .route(
                "/bitbucket/{user}/{repo}/{commit}/{file:.*}",
                web::delete().to_async(dbg::<Bitbucket>),
            )
            .route(
                "/gitlab/{user}/{repo}/{commit}/{file:.*}",
                web::get().to_async(handle_request::<GitLab>),
            )
            .route(
                "/gist/{user}/{repo}/{commit}/{file:.*}",
                web::get().to_async(serve_gist),
            )
            .route(
                "/gitlab/{user}/{repo}/{commit}/{file:.*}",
                web::delete().to_async(dbg::<GitLab>),
            )
            .service(actix_files::Files::new("/", "./public").index_file("index.html"))
    })
    .workers(OPT.workers)
    .bind((OPT.interface, OPT.port))?
    .run()?)
}
