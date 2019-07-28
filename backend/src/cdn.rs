use crate::{
    service::Service,
    statics::{self, CF_ZONE_IDENT},
};
use actix_web::{http::header, Error, HttpResponse};
use awc::Client;
use futures::Future;

pub(crate) struct Cloudflare;

impl Cloudflare {
    fn identifier() -> &'static str {
        &CF_ZONE_IDENT
    }

    pub(crate) fn dbg<T: Service>(
        client: &Client,
        file: &str,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        client
            .post(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
                Self::identifier()
            ))
            .header(header::USER_AGENT, statics::USER_AGENT.as_str())
            .header("X-Auth-Email", Self::auth_email())
            .header("X-Auth-Key", Self::auth_key())
            .content_type("application/json")
            .send_json(&CfPurgeRequest::singleton::<T>(file))
            .from_err()
            .and_then(|response| HttpResponse::build(response.status()).streaming(response))
    }

    pub(crate) fn purge_cache<T: Service>(
        client: &Client,
        file: &str,
    ) -> impl Future<Item = bool, Error = Error> {
        client
            .post(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
                Self::identifier()
            ))
            .header(header::USER_AGENT, statics::USER_AGENT.as_str())
            .header("X-Auth-Email", Self::auth_email())
            .header("X-Auth-Key", Self::auth_key())
            .content_type("application/json")
            .send_json(&CfPurgeRequest::singleton::<T>(file))
            .from_err()
            .and_then(|mut response| {
                response
                    .json::<CfPurgeResponse>()
                    .map(|resp| resp.success)
                    .from_err()
            })
    }

    fn auth_key() -> &'static str {
        &statics::CF_AUTH_KEY
    }

    fn auth_email() -> &'static str {
        &statics::CF_AUTH_USER
    }
}

#[derive(Serialize)]
struct CfPurgeRequest {
    files: Vec<String>,
}

impl CfPurgeRequest {
    fn singleton<T: Service>(file: &str) -> Self {
        let url = format!(
            "https://{}/{}/{}",
            statics::HOSTNAME.as_str(),
            T::path(),
            file
        );
        println!("{}", url);
        eprintln!("{}", url);
        Self { files: vec![url] }
    }
}

#[derive(Deserialize)]
struct CfPurgeResponse {
    success: bool,
}
