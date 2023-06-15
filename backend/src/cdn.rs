use crate::{
    error::Result,
    service::Service,
    statics::{self, CF_ZONE_IDENT},
};

use actix_web::{http::header, HttpResponse};
use awc::Client;
use serde::Serialize;
use tracing::trace;

pub(crate) struct Cloudflare;

impl Cloudflare {
    fn identifier() -> &'static str {
        &CF_ZONE_IDENT
    }

    pub(crate) async fn purge_cache<T: Service>(
        client: &Client,
        file: &str,
    ) -> Result<HttpResponse> {
        let payload = CfPurgeRequest::singleton::<T>(file);
        trace!("{payload:#?}");
        let response = client
            .post(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
                Self::identifier()
            ))
            .insert_header((header::USER_AGENT, statics::USER_AGENT.as_str()))
            .insert_header(("X-Auth-Email", Self::auth_email()))
            .insert_header(("X-Auth-Key", Self::auth_key()))
            .content_type("application/json")
            .send_json(&payload)
            .await?;
        Ok(HttpResponse::build(response.status()).streaming(response))
    }

    fn auth_key() -> &'static str {
        &statics::CF_AUTH_KEY
    }

    fn auth_email() -> &'static str {
        &statics::CF_AUTH_USER
    }
}

#[derive(Serialize, Debug)]
struct CfPurgeRequest {
    files: Vec<String>,
}

impl CfPurgeRequest {
    fn singleton<T: Service>(file: &str) -> Self {
        Self {
            files: vec![format!(
                "https://{}/{}/{}",
                statics::HOSTNAME.as_ref(),
                T::path(),
                file
            )],
        }
    }
}
