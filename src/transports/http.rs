use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::errors::Result;
use crate::transports::{BatchTransport, Transport};
use crate::types::{Call, MethodCall, Params, Request, RequestId, Response, Version};

/// HTTP transport
#[derive(Clone)]
pub struct HttpTransport {
    id: Arc<AtomicUsize>,
    url: String,
    bearer_auth_token: Option<String>,
    basic_auth_username: Option<String>,
    basic_auth_password: Option<String>,
    client: reqwest::Client,
}

impl HttpTransport {
    fn new_client() -> reqwest::Client {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("ClientBuilder config is valid; qed")
    }

    /// Create a new HTTP transport with given `url`.
    pub fn new<U: Into<String>>(url: U) -> Self {
        Self {
            id: Default::default(),
            url: url.into(),
            bearer_auth_token: None,
            basic_auth_username: None,
            basic_auth_password: None,
            client: Self::new_client(),
        }
    }

    /// Create a new HTTP transport with given `url` and bearer `token`.
    pub fn new_with_bearer_auth<U: Into<String>, T: Into<String>>(url: U, token: T) -> Self {
        Self {
            id: Default::default(),
            url: url.into(),
            bearer_auth_token: Some(token.into()),
            basic_auth_username: None,
            basic_auth_password: None,
            client: Self::new_client(),
        }
    }

    /// Create a new HTTP transport with given `url` and basic auth
    pub fn new_with_basic_auth<U: Into<String>, T: Into<String>>(
        url: U,
        username: T,
        password: T,
    ) -> Self {
        Self {
            id: Default::default(),
            url: url.into(),
            bearer_auth_token: None,
            basic_auth_username: Some(username.into()),
            basic_auth_password: Some(password.into()),
            client: Self::new_client(),
        }
    }

    async fn send_request(&self, request: &Request) -> Result<Response> {
        let builder = self.client.post(&self.url).json(request);
        let builder = if let Some(token) = &self.bearer_auth_token {
            builder.bearer_auth(token)
        } else {
            builder
        };

        let builder = if let Some(username) = &self.basic_auth_username {
            builder.basic_auth(username, self.basic_auth_password.clone())
        } else {
            builder
        };

        Ok(builder.send().await?.json().await?)
    }
}

#[async_trait::async_trait]
impl Transport for HttpTransport {
    fn prepare<M: Into<String>>(&self, method: M, params: Params) -> (RequestId, Call) {
        let id = self.id.fetch_add(1, Ordering::AcqRel);
        let call = Call::MethodCall(MethodCall {
            jsonrpc: Some(Version::V2),
            id,
            method: method.into(),
            params,
        });
        (id, call)
    }

    async fn execute(&self, _id: RequestId, request: &Request) -> Result<Response> {
        self.send_request(request).await
    }
}

#[async_trait::async_trait]
impl BatchTransport for HttpTransport {}
