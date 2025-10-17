use crate::{config::Opt, service::Github};

use clap::Parser;

use std::{borrow::Cow, env, sync::LazyLock, time::Duration};

const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const REDIRECT_AGE: Duration = Duration::from_secs(5 * 60);
pub(crate) const FAVICON: &[u8] = include_bytes!("../static/favicon32.png");
pub(crate) const REDIRECT_AGE_SECS: LazyLock<u32> =
    LazyLock::new(|| u32::try_from(REDIRECT_AGE.as_secs()).expect("redirect age to high"));
pub(crate) const USER_AGENT: LazyLock<String> = LazyLock::new(|| format!("yagcdn/{}", VERSION));
pub(crate) const OPT: LazyLock<Opt> = LazyLock::new(|| Opt::parse());
pub(crate) const GITHUB_AUTH_QUERY: LazyLock<Cow<'static, str>> =
    LazyLock::new(|| Github::auth_query().unwrap_or_default());
pub(crate) const CF_ZONE_IDENT: LazyLock<Cow<'static, str>> = LazyLock::new(|| {
    OPT.cf_zone
        .as_ref()
        .map(Cow::from)
        .or_else(|| load_env_var("CF_ZONE_IDENT"))
        .expect("Cloudflare zone identifier not set")
});
pub(crate) const CF_AUTH_KEY: LazyLock<Cow<'static, str>> = LazyLock::new(|| {
    OPT.cf_auth_key
        .as_ref()
        .map(Cow::from)
        .or_else(|| load_env_var("CF_AUTH_KEY"))
        .expect("Cloudflare auth key not set")
});
pub(crate) const CF_AUTH_USER: LazyLock<Cow<'static, str>> = LazyLock::new(|| {
    OPT.cf_auth_user
        .as_ref()
        .map(Cow::from)
        .or_else(|| load_env_var("CF_AUTH_USER"))
        .expect("Cloudflare auth user not set")
});
pub(crate) const HOSTNAME: LazyLock<Cow<'static, str>> = LazyLock::new(|| {
    OPT.hostname
        .as_ref()
        .map(Cow::from)
        .or_else(|| load_env_var("YAGCDN_HOSTNAME"))
        .unwrap_or_else(|| "yagcdn.tk".into())
});

pub(crate) fn load_env_var(key: &str) -> Option<Cow<'static, str>> {
    env::var(key).ok().and_then(|val| {
        if val.is_empty() {
            None
        } else {
            Some(val.into())
        }
    })
}
