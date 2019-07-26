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

#[derive(Deserialize)]
pub(crate) struct BitbucketApiResponse {
    values: Vec<BitbucketEntry>,
}

#[derive(Deserialize)]
pub(crate) struct BitbucketEntry {
    pub(crate) hash: String,
}

impl ApiResponse for BitbucketApiResponse {
    fn commit_ref(&self) -> &str {
        &self.values[0].hash
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

pub(crate) struct Bitbucket;

impl Service for Bitbucket {
    type Response = BitbucketApiResponse;

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
