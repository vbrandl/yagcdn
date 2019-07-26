use crate::data::FilePath;

pub(crate) trait ApiResponse {
    fn commit_ref(&self) -> &str;
}

#[derive(Deserialize)]
pub(crate) struct GitHubApiResponse {
    pub(crate) sha: String,
}

impl ApiResponse for GitHubApiResponse {
    fn commit_ref(&self) -> &str {
        &self.sha
    }
}

pub(crate) trait Service {
    type Response: for<'de> serde::Deserialize<'de> + ApiResponse + 'static;
    fn raw_url(user: &str, repo: &str, commit: &str, file: &str) -> String;
    fn api_url(path: &FilePath) -> String;
    fn redirect_url(user: &str, repo: &str, commit: &str, file: &str) -> String;
}

pub(crate) struct Github;

impl Service for Github {
    type Response = GitHubApiResponse;

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
