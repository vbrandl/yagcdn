use crate::{
    data::{Key, Service, State},
    proxy_file, purge_local_cache, redirect,
    service::Github,
    REDIRECT_AGE,
};
use actix_web::{dev::Service as _, http::StatusCode, middleware, test, web, App};
use awc::Client;
use std::sync::{Arc, RwLock};
use time_cache::{Cache, CacheResult};

#[test]
fn requesting_branch_redirects() {
    let state: State = Arc::new(RwLock::new(Cache::new(REDIRECT_AGE)));
    let mut app = test::init_service(
        App::new()
            .data(Client::new())
            .data(state)
            .wrap(middleware::NormalizePath)
            .route(
                "/github/{user}/{repo}/{commit}/{file:.*}",
                web::get().to_async(redirect::<Github>),
            ),
    );

    let req = test::TestRequest::with_uri("/github/vbrandl/yagcdn/master/Cargo.toml").to_request();
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
                "/github/{user}/{repo}/{commit:[0-9a-fA-F]{40}}/{file:.*}",
                web::get().to_async(proxy_file::<Github>),
            ),
    );

    let req = test::TestRequest::with_uri(
        "/github/vbrandl/yagcdn/f1b35e7c05b952be6de559051d7daad2ecf05369/Cargo.toml.invalid",
    )
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
                "/github/{user}/{repo}/{commit:[0-9a-fA-F]{40}}/{file:.*}",
                web::get().to_async(proxy_file::<Github>),
            ),
    );
    // github
    let req = test::TestRequest::with_uri(
        "/github/vbrandl/yagcdn/f1b35e7c05b952be6de559051d7daad2ecf05369/backend/Cargo.toml",
    )
    .to_request();
    let resp = test::block_fn(|| app.call(req)).unwrap();

    assert_eq!(StatusCode::OK, resp.status());
}

#[test]
fn redirect_cache() {
    let state: State = Arc::new(RwLock::new(Cache::new(REDIRECT_AGE)));
    let mut app = test::init_service(
        App::new()
            .data(Client::new())
            .data(Arc::clone(&state))
            .wrap(middleware::NormalizePath)
            .route(
                "/github/{user}/{repo}/{commit}/{file:.*}",
                web::get().to_async(redirect::<Github>),
            )
            .route(
                "/github/{user}/{repo}/{commit}/{file:.*}",
                web::delete().to_async(purge_local_cache::<Github>),
            ),
    );

    let req = test::TestRequest::with_uri("/github/vbrandl/yagcdn/master/Cargo.toml").to_request();
    let resp = test::block_fn(|| app.call(req)).unwrap();
    assert_eq!(StatusCode::SEE_OTHER, resp.status());

    let key = Key::new(
        Service::GitHub,
        Arc::new("vbrandl".to_string()),
        Arc::new("yagcdn".to_string()),
        Arc::new("master".to_string()),
    );
    {
        let cache = state.read().unwrap();
        let res = cache.get(&key);
        assert_ne!(CacheResult::Empty, res);
        assert_ne!(CacheResult::Invalid, res);
    } // release the lock

    let req = test::TestRequest::delete()
        .uri("/github/vbrandl/yagcdn/master/Cargo.toml")
        .to_request();
    let resp = test::block_fn(|| app.call(req)).unwrap();
    assert_eq!(StatusCode::OK, resp.status());

    {
        let cache = state.read().unwrap();
        assert_eq!(CacheResult::Empty, cache.get(&key));
    } // release the lock
}
