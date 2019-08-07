use crate::{
    cache::{Cache, Key},
    service::Service,
};
use std::sync::{Arc, RwLock};

pub(crate) type State = Arc<RwLock<Cache<Key, String>>>;

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

    pub(crate) fn to_key<T: Service>(&self) -> Key {
        Key::new(
            T::cache_service(),
            self.user.clone(),
            self.repo.clone(),
            self.commit.clone(),
        )
    }
}
