use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use url::Url;

use crate::{
    error::{Error, Result},
    types::internal::StandardApiResponse,
    util::{
        redact::{redact_text, truncate_snippet},
        url::endpoint_url,
    },
};

pub(crate) const DEFAULT_WEBHOOK_BASE_URL: &str = "https://oapi.dingtalk.com";
pub(crate) const DEFAULT_ENTERPRISE_BASE_URL: &str = "https://api.dingtalk.com";
pub(crate) const DEFAULT_MSG_KEY: &str = "sampleMarkdown";
const DEFAULT_ACCESS_TOKEN_TTL: Duration = Duration::from_secs(7_200);
const MIN_ACCESS_TOKEN_TTL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy)]
/// Configuration for attaching a redacted body snippet to API errors.
pub struct BodySnippetConfig {
    /// Whether body snippet capture is enabled.
    pub enabled: bool,
    /// Maximum number of bytes to keep.
    pub max_bytes: usize,
}

impl Default for BodySnippetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_bytes: 4096,
        }
    }
}

pub(crate) fn api_error(
    code: i64,
    message: impl Into<String>,
    request_id: Option<String>,
    body_snippet: Option<String>,
) -> Error {
    Error::Api {
        code,
        message: message.into(),
        request_id,
        body_snippet,
    }
}

pub(crate) fn build_webhook_url(base_url: &Url, token: &str, secret: Option<&str>) -> Result<Url> {
    let mut url = endpoint_url(base_url, &["robot", "send"])?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("access_token", token);

        if let Some(secret) = secret {
            let timestamp = crate::signature::current_timestamp_millis()?;
            let sign = crate::signature::create_signature(&timestamp, secret)?;
            query.append_pair("timestamp", &timestamp);
            query.append_pair("sign", &sign);
        }
    }
    Ok(url)
}

#[derive(Debug, Clone)]
pub(crate) struct AccessTokenCache {
    inner: Arc<RwLock<Option<CachedAccessToken>>>,
    refresh_margin: Duration,
}

#[derive(Debug, Clone)]
struct CachedAccessToken {
    token: String,
    expires_at: Instant,
}

impl AccessTokenCache {
    #[must_use]
    pub(crate) fn new(refresh_margin: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
            refresh_margin,
        }
    }

    pub(crate) fn get(&self) -> Option<String> {
        let now = Instant::now();
        let guard = self.inner.read().ok()?;
        let cached = guard.as_ref()?;
        let refresh_at = now.checked_add(self.refresh_margin)?;
        if refresh_at < cached.expires_at {
            Some(cached.token.clone())
        } else {
            None
        }
    }

    pub(crate) fn store(&self, token: String, expires_in_seconds: Option<i64>) {
        let ttl = normalize_token_ttl(expires_in_seconds);
        let expires_at = Instant::now().checked_add(ttl).unwrap_or_else(Instant::now);

        if let Ok(mut guard) = self.inner.write() {
            *guard = Some(CachedAccessToken { token, expires_at });
        }
    }
}

fn normalize_token_ttl(expires_in_seconds: Option<i64>) -> Duration {
    match expires_in_seconds {
        Some(value) if value > 0 => Duration::from_secs(value as u64).max(MIN_ACCESS_TOKEN_TTL),
        _ => DEFAULT_ACCESS_TOKEN_TTL,
    }
}

pub(crate) fn validate_standard_api_response(
    body: &str,
    body_snippet: BodySnippetConfig,
) -> Result<()> {
    if let Ok(response) = serde_json::from_str::<StandardApiResponse>(body)
        && let Some(errcode) = response.errcode
        && errcode != 0
    {
        let message = response
            .errmsg
            .unwrap_or_else(|| "unknown dingtalk api error".to_string());
        let snippet = body_snippet_for_error(body, body_snippet);
        return Err(api_error(errcode, message, None, snippet));
    }
    Ok(())
}

pub(crate) fn body_snippet_for_error(
    body: &str,
    body_snippet: BodySnippetConfig,
) -> Option<String> {
    if !body_snippet.enabled {
        return None;
    }

    let snippet = truncate_snippet(body, body_snippet.max_bytes);
    Some(redact_text(&snippet))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::url::normalize_base_url;

    #[test]
    fn build_webhook_url_without_secret_contains_token_only() {
        let base_url = normalize_base_url(DEFAULT_WEBHOOK_BASE_URL).expect("base");
        let url = build_webhook_url(&base_url, "token-123", None).expect("url");
        assert_eq!(
            url.as_str(),
            "https://oapi.dingtalk.com/robot/send?access_token=token-123"
        );
    }

    #[test]
    fn api_error_response_is_detected() {
        let body = r#"{"errcode":310000,"errmsg":"invalid"}"#;
        let error = validate_standard_api_response(body, BodySnippetConfig::default())
            .expect_err("should fail");
        match error {
            Error::Api {
                code,
                message,
                body_snippet,
                ..
            } => {
                assert_eq!(code, 310000);
                assert_eq!(message, "invalid");
                assert_eq!(
                    body_snippet.as_deref(),
                    Some("{\"errcode\":310000,\"errmsg\":\"invalid\"}")
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn api_success_response_passes() {
        let body = r#"{"errcode":0,"errmsg":"ok"}"#;
        validate_standard_api_response(body, BodySnippetConfig::default()).expect("ok");
    }

    #[test]
    fn api_error_response_can_disable_body_snippet() {
        let body = r#"{"errcode":310000,"errmsg":"invalid"}"#;
        let config = BodySnippetConfig {
            enabled: false,
            max_bytes: 64,
        };
        let error = validate_standard_api_response(body, config).expect_err("should fail");
        match error {
            Error::Api {
                code,
                message,
                body_snippet,
                ..
            } => {
                assert_eq!(code, 310000);
                assert_eq!(message, "invalid");
                assert_eq!(body_snippet, None);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn access_token_cache_honors_refresh_margin() {
        let cache = AccessTokenCache::new(Duration::from_secs(60));
        cache.store("token".to_string(), Some(1));
        assert!(cache.get().is_none());

        let cache = AccessTokenCache::new(Duration::from_secs(0));
        cache.store("token".to_string(), Some(60));
        assert_eq!(cache.get().as_deref(), Some("token"));
    }
}
