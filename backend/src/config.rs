use clap::Parser;

use std::net::IpAddr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Opt {
    #[arg(short = 'p', long = "port", default_value = "8080")]
    /// Port to listen on
    pub(crate) port: u16,
    #[arg(short = 'i', long = "interface", default_value = "0.0.0.0")]
    /// Interface to listen on
    pub(crate) interface: IpAddr,
    #[arg(short = 'w', long = "workers", default_value = "4")]
    /// Number of worker threads
    pub(crate) workers: usize,
    #[arg(long = "gh-id")]
    /// GitHub OAuth client ID
    pub(crate) github_id: Option<String>,
    #[arg(long = "gh-secret")]
    /// GitHub OAuth client secret
    pub(crate) github_secret: Option<String>,
    #[arg(long = "cf-zone")]
    /// Cloudflare zone identifier
    pub(crate) cf_zone: Option<String>,
    #[arg(long = "cf-auth-key")]
    /// Cloudflare auth key
    pub(crate) cf_auth_key: Option<String>,
    #[arg(long = "cf-auth-user")]
    /// Cloudflare auth user
    pub(crate) cf_auth_user: Option<String>,
    #[arg(long = "hostname")]
    /// Hostname
    pub(crate) hostname: Option<String>,
}
