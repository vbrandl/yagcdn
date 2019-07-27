const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const FAVICON: &[u8] = include_bytes!("../static/favicon32.png");
lazy_static! {
    pub(crate) static ref USER_AGENT: String = format!("gitache/{}", VERSION);
}
