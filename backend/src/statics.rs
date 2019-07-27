use crate::{config::Opt, service::Github};
use structopt::StructOpt;

const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const FAVICON: &[u8] = include_bytes!("../static/favicon32.png");
lazy_static! {
    pub(crate) static ref USER_AGENT: String = format!("gitache/{}", VERSION);
    pub(crate) static ref OPT: Opt = Opt::from_args();
    pub(crate) static ref GITHUB_AUTH_QUERY: String = Github::auth_query().unwrap_or_default();
}
