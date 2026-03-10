use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant, SystemTime},
};

use serde::de::DeserializeOwned;
use url::Url;

use crate::{
    auth::AppCredentials,
    error::{Error, HttpError, Result, TransportError},
    types::{
        enterprise::ApprovalProcessInstance,
        internal::{
            ApprovalCreateProcessInstanceResponse, ApprovalGetProcessInstanceResponse,
            GetTokenResponse, StandardApiResponse, TopApiResultResponse, TopApiSimpleResponse,
        },
    },
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

pub(crate) fn response_api_error(
    code: i64,
    message: impl Into<String>,
    request_id: Option<String>,
    body: &str,
    body_snippet: BodySnippetConfig,
) -> Error {
    api_error(
        code,
        message,
        request_id,
        body_snippet_for_error(body, body_snippet),
    )
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
    inner: Arc<RwLock<HashMap<AppCredentials, CachedAccessToken>>>,
}

#[derive(Debug, Clone)]
struct CachedAccessToken {
    token: String,
    expires_at: Instant,
}

impl AccessTokenCache {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub(crate) fn get(
        &self,
        credentials: &AppCredentials,
        refresh_margin: Duration,
    ) -> Option<String> {
        let now = Instant::now();
        let guard = self.inner.read().ok()?;
        let cached = guard.get(credentials)?;
        let refresh_at = now.checked_add(refresh_margin)?;
        if refresh_at < cached.expires_at {
            Some(cached.token.clone())
        } else {
            None
        }
    }

    pub(crate) fn store(
        &self,
        credentials: AppCredentials,
        token: String,
        expires_in_seconds: Option<i64>,
    ) {
        let ttl = normalize_token_ttl(expires_in_seconds);
        let expires_at = Instant::now().checked_add(ttl).unwrap_or_else(Instant::now);

        if let Ok(mut guard) = self.inner.write() {
            guard.insert(credentials, CachedAccessToken { token, expires_at });
        }
    }
}

fn normalize_token_ttl(expires_in_seconds: Option<i64>) -> Duration {
    match expires_in_seconds {
        Some(value) if value > 0 => Duration::from_secs(value as u64).max(MIN_ACCESS_TOKEN_TTL),
        _ => DEFAULT_ACCESS_TOKEN_TTL,
    }
}

#[derive(Debug)]
pub(crate) struct AccessTokenPayload {
    pub(crate) token: String,
    pub(crate) expires_in: Option<i64>,
}

struct SuccessfulResponseBody {
    body: String,
    header_request_id: Option<String>,
}

struct DecodedResponse<T> {
    value: T,
    body: String,
    header_request_id: Option<String>,
}

#[cfg(test)]
pub(crate) fn validate_standard_api_response(
    body: &str,
    body_snippet: BodySnippetConfig,
) -> Result<()> {
    validate_standard_api_response_with_request_id(body, None, body_snippet)
}

pub(crate) fn parse_standard_api_text_response(
    response: reqx::Response,
    body_snippet: BodySnippetConfig,
) -> Result<String> {
    let response = successful_response_body(response, body_snippet)?;
    validate_standard_api_response_with_request_id(
        &response.body,
        response.header_request_id,
        body_snippet,
    )?;
    Ok(response.body)
}

pub(crate) fn parse_get_token_response(
    response: reqx::Response,
    body_snippet: BodySnippetConfig,
) -> Result<AccessTokenPayload> {
    let DecodedResponse {
        value,
        body,
        header_request_id,
    } = decode_json_response::<GetTokenResponse>(response, body_snippet)?;
    let GetTokenResponse {
        errcode,
        errmsg,
        access_token,
        expires_in,
        request_id,
    } = value;
    let request_id = request_id.or(header_request_id);

    if errcode != 0 {
        return Err(response_api_error(
            errcode,
            errmsg,
            request_id,
            &body,
            body_snippet,
        ));
    }

    let token = access_token.ok_or_else(|| {
        response_api_error(
            -1,
            "No access token returned",
            request_id.clone(),
            &body,
            body_snippet,
        )
    })?;

    Ok(AccessTokenPayload { token, expires_in })
}

pub(crate) fn parse_topapi_result_response<T>(
    response: reqx::Response,
    body_snippet: BodySnippetConfig,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let DecodedResponse {
        value,
        body,
        header_request_id,
    } = decode_json_response::<TopApiResultResponse<T>>(response, body_snippet)?;
    let TopApiResultResponse {
        errcode,
        errmsg,
        result,
        request_id,
    } = value;
    let request_id = request_id.or(header_request_id);

    if errcode != 0 {
        return Err(response_api_error(
            errcode,
            errmsg,
            request_id,
            &body,
            body_snippet,
        ));
    }

    result.ok_or_else(|| {
        response_api_error(
            -1,
            "Missing result field in topapi response",
            request_id,
            &body,
            body_snippet,
        )
    })
}

pub(crate) fn parse_topapi_unit_response(
    response: reqx::Response,
    body_snippet: BodySnippetConfig,
) -> Result<()> {
    let DecodedResponse {
        value,
        body,
        header_request_id,
    } = decode_json_response::<TopApiSimpleResponse>(response, body_snippet)?;
    let TopApiSimpleResponse {
        errcode,
        errmsg,
        request_id,
    } = value;
    let request_id = request_id.or(header_request_id);

    if errcode != 0 {
        return Err(response_api_error(
            errcode,
            errmsg,
            request_id,
            &body,
            body_snippet,
        ));
    }

    Ok(())
}

pub(crate) fn parse_approval_create_response(
    response: reqx::Response,
    body_snippet: BodySnippetConfig,
) -> Result<String> {
    let DecodedResponse {
        value,
        body,
        header_request_id,
    } = decode_json_response::<ApprovalCreateProcessInstanceResponse>(response, body_snippet)?;
    let ApprovalCreateProcessInstanceResponse {
        errcode,
        errmsg,
        process_instance_id,
        request_id,
    } = value;
    let request_id = request_id.or(header_request_id);

    if errcode != 0 {
        return Err(response_api_error(
            errcode,
            errmsg,
            request_id,
            &body,
            body_snippet,
        ));
    }

    process_instance_id.ok_or_else(|| {
        response_api_error(
            -1,
            "Missing process_instance_id in response",
            request_id,
            &body,
            body_snippet,
        )
    })
}

pub(crate) fn parse_approval_get_response(
    response: reqx::Response,
    body_snippet: BodySnippetConfig,
) -> Result<ApprovalProcessInstance> {
    let DecodedResponse {
        value,
        body,
        header_request_id,
    } = decode_json_response::<ApprovalGetProcessInstanceResponse>(response, body_snippet)?;
    let ApprovalGetProcessInstanceResponse {
        errcode,
        errmsg,
        process_instance,
        request_id,
    } = value;
    let request_id = request_id.or(header_request_id);

    if errcode != 0 {
        return Err(response_api_error(
            errcode,
            errmsg,
            request_id,
            &body,
            body_snippet,
        ));
    }

    process_instance.ok_or_else(|| {
        response_api_error(
            -1,
            "Missing process_instance field in response",
            request_id,
            &body,
            body_snippet,
        )
    })
}

fn validate_standard_api_response_with_request_id(
    body: &str,
    header_request_id: Option<String>,
    body_snippet: BodySnippetConfig,
) -> Result<()> {
    if let Some(response) = parse_standard_api_response_body(body)
        && let Some(errcode) = response.errcode
        && errcode != 0
    {
        let message = response
            .errmsg
            .unwrap_or_else(|| "unknown dingtalk api error".to_string());
        let request_id = response.request_id.or(header_request_id);
        let snippet = body_snippet_for_error(body, body_snippet);
        return Err(api_error(errcode, message, request_id, snippet));
    }
    Ok(())
}

fn successful_response_body(
    response: reqx::Response,
    body_snippet: BodySnippetConfig,
) -> Result<SuccessfulResponseBody> {
    let status = response.status().as_u16();
    let header_request_id = response_request_id(&response);
    let retry_after = response_retry_after(&response);
    let body = response.text_lossy();

    if !(200..=299).contains(&status) {
        return Err(http_error_from_response(
            status,
            &body,
            header_request_id,
            retry_after,
            body_snippet,
        ));
    }

    Ok(SuccessfulResponseBody {
        body,
        header_request_id,
    })
}

fn decode_json_response<T>(
    response: reqx::Response,
    body_snippet: BodySnippetConfig,
) -> Result<DecodedResponse<T>>
where
    T: DeserializeOwned,
{
    let SuccessfulResponseBody {
        body,
        header_request_id,
    } = successful_response_body(response, body_snippet)?;
    let value = serde_json::from_str(&body)?;

    Ok(DecodedResponse {
        value,
        body,
        header_request_id,
    })
}

fn parse_standard_api_response_body(body: &str) -> Option<StandardApiResponse> {
    serde_json::from_str(body).ok()
}

fn http_error_from_response(
    status: u16,
    body: &str,
    header_request_id: Option<String>,
    retry_after: Option<Duration>,
    body_snippet: BodySnippetConfig,
) -> Error {
    let parsed = parse_standard_api_response_body(body);
    let message = parsed.as_ref().and_then(|response| response.errmsg.clone());
    let request_id = parsed
        .and_then(|response| response.request_id)
        .or(header_request_id);
    let snippet = body_snippet_for_error(body, body_snippet);
    let error = HttpError {
        status,
        message: message.clone(),
        request_id: request_id.clone(),
        body_snippet: snippet.clone(),
    };

    match status {
        401 | 403 => Error::Auth(error),
        404 => Error::NotFound(error),
        409 | 412 => Error::Conflict(error),
        429 => Error::RateLimited { retry_after, error },
        _ => Error::Transport(Box::new(TransportError {
            status: Some(status),
            message,
            request_id,
            body_snippet: snippet,
            retry_after,
            retryable: matches!(status, 429 | 500..=599),
            code: "http_status",
            method: None,
            uri: None,
            timeout_phase: None,
            transport_kind: None,
        })),
    }
}

fn response_request_id(response: &reqx::Response) -> Option<String> {
    response
        .headers()
        .get("x-request-id")
        .or_else(|| response.headers().get("x-acs-request-id"))
        .or_else(|| response.headers().get("x-amz-request-id"))
        .or_else(|| response.headers().get("x-amz-id-2"))
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn response_retry_after(response: &reqx::Response) -> Option<Duration> {
    let header = response.headers().get("retry-after")?;
    let value = header.to_str().ok()?;
    parse_retry_after(value, SystemTime::now())
}

fn parse_retry_after(value: &str, now: SystemTime) -> Option<Duration> {
    if let Ok(seconds) = value.trim().parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    let when = httpdate::parse_http_date(value).ok()?;
    when.duration_since(now).ok()
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
    fn access_token_cache_honors_refresh_margin_per_credentials() {
        let cache = AccessTokenCache::new();
        let credentials = AppCredentials::new("app-key", "app-secret");
        cache.store(credentials.clone(), "token".to_string(), Some(1));
        assert!(cache.get(&credentials, Duration::from_secs(60)).is_none());

        cache.store(credentials.clone(), "token".to_string(), Some(60));
        assert_eq!(
            cache.get(&credentials, Duration::from_secs(0)).as_deref(),
            Some("token")
        );
    }
}
