use crate::{
    data::{self, FilePath, State},
    statics::{load_env_var, GITHUB_AUTH_QUERY, OPT, REDIRECT_AGE},
};
use actix_web::{
    http::{
        header::{CacheControl, CacheDirective, LOCATION},
        StatusCode,
    },
    web, Error, HttpResponse,
};
use awc::{error::PayloadError, Client, ClientResponse};
use bytes::Bytes;
use futures::{Future, Stream};
use std::borrow::Cow;

pub(crate) trait ApiResponse {
    fn commit_ref(&self) -> &str;
}

#[derive(Deserialize)]
pub(crate) struct GitHubApiResponse(String);

impl ApiResponse for GitHubApiResponse {
    fn commit_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Deserialize)]
pub(crate) struct BitbucketApiResponse {
    values: Vec<BitbucketEntry>,
}

#[derive(Deserialize)]
struct BitbucketEntry {
    hash: String,
}

impl ApiResponse for BitbucketApiResponse {
    fn commit_ref(&self) -> &str {
        &self.values[0].hash
    }
}

#[derive(Deserialize)]
struct GitLabProject {
    id: u64,
}

#[derive(Deserialize)]
pub(crate) struct GitLabApiResponse {
    commit: GitLabCommit,
}

#[derive(Deserialize)]
struct GitLabCommit {
    id: String,
}

impl ApiResponse for GitLabApiResponse {
    fn commit_ref(&self) -> &str {
        &self.commit.id
    }
}

pub(crate) trait Service: Sized {
    type Response: for<'de> serde::Deserialize<'de> + ApiResponse + 'static;

    fn raw_url(user: &str, repo: &str, commit: &str, file: &str) -> String;

    fn cache_service() -> data::Service;

    fn api_url(path: &FilePath) -> String;

    fn path() -> &'static str;

    fn api_accept() -> Option<&'static str> {
        None
    }

    fn redirect_url(user: &str, repo: &str, commit: &str, file: &str) -> String;

    fn request_head<S>(
        mut response: ClientResponse<S>,
        data: web::Path<FilePath>,
        _client: web::Data<Client>,
        cache: State,
    ) -> Box<dyn Future<Item = HttpResponse, Error = Error>>
    where
        S: 'static + Stream<Item = Bytes, Error = PayloadError>,
    {
        match response.status() {
            StatusCode::OK => Box::new(
                response
                    .json::<Self::Response>()
                    .map(move |resp| {
                        if let Ok(mut cache) = cache.write() {
                            let key = data.to_key::<Self>();
                            cache.store(key, resp.commit_ref().to_string());
                        }
                        HttpResponse::SeeOther()
                            .header(
                                LOCATION,
                                Self::redirect_url(
                                    &data.user,
                                    &data.repo,
                                    resp.commit_ref(),
                                    &data.file,
                                )
                                .as_str(),
                            )
                            .set(CacheControl(vec![
                                CacheDirective::Public,
                                CacheDirective::MaxAge(REDIRECT_AGE.as_secs() as u32),
                            ]))
                            .finish()
                    })
                    .from_err(),
            ) as Box<dyn Future<Item = HttpResponse, Error = Error>>,
            code => Box::new(futures::future::ok(HttpResponse::build(code).finish()))
                as Box<dyn Future<Item = HttpResponse, Error = Error>>,
        }
    }
}

pub(crate) struct Github;

impl Github {
    pub(crate) fn auth_query() -> Option<Cow<'static, str>> {
        OPT.github_id
            .as_ref()
            .map(Cow::from)
            .or_else(|| load_env_var("GITHUB_CLIENT_ID"))
            .and_then(|id| {
                OPT.github_secret
                    .as_ref()
                    .map(Cow::from)
                    .or_else(|| load_env_var("GITHUB_CLIENT_SECRET"))
                    .map(|secret| format!("?client_id={}&client_secret={}", id, secret).into())
            })
    }
}

impl Service for Github {
    type Response = GitHubApiResponse;

    fn cache_service() -> data::Service {
        data::Service::GitHub
    }

    fn path() -> &'static str {
        "github"
    }

    fn api_accept() -> Option<&'static str> {
        Some("application/vnd.github.3.sha")
    }

    fn raw_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            user, repo, commit, file
        )
    }

    fn api_url(path: &FilePath) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/commits/{}{}",
            path.user,
            path.repo,
            path.commit,
            GITHUB_AUTH_QUERY.as_ref()
        )
    }

    fn redirect_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!("/github/{}/{}/{}/{}", user, repo, commit, file)
    }

    fn request_head<S>(
        mut response: ClientResponse<S>,
        data: web::Path<FilePath>,
        _client: web::Data<Client>,
        cache: State,
    ) -> Box<dyn Future<Item = HttpResponse, Error = Error>>
    where
        S: 'static + Stream<Item = Bytes, Error = PayloadError>,
    {
        match response.status() {
            StatusCode::OK => Box::new(
                response
                    .body()
                    // .json::<Self::Response>()
                    .map(move |resp| {
                        let head = String::from_utf8_lossy(resp.as_ref());
                        if let Ok(mut cache) = cache.write() {
                            let key = data.to_key::<Self>();
                            cache.store(key, head.to_string());
                        }
                        HttpResponse::SeeOther()
                            .header(
                                LOCATION,
                                Self::redirect_url(&data.user, &data.repo, &head, &data.file)
                                    .as_str(),
                            )
                            .set(CacheControl(vec![
                                CacheDirective::Public,
                                CacheDirective::MaxAge(REDIRECT_AGE.as_secs() as u32),
                            ]))
                            .finish()
                    })
                    .from_err(),
            ) as Box<dyn Future<Item = HttpResponse, Error = Error>>,
            code => Box::new(futures::future::ok(HttpResponse::build(code).finish()))
                as Box<dyn Future<Item = HttpResponse, Error = Error>>,
        }
    }
}

pub(crate) struct Bitbucket;

impl Service for Bitbucket {
    type Response = BitbucketApiResponse;

    fn cache_service() -> data::Service {
        data::Service::Bitbucket
    }

    fn path() -> &'static str {
        "bitbucket"
    }

    fn raw_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!(
            "https://bitbucket.org/{}/{}/raw/{}/{}",
            user, repo, commit, file
        )
    }

    fn api_url(path: &FilePath) -> String {
        format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/commits/{}?pagelen=1",
            path.user, path.repo, path.commit
        )
    }

    fn redirect_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!("/bitbucket/{}/{}/{}/{}", user, repo, commit, file)
    }
}

pub(crate) struct GitLab;

impl Service for GitLab {
    type Response = GitLabApiResponse;

    fn cache_service() -> data::Service {
        data::Service::GitLab
    }

    fn path() -> &'static str {
        "gitlab"
    }

    fn raw_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!(
            "https://gitlab.com/{}/{}/raw/{}/{}",
            user, repo, commit, file
        )
    }

    fn api_url(path: &FilePath) -> String {
        let repo_pattern = format!("{}/{}", path.user, path.repo).replace("/", "%2F");
        format!("https://gitlab.com/api/v4/projects/{}", repo_pattern)
    }

    fn redirect_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!("/gitlab/{}/{}/{}/{}", user, repo, commit, file)
    }

    fn request_head<S>(
        mut response: ClientResponse<S>,
        data: web::Path<FilePath>,
        client: web::Data<Client>,
        cache: State,
    ) -> Box<dyn Future<Item = HttpResponse, Error = Error>>
    where
        S: 'static + Stream<Item = Bytes, Error = PayloadError>,
    {
        match response.status() {
            StatusCode::OK => Box::new(
                response
                    .json::<GitLabProject>()
                    .map(move |resp| resp.id)
                    .from_err()
                    .and_then(move |repo_id| {
                        client
                            .get(format!(
                                "https://gitlab.com/api/v4/projects/{}/repository/branches/{}",
                                repo_id, data.commit
                            ))
                            .send()
                            .from_err()
                            .and_then(|mut respo| match respo.status() {
                                StatusCode::OK => Box::new(
                                    respo
                                        .json::<Self::Response>()
                                        .map(move |resp| {
                                            if let Ok(mut cache) = cache.write() {
                                                let key = data.to_key::<Self>();
                                                cache.store(key, resp.commit_ref().to_string());
                                            }
                                            HttpResponse::SeeOther()
                                                .header(
                                                    LOCATION,
                                                    Self::redirect_url(
                                                        &data.user,
                                                        &data.repo,
                                                        resp.commit_ref(),
                                                        &data.file,
                                                    )
                                                    .as_str(),
                                                )
                                                .set(CacheControl(vec![
                                                    CacheDirective::Public,
                                                    CacheDirective::MaxAge(
                                                        REDIRECT_AGE.as_secs() as u32
                                                    ),
                                                ]))
                                                .finish()
                                        })
                                        .from_err(),
                                )
                                    as Box<dyn Future<Item = HttpResponse, Error = Error>>,
                                code => Box::new(futures::future::ok(
                                    HttpResponse::build(code).finish(),
                                ))
                                    as Box<dyn Future<Item = HttpResponse, Error = Error>>,
                            })
                            .from_err()
                    }),
            ) as Box<dyn Future<Item = HttpResponse, Error = Error>>,
            code => Box::new(futures::future::ok(HttpResponse::build(code).finish()))
                as Box<dyn Future<Item = HttpResponse, Error = Error>>,
        }
    }
}
