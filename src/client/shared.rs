use std::time::Duration;

use reqx::{advanced::ClientProfile, prelude::RetryPolicy as ReqxRetryPolicy};
use url::Url;

use crate::{
    auth::AppCredentials,
    error::Result,
    transport::{
        AccessTokenCache, BodySnippetConfig, DEFAULT_ENTERPRISE_BASE_URL, DEFAULT_WEBHOOK_BASE_URL,
    },
    util::url::{endpoint_url, normalize_base_url},
};

pub(crate) const DEFAULT_CLIENT_NAME: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
pub(crate) const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub(crate) const DEFAULT_TOKEN_REFRESH_MARGIN: Duration = Duration::from_secs(120);
pub(crate) const MIN_TIMEOUT: Duration = Duration::from_millis(1);

#[derive(Debug, Clone)]
pub(crate) struct BuilderConfig {
    pub(crate) client_name: String,
    pub(crate) profile: ClientProfile,
    pub(crate) request_timeout: Option<Duration>,
    pub(crate) total_timeout: Option<Duration>,
    pub(crate) connect_timeout: Duration,
    pub(crate) no_system_proxy: bool,
    pub(crate) webhook_base_url: String,
    pub(crate) enterprise_base_url: String,
    pub(crate) retry_policy: Option<ReqxRetryPolicy>,
    pub(crate) retry_non_idempotent: bool,
    pub(crate) default_headers: Vec<(String, String)>,
    pub(crate) cache_access_token: bool,
    pub(crate) token_refresh_margin: Duration,
    pub(crate) body_snippet: BodySnippetConfig,
}

impl Default for BuilderConfig {
    fn default() -> Self {
        Self {
            client_name: DEFAULT_CLIENT_NAME.to_owned(),
            profile: ClientProfile::StandardSdk,
            request_timeout: None,
            total_timeout: None,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            no_system_proxy: false,
            webhook_base_url: DEFAULT_WEBHOOK_BASE_URL.to_owned(),
            enterprise_base_url: DEFAULT_ENTERPRISE_BASE_URL.to_owned(),
            retry_policy: None,
            retry_non_idempotent: false,
            default_headers: Vec::new(),
            cache_access_token: true,
            token_refresh_margin: DEFAULT_TOKEN_REFRESH_MARGIN,
            body_snippet: BodySnippetConfig::default(),
        }
    }
}

impl BuilderConfig {
    pub(crate) fn apply_profile(&mut self, value: ClientProfile) {
        self.profile = value;
        self.request_timeout = None;
        self.total_timeout = None;
        self.retry_policy = None;
    }

    pub(crate) fn normalized_base_urls(&self) -> Result<BaseUrls> {
        Ok(BaseUrls {
            webhook: normalize_base_url(self.webhook_base_url.clone())?,
            enterprise: normalize_base_url(self.enterprise_base_url.clone())?,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BaseUrls {
    pub(crate) webhook: Url,
    pub(crate) enterprise: Url,
}

#[derive(Debug, Clone)]
pub(crate) struct SharedClientState {
    webhook_base_url: Url,
    enterprise_base_url: Url,
    access_token_cache: Option<AccessTokenCache>,
    token_refresh_margin: Duration,
    body_snippet: BodySnippetConfig,
}

impl SharedClientState {
    pub(crate) fn new(
        base_urls: BaseUrls,
        cache_access_token: bool,
        token_refresh_margin: Duration,
        body_snippet: BodySnippetConfig,
    ) -> Self {
        Self {
            webhook_base_url: base_urls.webhook,
            enterprise_base_url: base_urls.enterprise,
            access_token_cache: cache_access_token.then(AccessTokenCache::new),
            token_refresh_margin,
            body_snippet,
        }
    }

    pub(crate) fn webhook_base_url(&self) -> &Url {
        &self.webhook_base_url
    }

    pub(crate) fn webhook_endpoint(&self, segments: &[&str]) -> Result<Url> {
        endpoint_url(&self.webhook_base_url, segments)
    }

    pub(crate) fn enterprise_endpoint(&self, segments: &[&str]) -> Result<Url> {
        endpoint_url(&self.enterprise_base_url, segments)
    }

    pub(crate) fn cached_access_token(&self, credentials: &AppCredentials) -> Option<String> {
        self.access_token_cache
            .as_ref()
            .and_then(|cache| cache.get(credentials, self.token_refresh_margin))
    }

    pub(crate) fn store_access_token(
        &self,
        credentials: &AppCredentials,
        token: String,
        expires_in_seconds: Option<i64>,
    ) {
        if let Some(cache) = &self.access_token_cache {
            cache.store(credentials.clone(), token, expires_in_seconds);
        }
    }

    pub(crate) fn body_snippet(&self) -> BodySnippetConfig {
        self.body_snippet
    }
}

macro_rules! impl_builder_methods {
    ($builder:ident) => {
        impl $builder {
            /// Creates a builder with defaults.
            #[must_use]
            pub fn new() -> Self {
                Self::default()
            }

            /// Applies a reqx transport profile and clears profile-controlled overrides.
            #[must_use]
            pub fn profile(mut self, value: reqx::advanced::ClientProfile) -> Self {
                self.config.apply_profile(value);
                self
            }

            /// Sets client name (user-agent like identifier).
            #[must_use]
            pub fn client_name(mut self, value: impl Into<String>) -> Self {
                self.config.client_name = value.into();
                self
            }

            /// Sets per-request timeout.
            #[must_use]
            pub fn request_timeout(mut self, value: std::time::Duration) -> Self {
                self.config.request_timeout = Some(value.max(crate::client::shared::MIN_TIMEOUT));
                self
            }

            /// Sets total request deadline.
            #[must_use]
            pub fn total_timeout(mut self, value: std::time::Duration) -> Self {
                self.config.total_timeout = Some(value.max(crate::client::shared::MIN_TIMEOUT));
                self
            }

            /// Sets TCP connect timeout.
            #[must_use]
            pub fn connect_timeout(mut self, value: std::time::Duration) -> Self {
                self.config.connect_timeout = value.max(crate::client::shared::MIN_TIMEOUT);
                self
            }

            /// Enables or disables system proxy usage.
            #[must_use]
            pub fn no_system_proxy(mut self, enabled: bool) -> Self {
                self.config.no_system_proxy = enabled;
                self
            }

            /// Overrides webhook API base URL.
            #[must_use]
            pub fn webhook_base_url(mut self, value: impl Into<String>) -> Self {
                self.config.webhook_base_url = value.into();
                self
            }

            /// Overrides enterprise API base URL.
            #[must_use]
            pub fn enterprise_base_url(mut self, value: impl Into<String>) -> Self {
                self.config.enterprise_base_url = value.into();
                self
            }

            /// Overrides the reqx retry policy selected by the current profile.
            #[must_use]
            pub fn retry_policy(mut self, value: reqx::prelude::RetryPolicy) -> Self {
                self.config.retry_policy = Some(value);
                self
            }

            /// Enables retry for non-idempotent requests.
            #[must_use]
            pub fn retry_non_idempotent(mut self, enabled: bool) -> Self {
                self.config.retry_non_idempotent = enabled;
                self
            }

            /// Adds a default header to all requests.
            #[must_use]
            pub fn default_header(
                mut self,
                name: impl Into<String>,
                value: impl Into<String>,
            ) -> Self {
                self.config
                    .default_headers
                    .push((name.into(), value.into()));
                self
            }

            /// Enables or disables enterprise access-token cache.
            #[must_use]
            pub fn cache_access_token(mut self, enabled: bool) -> Self {
                self.config.cache_access_token = enabled;
                self
            }

            /// Sets refresh margin for cached access-token expiration.
            #[must_use]
            pub fn token_refresh_margin(mut self, value: std::time::Duration) -> Self {
                self.config.token_refresh_margin = value;
                self
            }

            /// Configures body snippet capture for API errors.
            #[must_use]
            pub fn body_snippet(mut self, value: crate::transport::BodySnippetConfig) -> Self {
                self.config.body_snippet = value;
                self
            }
        }
    };
}

pub(crate) use impl_builder_methods;
