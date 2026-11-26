use std::time::Duration;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Serialize;
use serde_json::json;
use ureq::{RequestBuilder, typestate::WithBody};

use super::{
    auth::{self, CLIENT_ID, DEVICE_TYPE, Grant},
    device::DeviceId,
    error::Error,
    item::{Draft, Sealed},
    kdf::Kdf,
    server::Server,
    sync,
};

#[derive(Clone)]
pub struct Client {
    agent: ureq::Agent,
}

pub struct EmailCode<'a> {
    pub email: &'a str,
    pub hash: &'a str,
    pub device: &'a DeviceId,
}

struct Reply {
    status: u16,
    body: String,
}

const TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RESPONSE_BYTES: u64 = 64 * 1024 * 1024;
const CLIENT_VERSION: &str = "2025.11.0";
const SUCCESS: std::ops::Range<u16> = 200..300;

impl Default for Client {
    fn default() -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .http_status_as_error(false)
            .build()
            .into();
        Self { agent }
    }
}

impl Reply {
    fn success(self) -> Result<String, Error> {
        match SUCCESS.contains(&self.status) {
            true => Ok(self.body),
            false => Err(Error::rejected(self.status, &self.body)),
        }
    }
}

fn read(response: Result<ureq::http::Response<ureq::Body>, ureq::Error>) -> Result<Reply, Error> {
    let response = response.map_err(Error::Network)?;
    let status = response.status().as_u16();
    let body = response
        .into_body()
        .with_config()
        .limit(MAX_RESPONSE_BYTES)
        .read_to_string()
        .map_err(Error::Network)?;
    Ok(Reply { status, body })
}

fn identified<Body>(request: RequestBuilder<Body>) -> RequestBuilder<Body> {
    request
        .header("Bitwarden-Client-Name", CLIENT_ID)
        .header("Bitwarden-Client-Version", CLIENT_VERSION)
        .header("Device-Type", DEVICE_TYPE)
}

fn authorized<Body>(request: RequestBuilder<Body>, access_token: &str) -> RequestBuilder<Body> {
    request.header("Authorization", format!("Bearer {access_token}"))
}

fn send_json(request: RequestBuilder<WithBody>, body: &impl Serialize) -> Result<Reply, Error> {
    let text = serde_json::to_string(body)?;
    read(request.content_type("application/json").send(text))
}

impl Client {
    fn post(&self, address: &str) -> RequestBuilder<WithBody> {
        identified(self.agent.post(address))
    }

    fn post_json(&self, address: &str, body: &serde_json::Value) -> Result<Reply, Error> {
        send_json(self.post(address), body)
    }

    pub fn prelogin(&self, server: &Server, email: &str) -> Result<Kdf, Error> {
        let address = format!("{}/accounts/prelogin", server.identity());
        let body = self
            .post_json(&address, &json!({ "email": email }))?
            .success()?;
        Kdf::from_prelogin(&body)
    }

    pub fn token(&self, server: &Server, grant: &Grant) -> Result<auth::Reply, Error> {
        let address = format!("{}/connect/token", server.identity());
        let request = match grant.email() {
            Some(email) => self
                .post(&address)
                .header("Auth-Email", URL_SAFE_NO_PAD.encode(email)),
            None => self.post(&address),
        };
        let reply = read(request.send_form(grant.form()))?;
        auth::Reply::parse(reply.status, &reply.body)
    }

    pub fn send_email_code(&self, server: &Server, request: &EmailCode) -> Result<(), Error> {
        let address = format!("{}/two-factor/send-email-login", server.api());
        let body = json!({
            "email": request.email,
            "masterPasswordHash": request.hash,
            "deviceIdentifier": request.device.as_str(),
        });
        self.post_json(&address, &body)?.success().map(drop)
    }

    pub fn sync(&self, server: &Server, access_token: &str) -> Result<sync::Reply, Error> {
        let address = format!("{}/sync?excludeDomains=true", server.api());
        let response = authorized(identified(self.agent.get(&address)), access_token).call();
        sync::Reply::parse(&read(response)?.success()?)
    }

    pub fn create(
        &self,
        server: &Server,
        access_token: &str,
        draft: &Draft,
    ) -> Result<Sealed, Error> {
        let address = format!("{}/ciphers", server.api());
        let request = authorized(self.post(&address), access_token);
        let body = send_json(request, draft)?.success()?;
        Ok(serde_json::from_str(&body)?)
    }
}
