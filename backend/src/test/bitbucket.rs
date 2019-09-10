use crate::{data::State, proxy_file, redirect, service::Bitbucket, REDIRECT_AGE};
use actix_web::{dev::Service, http::StatusCode, middleware, test, web, App};
use awc::Client;
use std::sync::{Arc, RwLock};
use time_cache::Cache;

#[test]
fn requesting_branch_redirects() {
    let state: State = Arc::new(RwLock::new(Cache::new(REDIRECT_AGE)));
    let mut app = test::init_service(
        App::new()
        .data(Client::new())
        .data(state)
        .wrap(middleware::NormalizePath)
        .route(
            "/bitbucket/{user}/{repo}/{commit}/{file:.*}",
            web::get().to_async(redirect::<Bitbucket>),
            ),
            );

    let req = test::TestRequest::with_uri("/bitbucket/vbrandl/vbrandl.net/master/README.md")
        .to_request();
    let resp = test::block_fn(|| app.call(req)).unwrap();

    assert_eq!(StatusCode::SEE_OTHER, resp.status());
}

#[test]
fn invalid_file_404() {
    let state: State = Arc::new(RwLock::new(Cache::new(REDIRECT_AGE)));
    let mut app = test::init_service(
        App::new()
        .data(Client::new())
        .data(state)
        .wrap(middleware::NormalizePath)
        .route(
            "/bitbucket/{user}/{repo}/{commit:[0-9a-fA-F]{40}}/{file:.*}",
            web::get().to_async(proxy_file::<Bitbucket>),
            ),
            );

    let req =
        test::TestRequest::with_uri("/bitbucket/vbrandl/vbrandl.net/369c392927a6d75f16c5dc38e2577276b94676bd/README.md.invalid")
        .to_request();
    let resp = test::block_fn(|| app.call(req)).unwrap();

    assert_eq!(StatusCode::NOT_FOUND, resp.status());
}

#[test]
fn valid_file_200() {
    let state: State = Arc::new(RwLock::new(Cache::new(REDIRECT_AGE)));
    let mut app = test::init_service(
        App::new()
        .data(Client::new())
        .data(state)
        .wrap(middleware::NormalizePath)
        .route(
            "/bitbucket/{user}/{repo}/{commit:[0-9a-fA-F]{40}}/{file:.*}",
            web::get().to_async(proxy_file::<Bitbucket>),
            ),
            );

    let req =
        test::TestRequest::with_uri("/bitbucket/vbrandl/vbrandl.net/369c392927a6d75f16c5dc38e2577276b94676bd/README.md")
        .to_request();
    let resp = test::block_fn(|| app.call(req)).unwrap();

    assert_eq!(StatusCode::OK, resp.status());
}
