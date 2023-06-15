use crate::service;

use serde::Deserialize;
use time_cache::Cache;
use tokio::sync::RwLock;

use std::sync::Arc;

pub(crate) type State = RwLock<Cache<Key, String>>;

#[derive(Deserialize, Debug)]
pub(crate) struct FilePath {
    pub(crate) user: Arc<String>,
    pub(crate) repo: Arc<String>,
    pub(crate) commit: Arc<String>,
    pub(crate) file: Arc<String>,
}

impl FilePath {
    pub(crate) fn path(&self) -> String {
        format!("{}/{}/{}/{}", self.user, self.repo, self.commit, self.file)
    }

    pub(crate) fn to_key<T: service::Service>(&self) -> Key {
        Key::new(
            T::cache_service(),
            Arc::clone(&self.user),
            Arc::clone(&self.repo),
            Arc::clone(&self.commit),
        )
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub(crate) struct Key(Service, Arc<String>, Arc<String>, Arc<String>);

#[derive(Eq, PartialEq, Hash, Debug)]
pub(crate) enum Service {
    GitHub,
    GitLab,
    Bitbucket,
}

impl Key {
    pub(crate) fn new(
        service: Service,
        user: Arc<String>,
        repo: Arc<String>,
        branch: Arc<String>,
    ) -> Self {
        Key(service, user, repo, branch)
    }
}
