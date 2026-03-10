use std::sync::Arc;

use reqx::{advanced::PermissiveRetryEligibility, prelude::Client as HttpClient};
use url::Url;

use crate::{
    api::{EnterpriseService, WebhookService},
    auth::AppCredentials,
    client::shared::{self, BuilderConfig, SharedClientState},
    error::{Error, Result},
};

/// Builder for async [`Client`].
#[derive(Debug, Clone, Default)]
pub struct ClientBuilder {
    config: BuilderConfig,
}

shared::impl_builder_methods!(ClientBuilder);

impl ClientBuilder {
    /// Builds an async [`Client`].
    pub fn build(self) -> Result<Client> {
        let base_urls = self.config.normalized_base_urls()?;
        let webhook_http = self.build_http_client(&base_urls.webhook)?;
        let enterprise_http = self.build_http_client(&base_urls.enterprise)?;

        Ok(Client {
            inner: Arc::new(Inner {
                webhook_http,
                enterprise_http,
                shared: SharedClientState::new(
                    base_urls,
                    self.config.cache_access_token,
                    self.config.token_refresh_margin,
                    self.config.body_snippet,
                ),
            }),
        })
    }

    fn build_http_client(&self, base_url: &Url) -> Result<HttpClient> {
        let mut builder = HttpClient::builder(base_url.as_str())
            .profile(self.config.profile)
            .client_name(self.config.client_name.clone())
            .connect_timeout(self.config.connect_timeout);

        if let Some(request_timeout) = self.config.request_timeout {
            builder = builder.request_timeout(request_timeout);
        }

        if let Some(total_timeout) = self.config.total_timeout {
            builder = builder.total_timeout(total_timeout);
        }

        if self.config.no_system_proxy {
            builder = builder.no_proxy(["*"]);
        }

        if let Some(retry_policy) = &self.config.retry_policy {
            builder = builder.retry_policy(retry_policy.clone());
        }

        if self.config.retry_non_idempotent {
            builder = builder.retry_eligibility(Arc::new(PermissiveRetryEligibility));
        }

        for (name, value) in &self.config.default_headers {
            builder = builder.try_default_header(name, value)?;
        }

        builder.build().map_err(Error::from)
    }
}

#[derive(Clone)]
/// Async DingTalk SDK client.
pub struct Client {
    inner: Arc<Inner>,
}

struct Inner {
    webhook_http: HttpClient,
    enterprise_http: HttpClient,
    shared: SharedClientState,
}

impl Client {
    /// Returns a new builder.
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Builds a client using defaults.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Creates a webhook robot service.
    #[must_use]
    pub fn webhook(&self, token: impl Into<String>, secret: Option<String>) -> WebhookService {
        WebhookService::new(self.clone(), token, secret)
    }

    /// Creates an enterprise robot service.
    #[must_use]
    pub fn enterprise(
        &self,
        appkey: impl Into<String>,
        appsecret: impl Into<String>,
        robot_code: impl Into<String>,
    ) -> EnterpriseService {
        EnterpriseService::new(self.clone(), appkey, appsecret, robot_code)
    }

    pub(crate) fn webhook_http(&self) -> &HttpClient {
        &self.inner.webhook_http
    }

    pub(crate) fn enterprise_http(&self) -> &HttpClient {
        &self.inner.enterprise_http
    }

    pub(crate) fn webhook_base_url(&self) -> &Url {
        self.inner.shared.webhook_base_url()
    }

    pub(crate) fn webhook_endpoint(&self, segments: &[&str]) -> Result<Url> {
        self.inner.shared.webhook_endpoint(segments)
    }

    pub(crate) fn enterprise_endpoint(&self, segments: &[&str]) -> Result<Url> {
        self.inner.shared.enterprise_endpoint(segments)
    }

    pub(crate) fn cached_access_token(&self, credentials: &AppCredentials) -> Option<String> {
        self.inner.shared.cached_access_token(credentials)
    }

    pub(crate) fn store_access_token(
        &self,
        credentials: &AppCredentials,
        token: String,
        expires_in_seconds: Option<i64>,
    ) {
        self.inner
            .shared
            .store_access_token(credentials, token, expires_in_seconds);
    }

    pub(crate) fn body_snippet(&self) -> crate::transport::BodySnippetConfig {
        self.inner.shared.body_snippet()
    }
}
