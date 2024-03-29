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
    dev::Service as _,
    get,
    http::header::{self, CacheControl, CacheDirective, HeaderName, HeaderValue, LOCATION},
    middleware, web, App, HttpMessage, HttpResponse, HttpServer, Responder,
};
use awc::{http::StatusCode, Client};
use time_cache::{Cache, CacheResult};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument};
use tracing_actix_web::{RequestId, TracingLogger};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[instrument(skip(data, client), fields(path = data.path(), service = T::path()))]
async fn proxy_file<T: Service>(
    client: web::Data<Client>,
    data: web::Path<FilePath>,
) -> Result<impl Responder> {
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
            // mime type is guessed from the file extension
            let mime = mime_guess::from_path(&*data.file).first_or_octet_stream();
            info!(mime = %mime, "proxying file");
            Ok(HttpResponse::Ok()
                .content_type(mime.as_ref())
                .insert_header(CacheControl(vec![
                    CacheDirective::Public,
                    CacheDirective::MaxAge(2_592_000_000),
                ]))
                .streaming(response))
        }
        code => {
            error!(code = %code, "error from remote");
            Ok(HttpResponse::build(code).finish())
        }
    }
}

#[instrument(skip(cache, data, client), fields(path = data.path(), service = T::path()))]
async fn redirect<T: Service>(
    cache: web::Data<State>,
    client: web::Data<Client>,
    data: web::Path<FilePath>,
) -> Result<impl Responder> {
    let invalid = {
        let cache = cache.read().await;
        let key = data.to_key::<T>();
        match cache.get(&key) {
            CacheResult::Cached(head) => {
                debug!("Loading HEAD from cache");
                return Ok(HttpResponse::SeeOther()
                    .insert_header((
                        LOCATION,
                        T::redirect_url(&data.user, &data.repo, head, &data.file).as_str(),
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
        debug!("Clearing cache. Removing invalid elements");
        cache.clear();
    }
    info!("Redirecting");
    T::request_head(data, cache, &client).await
}

#[instrument(skip(data, client), fields(path = data.path(), service = "gist"))]
async fn serve_gist(client: web::Data<Client>, data: web::Path<FilePath>) -> Result<HttpResponse> {
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
            // mime type is guessed from the file extension
            let mime = mime_guess::from_path(&*data.file).first_or_octet_stream();
            Ok(HttpResponse::Ok()
                .content_type(mime.as_ref())
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
#[instrument]
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
#[instrument(skip(cache, data), fields(path = data.path(), service = T::path()))]
async fn purge_local_cache<T: Service>(
    cache: web::Data<State>,
    data: web::Path<FilePath>,
) -> HttpResponse {
    let mut cache = cache.write().await;
    info!("Invalidating local cache");
    let key = data.to_key::<T>();
    cache.invalidate(&key);
    HttpResponse::Ok().finish()
}

#[instrument(skip(data, client), fields(path = data.path(), service = T::path()))]
async fn purge_cf_cache<T: Service>(
    client: web::Data<Client>,
    data: web::Path<FilePath>,
) -> Result<HttpResponse> {
    info!("purging cache");
    Cloudflare::purge_cache::<T>(&client, &data.path()).await
}

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "yagcdn=debug,actix_web=trace,tracing_actix_web=debug".into()),
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
            // set the request id in the `x-request-id` response header
            .wrap_fn(|req, srv| {
                let request_id = req.extensions().get::<RequestId>().copied();
                let res = srv.call(req);
                async move {
                    let mut res = res.await?;
                    if let Some(request_id) = request_id {
                        res.headers_mut().insert(
                            HeaderName::from_static("x-request-id"),
                            // this unwrap never fails, since UUIDs are valid ASCII strings
                            HeaderValue::from_str(&request_id.to_string()).unwrap(),
                        );
                    }
                    Ok(res)
                }
            })
            .app_data(state.clone())
            .app_data(web::Data::new(Client::default()))
            .wrap(TracingLogger::default())
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
