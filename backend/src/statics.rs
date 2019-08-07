use crate::{config::Opt, service::Github};
use std::{env, time::Duration};
use structopt::StructOpt;

const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const REDIRECT_AGE: Duration = Duration::from_secs(5 * 60);
pub(crate) const FAVICON: &[u8] = include_bytes!("../static/favicon32.png");
lazy_static! {
    pub(crate) static ref USER_AGENT: String = format!("gitache/{}", VERSION);
    pub(crate) static ref OPT: Opt = Opt::from_args();
    pub(crate) static ref GITHUB_AUTH_QUERY: String = Github::auth_query().unwrap_or_default();
    pub(crate) static ref CF_ZONE_IDENT: String = OPT
        .cf_zone
        .clone()
        .or_else(|| load_env_var("CF_ZONE_IDENT"))
        .expect("Cloudflare zone identifier not set");
    pub(crate) static ref CF_AUTH_KEY: String = OPT
        .cf_auth_key
        .clone()
        .or_else(|| load_env_var("CF_AUTH_KEY"))
        .expect("Cloudflare auth key not set");
    pub(crate) static ref CF_AUTH_USER: String = OPT
        .cf_auth_user
        .clone()
        .or_else(|| load_env_var("CF_AUTH_USER"))
        .expect("Cloudflare auth user not set");
    pub(crate) static ref HOSTNAME: String = OPT
        .hostname
        .clone()
        .or_else(|| load_env_var("GITACHE_HOSTNAME"))
        .unwrap_or_else(|| "gitcdn.tk".to_string());
}

pub(crate) fn load_env_var(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .and_then(|val| if val.is_empty() { None } else { Some(val) })
}
