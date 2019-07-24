#[derive(Deserialize, Debug)]
pub(crate) struct FilePath {
    pub(crate) user: String,
    pub(crate) repo: String,
    pub(crate) commit: String,
    pub(crate) file: String,
}

