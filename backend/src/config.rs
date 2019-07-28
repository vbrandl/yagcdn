use std::net::IpAddr;

#[derive(StructOpt)]
pub(crate) struct Opt {
    #[structopt(short = "p", long = "port", default_value = "8080")]
    /// Port to listen on
    pub(crate) port: u16,
    #[structopt(short = "i", long = "interface", default_value = "0.0.0.0")]
    /// Interface to listen on
    pub(crate) interface: IpAddr,
    #[structopt(short = "w", long = "workers", default_value = "4")]
    /// Number of worker threads
    pub(crate) workers: usize,
    #[structopt(long = "gh-id")]
    /// GitHub OAuth client ID
    pub(crate) github_id: Option<String>,
    #[structopt(long = "gh-secret")]
    /// GitHub OAuth client secret
    pub(crate) github_secret: Option<String>,
    #[structopt(long = "cf-zone")]
    /// Cloudflare zone identifier
    pub(crate) cf_zone: Option<String>,
    #[structopt(long = "cf-auth-key")]
    /// Cloudflare auth key
    pub(crate) cf_auth_key: Option<String>,
    #[structopt(long = "cf-auth-user")]
    /// Cloudflare auth user
    pub(crate) cf_auth_user: Option<String>,
    #[structopt(long = "hostname")]
    /// Hostname
    pub(crate) hostname: Option<String>,
}
