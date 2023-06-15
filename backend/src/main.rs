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
    statics::{FAVICON, OPT, REDIRECT_AGE, REDIRECT_AGE_SECS},
};

use actix_web::{
    get,
    http::header::{self, CacheControl, CacheDirective, LOCATION},
    middleware, web, App, HttpResponse, HttpServer,
};
use awc::{http::StatusCode, Client};
use time_cache::{Cache, CacheResult};
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn proxy_file<T: Service>(data: web::Path<FilePath>) -> Result<HttpResponse> {
    let client = Client::default();
    let response = client
        .get(&T::raw_url(
            &data.user,
            &data.repo,
            &data.commit,
            &data.file,
        ))
        .insert_header((header::USER_AGENT, statics::USER_AGENT.as_str()))
        .send()
        .await?;
    match response.status() {
        StatusCode::OK => {
            let mime = mime_guess::from_path(&*data.file).first_or_octet_stream();
            Ok(HttpResponse::Ok()
                .content_type(mime.to_string().as_str())
                .insert_header(CacheControl(vec![
                    CacheDirective::Public,
                    CacheDirective::MaxAge(2_592_000_000),
                ]))
                .streaming(response))
        }
        code => Ok(HttpResponse::build(code).finish()),
    }
}

async fn redirect<T: Service>(
    cache: web::Data<State>,
    data: web::Path<FilePath>,
) -> Result<HttpResponse> {
    let invalid = {
        let cache = cache.read().await;
        let key = data.to_key::<T>();
        match cache.get(&key) {
            CacheResult::Cached(head) => {
                info!("Loading HEAD from cache for {}/{}", T::path(), data.path());
                let head = head.clone();
                return Ok(HttpResponse::SeeOther()
                    .insert_header((
                        LOCATION,
                        T::redirect_url(&data.user, &data.repo, &head, &data.file).as_str(),
                    ))
                    .insert_header(CacheControl(vec![
                        CacheDirective::Public,
                        CacheDirective::MaxAge(*REDIRECT_AGE_SECS),
                    ]))
                    .finish());
            }
            CacheResult::Invalid => true,
            CacheResult::Empty => false,
        }
    };
    if invalid {
        let mut cache = cache.write().await;
        info!("Clearing cache. Removing invalid elements");
        cache.clear();
    }
    T::request_head(data, cache).await
}

async fn serve_gist(data: web::Path<FilePath>) -> Result<HttpResponse> {
    let client = Client::default();
    let url = format!(
        "https://gist.github.com/{}/{}/raw/{}/{}",
        data.user, data.repo, data.commit, data.file
    );
    let response = client
        .get(url)
        .insert_header((header::USER_AGENT, statics::USER_AGENT.as_str()))
        .send()
        .await?;
    match response.status() {
        StatusCode::OK => {
            let mime = mime_guess::from_path(&*data.file).first_or_octet_stream();
            Ok(HttpResponse::Ok()
                .content_type(mime.to_string().as_str())
                .insert_header(CacheControl(vec![
                    CacheDirective::Public,
                    CacheDirective::MaxAge(2_592_000_000),
                ]))
                .streaming(response))
        }
        code => Ok(HttpResponse::build(code).finish()),
    }
}

#[get("/favicon.ico")]
#[allow(clippy::unused_async)]
async fn favicon32() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/png")
        .insert_header(CacheControl(vec![
            CacheDirective::Public,
            CacheDirective::MaxAge(2_592_000_000),
        ]))
        .body(FAVICON)
}

#[allow(clippy::unused_async)]
async fn purge_local_cache<T: 'static + Service>(
    cache: web::Data<State>,
    data: web::Path<FilePath>,
) -> HttpResponse {
    let mut cache = cache.write().await;
    info!("Invalidating local cache for {}/{}", T::path(), data.path());
    let key = data.to_key::<T>();
    cache.invalidate(&key);
    HttpResponse::Ok().finish()
}

async fn purge_cf_cache<T: 'static + Service>(data: web::Path<FilePath>) -> Result<HttpResponse> {
    let client = Client::default();
    Cloudflare::purge_cache::<T>(&client, &data.path()).await
}

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "yagcdn=debug,actix_server=info,actix_web=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[actix_web::main]
async fn main() -> Result<()> {
    init_logging();

    let state = web::Data::new(RwLock::new(Cache::<data::Key, String>::new(REDIRECT_AGE)));
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .service(favicon32)
            .route(
                "/github/{user}/{repo}/{commit:[0-9a-fA-F]{40}}/{file:.*}",
                web::get().to(proxy_file::<Github>),
            )
            .route(
                "/github/{user}/{repo}/{commit}/{file:.*}",
                web::get().to(redirect::<Github>),
            )
            .route(
                "/github/{user}/{repo}/{commit:[0-9a-fA-F]{40}}/{file:.*}",
                web::delete().to(purge_cf_cache::<Github>),
            )
            .route(
                "/github/{user}/{repo}/{commit}/{file:.*}",
                web::delete().to(purge_local_cache::<Github>),
            )
            .route(
                "/bitbucket/{user}/{repo}/{commit:[0-9a-fA-F]{40}}/{file:.*}",
                web::get().to(proxy_file::<Bitbucket>),
            )
            .route(
                "/bitbucket/{user}/{repo}/{commit}/{file:.*}",
                web::get().to(redirect::<Bitbucket>),
            )
            .route(
                "/bitbucket/{user}/{repo}/{commit:[0-9a-fA-F]{40}}/{file:.*}",
                web::delete().to(purge_cf_cache::<Bitbucket>),
            )
            .route(
                "/bitbucket/{user}/{repo}/{commit}/{file:.*}",
                web::delete().to(purge_local_cache::<Bitbucket>),
            )
            .route(
                "/gitlab/{user}/{repo}/{commit:[0-9a-fA-F]{40}}/{file:.*}",
                web::get().to(proxy_file::<GitLab>),
            )
            .route(
                "/gitlab/{user}/{repo}/{commit}/{file:.*}",
                web::get().to(redirect::<GitLab>),
            )
            .route(
                "/gitlab/{user}/{repo}/{commit:[0-9a-fA-F]{40}}/{file:.*}",
                web::delete().to(purge_cf_cache::<GitLab>),
            )
            .route(
                "/gitlab/{user}/{repo}/{commit}/{file:.*}",
                web::delete().to(purge_cf_cache::<GitLab>),
            )
            .route(
                "/gist/{user}/{repo}/{commit}/{file:.*}",
                web::get().to(serve_gist),
            )
            .service(actix_files::Files::new("/", "./public").index_file("index.html"))
    })
    .workers(OPT.workers)
    .bind((OPT.interface, OPT.port))?
    .run()
    .await?)
}
