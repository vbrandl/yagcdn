use crate::{
    data::{self, FilePath, State},
    error::Result,
    statics::{self, load_env_var, GITHUB_AUTH_QUERY, OPT, REDIRECT_AGE_SECS},
};

use actix_web::{
    http::{
        header::{self, CacheControl, CacheDirective, LOCATION},
        StatusCode,
    },
    web, HttpResponse,
};
use awc::Client;
use serde::Deserialize;
use tracing::error;

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

#[async_trait::async_trait(?Send)]
pub(crate) trait Service: Sized {
    type Response: for<'de> serde::Deserialize<'de> + ApiResponse;

    fn raw_url(user: &str, repo: &str, commit: &str, file: &str) -> String;

    fn cache_service() -> data::Service;

    fn api_url(path: &FilePath) -> String;

    fn path() -> &'static str;

    fn api_accept() -> Option<&'static str> {
        None
    }

    fn redirect_url(user: &str, repo: &str, commit: &str, file: &str) -> String;

    async fn request_head(
        data: web::Path<FilePath>,
        cache: web::Data<State>,
        client: &Client,
    ) -> Result<HttpResponse> {
        let req = client
            .get(&Self::api_url(&data))
            .insert_header((header::USER_AGENT, statics::USER_AGENT.as_str()));
        let req = if let Some(accept) = Self::api_accept() {
            req.insert_header((header::ACCEPT, accept))
        } else {
            req
        };
        let mut response = req.send().await?;
        let status = response.status();
        Ok(match status {
            StatusCode::OK => {
                let resp = response.json::<Self::Response>().await?;
                let mut cache = cache.write().await;
                let key = data.to_key::<Self>();
                cache.store(key, resp.commit_ref().to_string());
                HttpResponse::SeeOther()
                    .insert_header((
                        LOCATION,
                        Self::redirect_url(&data.user, &data.repo, resp.commit_ref(), &data.file)
                            .as_str(),
                    ))
                    .insert_header(CacheControl(vec![
                        CacheDirective::Public,
                        CacheDirective::MaxAge(*REDIRECT_AGE_SECS),
                    ]))
                    .finish()
            }
            code => {
                error!(code = %code, "request failed");
                HttpResponse::build(code).finish()
            }
        })
    }
}

pub(crate) struct Github;

impl Github {
    pub(crate) fn auth_query() -> Option<Cow<'static, str>> {
        match (
            OPT.github_id
                .as_ref()
                .map(Cow::from)
                .or_else(|| load_env_var("GITHUB_CLIENT_ID")),
            OPT.github_secret
                .as_ref()
                .map(Cow::from)
                .or_else(|| load_env_var("GITHUB_CLIENT_SECRET")),
        ) {
            (Some(id), Some(secret)) => {
                Some(format!("?client_id={id}&client_secret={secret}").into())
            }
            _ => None,
        }
    }
}

#[async_trait::async_trait(?Send)]
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
        format!("https://raw.githubusercontent.com/{user}/{repo}/{commit}/{file}")
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
        format!("/github/{user}/{repo}/{commit}/{file}")
    }

    async fn request_head(
        data: web::Path<FilePath>,
        cache: web::Data<State>,
        client: &Client,
    ) -> Result<HttpResponse> {
        let req = client
            .get(&Self::api_url(&data))
            .insert_header((header::USER_AGENT, statics::USER_AGENT.as_str()));
        let req = if let Some(accept) = Self::api_accept() {
            req.insert_header((header::ACCEPT, accept))
        } else {
            req
        };
        let mut response = req.send().await?;
        let status = response.status();
        Ok(match status {
            StatusCode::OK => {
                let resp = response.body().await?;
                let head = String::from_utf8_lossy(resp.as_ref());
                let mut cache = cache.write().await;
                let key = data.to_key::<Self>();
                cache.store(key, head.to_string());
                HttpResponse::SeeOther()
                    .insert_header((
                        LOCATION,
                        Self::redirect_url(&data.user, &data.repo, &head, &data.file).as_str(),
                    ))
                    .insert_header(CacheControl(vec![
                        CacheDirective::Public,
                        CacheDirective::MaxAge(*REDIRECT_AGE_SECS),
                    ]))
                    .finish()
            }
            code => {
                error!(code = %code, "request failed");
                HttpResponse::build(code).finish()
            }
        })
    }
}

pub(crate) struct Bitbucket;

#[async_trait::async_trait(?Send)]
impl Service for Bitbucket {
    type Response = BitbucketApiResponse;

    fn cache_service() -> data::Service {
        data::Service::Bitbucket
    }

    fn path() -> &'static str {
        "bitbucket"
    }

    fn raw_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!("https://bitbucket.org/{user}/{repo}/raw/{commit}/{file}")
    }

    fn api_url(path: &FilePath) -> String {
        format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/commits/{}?pagelen=1",
            path.user, path.repo, path.commit
        )
    }

    fn redirect_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!("/bitbucket/{user}/{repo}/{commit}/{file}")
    }
}

pub(crate) struct GitLab;

#[async_trait::async_trait(?Send)]
impl Service for GitLab {
    type Response = GitLabApiResponse;

    fn cache_service() -> data::Service {
        data::Service::GitLab
    }

    fn path() -> &'static str {
        "gitlab"
    }

    fn raw_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!("https://gitlab.com/{user}/{repo}/raw/{commit}/{file}")
    }

    fn api_url(path: &FilePath) -> String {
        let repo_pattern = format!("{}/{}", path.user, path.repo).replace('/', "%2F");
        format!("https://gitlab.com/api/v4/projects/{repo_pattern}")
    }

    fn redirect_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!("/gitlab/{user}/{repo}/{commit}/{file}")
    }

    async fn request_head(
        data: web::Path<FilePath>,
        cache: web::Data<State>,
        client: &Client,
    ) -> Result<HttpResponse> {
        let req = client
            .get(&Self::api_url(&data))
            .insert_header((header::USER_AGENT, statics::USER_AGENT.as_str()));
        let req = if let Some(accept) = Self::api_accept() {
            req.insert_header((header::ACCEPT, accept))
        } else {
            req
        };
        let mut response = req.send().await?;
        Ok(match response.status() {
            StatusCode::OK => {
                let resp = response.json::<GitLabProject>().await?;
                let repo_id = resp.id;
                let mut respo = client
                    .get(format!(
                        "https://gitlab.com/api/v4/projects/{}/repository/branches/{}",
                        repo_id, data.commit
                    ))
                    .send()
                    .await?;
                match respo.status() {
                    StatusCode::OK => {
                        let resp = respo.json::<Self::Response>().await?;
                        let mut cache = cache.write().await;
                        let key = data.to_key::<Self>();
                        cache.store(key, resp.commit_ref().to_string());
                        HttpResponse::SeeOther()
                            .insert_header((
                                LOCATION,
                                Self::redirect_url(
                                    &data.user,
                                    &data.repo,
                                    resp.commit_ref(),
                                    &data.file,
                                )
                                .as_str(),
                            ))
                            .insert_header(CacheControl(vec![
                                CacheDirective::Public,
                                CacheDirective::MaxAge(*REDIRECT_AGE_SECS),
                            ]))
                            .finish()
                    }
                    code => {
                        error!(code = %code, "request failed");
                        HttpResponse::build(code).finish()
                    }
                }
            }
            code => {
                error!(code = %code, "request failed");
                HttpResponse::build(code).finish()
            }
        })
    }
}
