use crate::data::FilePath;
// use actix_web::Error;
// use awc::Client;
// use futures::Future;
// use std::borrow::Cow;

#[derive(Deserialize)]
pub(crate) struct GitHubApiResponse {
    pub(crate) sha: String,
}

pub(crate) trait Service {
    fn raw_url(user: &str, repo: &str, commit: &str, file: &str) -> String;
    fn api_url(path: &FilePath) -> String;
    fn redirect_url(user: &str, repo: &str, commit: &str, file: &str) -> String;
}

pub(crate) struct Github;

impl Service for Github {
    fn raw_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            user, repo, commit, file
        )
    }

    fn api_url(path: &FilePath) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/commits/{}",
            path.user, path.repo, path.commit
        )
    }

    fn redirect_url(user: &str, repo: &str, commit: &str, file: &str) -> String {
        format!("/github/{}/{}/{}/{}", user, repo, commit, file)
    }
}
