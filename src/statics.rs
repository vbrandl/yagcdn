const VERSION: &str = env!("CARGO_PKG_VERSION");
lazy_static! {
    pub(crate) static ref USER_AGENT: String = format!("gitache/{}", VERSION);
}
