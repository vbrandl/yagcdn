use crate::{config::Opt, service::Github};
use std::{borrow::Cow, env, time::Duration};
use structopt::StructOpt;

const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const REDIRECT_AGE: Duration = Duration::from_secs(5 * 60);
pub(crate) const FAVICON: &[u8] = include_bytes!("../static/favicon32.png");
lazy_static! {
    pub(crate) static ref USER_AGENT: String = format!("yagcdn/{}", VERSION);
    pub(crate) static ref OPT: Opt = Opt::from_args();
    pub(crate) static ref GITHUB_AUTH_QUERY: Cow<'static, str> =
        Github::auth_query().unwrap_or_default().into();
    pub(crate) static ref CF_ZONE_IDENT: Cow<'static, str> = OPT
        .cf_zone
        .as_ref()
        .map(Cow::from)
        .or_else(|| load_env_var("CF_ZONE_IDENT"))
        .expect("Cloudflare zone identifier not set")
        .into();
    pub(crate) static ref CF_AUTH_KEY: Cow<'static, str> = OPT
        .cf_auth_key
        .as_ref()
        .map(Cow::from)
        .or_else(|| load_env_var("CF_AUTH_KEY"))
        .expect("Cloudflare auth key not set")
        .into();
    pub(crate) static ref CF_AUTH_USER: Cow<'static, str> = OPT
        .cf_auth_user
        .as_ref()
        .map(Cow::from)
        .or_else(|| load_env_var("CF_AUTH_USER"))
        .expect("Cloudflare auth user not set")
        .into();
    pub(crate) static ref HOSTNAME: Cow<'static, str> = OPT
        .hostname
        .as_ref()
        .map(Cow::from)
        .or_else(|| load_env_var("YAGCDN_HOSTNAME"))
        .unwrap_or_else(|| "yagcdn.tk".into());
}

pub(crate) fn load_env_var(key: &str) -> Option<Cow<'static, str>> {
    env::var(key).ok().and_then(|val| {
        if val.is_empty() {
            None
        } else {
            Some(val.into())
        }
    })
}
