use crate::{config::Opt, service::Github};
use std::env;
use structopt::StructOpt;

const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const FAVICON: &[u8] = include_bytes!("../static/favicon32.png");
lazy_static! {
    pub(crate) static ref USER_AGENT: String = format!("gitache/{}", VERSION);
    pub(crate) static ref OPT: Opt = Opt::from_args();
    pub(crate) static ref GITHUB_AUTH_QUERY: String = Github::auth_query().unwrap_or_default();
    pub(crate) static ref CF_ZONE_IDENT: String = OPT
        .cf_zone
        .clone()
        .or_else(|| env::var("CF_ZONE_IDENT").ok())
        .expect("Cloudflare zone identifier not set");
    pub(crate) static ref CF_AUTH_KEY: String = OPT
        .cf_auth_key
        .clone()
        .or_else(|| env::var("CF_AUTH_KEY").ok())
        .expect("Cloudflare auth key not set");
    pub(crate) static ref CF_AUTH_USER: String = OPT
        .cf_auth_user
        .clone()
        .or_else(|| env::var("CF_AUTH_USER").ok())
        .expect("Cloudflare auth user not set");
    pub(crate) static ref HOSTNAME: String = OPT
        .hostname
        .clone()
        .or_else(|| env::var("GITACHE_HOSTNAME").ok())
        .unwrap_or_else(|| "gitcdn.tk".to_string());
}
