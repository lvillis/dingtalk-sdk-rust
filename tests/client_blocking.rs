#![cfg(feature = "_blocking")]

use std::time::Duration;

use dingtalk_sdk::{
    BlockingClient, BodySnippetConfig, ClientProfile, ContactListSubDepartmentsRequest, ErrorKind,
    RetryPolicy,
};

#[test]
fn blocking_client_builder_and_services_smoke_test() {
    let client = BlockingClient::builder()
        .profile(ClientProfile::LowLatency)
        .client_name("dingtalk-sdk-tests/blocking")
        .request_timeout(Duration::from_secs(3))
        .connect_timeout(Duration::from_secs(2))
        .total_timeout(Duration::from_secs(5))
        .no_system_proxy(true)
        .retry_policy(
            RetryPolicy::standard()
                .max_attempts(4)
                .base_backoff(Duration::from_millis(100)),
        )
        .default_header("x-sdk-test", "blocking")
        .cache_access_token(false)
        .token_refresh_margin(Duration::from_secs(30))
        .body_snippet(BodySnippetConfig {
            enabled: false,
            max_bytes: 128,
        })
        .webhook_base_url("https://oapi.dingtalk.com")
        .enterprise_base_url("https://api.dingtalk.com")
        .build()
        .expect("client should build");

    let _webhook = client.webhook("token", None);
    let _enterprise = client.enterprise("appkey", "appsecret", "robot-code");
}

#[test]
fn blocking_client_builder_rejects_query_in_base_url() {
    let result = BlockingClient::builder()
        .enterprise_base_url("https://api.dingtalk.com?debug=true")
        .build();

    assert!(result.is_err(), "base url with query must be rejected");
    let error = result.err().expect("error should be present");

    assert_eq!(error.kind(), ErrorKind::InvalidConfig);
}

#[test]
fn blocking_enterprise_request_builder_serializes_expected_fields() {
    let request = ContactListSubDepartmentsRequest::new(11).language("zh_CN");
    let value = serde_json::to_value(request).expect("request should serialize");

    assert_eq!(
        value.get("dept_id").and_then(serde_json::Value::as_i64),
        Some(11)
    );
    assert_eq!(
        value.get("language").and_then(serde_json::Value::as_str),
        Some("zh_CN")
    );
}
