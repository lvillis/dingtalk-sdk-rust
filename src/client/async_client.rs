use std::{sync::Arc, time::Duration};

use reqx::{
    PermissiveRetryEligibility, RetryPolicy as ReqxRetryPolicy, prelude::Client as HttpClient,
};
use url::Url;

use crate::{
    api::{EnterpriseService, WebhookService},
    error::{Error, Result},
    retry::RetryConfig,
    transport::{BodySnippetConfig, DEFAULT_ENTERPRISE_BASE_URL, DEFAULT_WEBHOOK_BASE_URL},
    util::url::{endpoint_url, normalize_base_url},
};

const DEFAULT_CLIENT_NAME: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_TOKEN_REFRESH_MARGIN: Duration = Duration::from_secs(120);

/// Builder for async [`Client`].
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    client_name: String,
    request_timeout: Duration,
    total_timeout: Option<Duration>,
    connect_timeout: Duration,
    no_system_proxy: bool,
    webhook_base_url: Url,
    enterprise_base_url: Url,
    retry_config: Option<RetryConfig>,
    retry_non_idempotent: bool,
    default_headers: Vec<(String, String)>,
    cache_access_token: bool,
    token_refresh_margin: Duration,
    body_snippet: BodySnippetConfig,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            client_name: DEFAULT_CLIENT_NAME.to_owned(),
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            total_timeout: None,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            no_system_proxy: false,
            webhook_base_url: normalize_base_url(DEFAULT_WEBHOOK_BASE_URL)
                .expect("default webhook base url must be valid"),
            enterprise_base_url: normalize_base_url(DEFAULT_ENTERPRISE_BASE_URL)
                .expect("default enterprise base url must be valid"),
            retry_config: None,
            retry_non_idempotent: false,
            default_headers: Vec::new(),
            cache_access_token: true,
            token_refresh_margin: DEFAULT_TOKEN_REFRESH_MARGIN,
            body_snippet: BodySnippetConfig::default(),
        }
    }
}

impl ClientBuilder {
    /// Creates a builder with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets client name (user-agent like identifier).
    #[must_use]
    pub fn client_name(mut self, value: impl Into<String>) -> Self {
        self.client_name = value.into();
        self
    }

    /// Sets per-request timeout.
    #[must_use]
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// Sets total request deadline.
    #[must_use]
    pub fn total_timeout(mut self, value: Duration) -> Self {
        self.total_timeout = Some(value);
        self
    }

    /// Sets TCP connect timeout.
    #[must_use]
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// Enables or disables system proxy usage.
    #[must_use]
    pub fn no_system_proxy(mut self, enabled: bool) -> Self {
        self.no_system_proxy = enabled;
        self
    }

    /// Overrides webhook API base URL.
    pub fn webhook_base_url(mut self, value: impl Into<String>) -> Result<Self> {
        self.webhook_base_url = normalize_base_url(value.into())?;
        Ok(self)
    }

    /// Overrides enterprise API base URL.
    pub fn enterprise_base_url(mut self, value: impl Into<String>) -> Result<Self> {
        self.enterprise_base_url = normalize_base_url(value.into())?;
        Ok(self)
    }

    /// Sets retry configuration.
    #[must_use]
    pub fn retry(mut self, value: RetryConfig) -> Self {
        self.retry_config = Some(value);
        self
    }

    /// Convenience helper to configure standard retries.
    #[must_use]
    pub fn with_retry(mut self, max_retries: usize, base_backoff: Duration) -> Self {
        self.retry_config = Some(RetryConfig::new(max_retries, base_backoff));
        self
    }

    /// Enables retry for non-idempotent requests.
    #[must_use]
    pub fn retry_non_idempotent(mut self, enabled: bool) -> Self {
        self.retry_non_idempotent = enabled;
        self
    }

    /// Adds a default header to all requests.
    #[must_use]
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.push((name.into(), value.into()));
        self
    }

    /// Enables or disables enterprise access-token cache.
    #[must_use]
    pub fn cache_access_token(mut self, enabled: bool) -> Self {
        self.cache_access_token = enabled;
        self
    }

    /// Sets refresh margin for cached access-token expiration.
    #[must_use]
    pub fn token_refresh_margin(mut self, value: Duration) -> Self {
        self.token_refresh_margin = value;
        self
    }

    /// Configures body snippet capture for API errors.
    #[must_use]
    pub fn body_snippet(mut self, value: BodySnippetConfig) -> Self {
        self.body_snippet = value;
        self
    }

    /// Builds an async [`Client`].
    pub fn build(self) -> Result<Client> {
        let webhook_http = self.build_http_client(&self.webhook_base_url)?;
        let enterprise_http = self.build_http_client(&self.enterprise_base_url)?;

        Ok(Client {
            inner: Arc::new(Inner {
                webhook_http,
                enterprise_http,
                webhook_base_url: self.webhook_base_url,
                enterprise_base_url: self.enterprise_base_url,
                cache_access_token: self.cache_access_token,
                token_refresh_margin: self.token_refresh_margin,
                body_snippet: self.body_snippet,
            }),
        })
    }

    fn build_http_client(&self, base_url: &Url) -> Result<HttpClient> {
        let mut builder = HttpClient::builder(base_url.as_str())
            .client_name(self.client_name.clone())
            .request_timeout(self.request_timeout)
            .connect_timeout(self.connect_timeout);

        if let Some(total_timeout) = self.total_timeout {
            builder = builder.total_timeout(total_timeout);
        }

        if self.no_system_proxy {
            builder = builder.no_proxy(["*"]);
        }

        if let Some(retry_config) = self.retry_config {
            let retry_policy = ReqxRetryPolicy::standard()
                .max_attempts(retry_config.max_retries.saturating_add(1))
                .base_backoff(retry_config.base_backoff);
            builder = builder.retry_policy(retry_policy);
        }

        if self.retry_non_idempotent {
            builder = builder.retry_eligibility(Arc::new(PermissiveRetryEligibility));
        }

        for (name, value) in &self.default_headers {
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
    webhook_base_url: Url,
    enterprise_base_url: Url,
    cache_access_token: bool,
    token_refresh_margin: Duration,
    body_snippet: BodySnippetConfig,
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
        &self.inner.webhook_base_url
    }

    pub(crate) fn webhook_endpoint(&self, segments: &[&str]) -> Result<Url> {
        endpoint_url(&self.inner.webhook_base_url, segments)
    }

    pub(crate) fn enterprise_endpoint(&self, segments: &[&str]) -> Result<Url> {
        endpoint_url(&self.inner.enterprise_base_url, segments)
    }

    pub(crate) fn cache_access_token_enabled(&self) -> bool {
        self.inner.cache_access_token
    }

    pub(crate) fn token_refresh_margin(&self) -> Duration {
        self.inner.token_refresh_margin
    }

    pub(crate) fn body_snippet(&self) -> BodySnippetConfig {
        self.inner.body_snippet
    }
}
