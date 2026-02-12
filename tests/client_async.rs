#![cfg(feature = "_async")]

use std::time::Duration;

use dingtalk_sdk::{
    ApprovalTerminateProcessInstanceRequest, BodySnippetConfig, Client, ContactGetUserRequest,
    ErrorKind,
};

#[test]
fn async_client_builder_and_services_smoke_test() {
    let client = Client::builder()
        .client_name("dingtalk-sdk-tests/async")
        .request_timeout(Duration::from_secs(3))
        .connect_timeout(Duration::from_secs(2))
        .total_timeout(Duration::from_secs(5))
        .no_system_proxy(true)
        .default_header("x-sdk-test", "async")
        .cache_access_token(false)
        .token_refresh_margin(Duration::from_secs(30))
        .body_snippet(BodySnippetConfig {
            enabled: false,
            max_bytes: 128,
        })
        .webhook_base_url("https://oapi.dingtalk.com")
        .expect("webhook base url should be accepted")
        .enterprise_base_url("https://api.dingtalk.com")
        .expect("enterprise base url should be accepted")
        .build()
        .expect("client should build");

    let _webhook = client.webhook("token", None);
    let _enterprise = client.enterprise("appkey", "appsecret", "robot-code");
}

#[test]
fn async_client_builder_rejects_query_in_base_url() {
    let error = Client::builder()
        .webhook_base_url("https://oapi.dingtalk.com?debug=true")
        .expect_err("base url with query must be rejected");

    assert_eq!(error.kind(), ErrorKind::InvalidConfig);
}

#[test]
fn async_enterprise_request_builders_serialize_expected_fields() {
    let request = ContactGetUserRequest::new("manager-1").language("zh_CN");
    let request_value = serde_json::to_value(request).expect("request should serialize");
    assert_eq!(
        request_value
            .get("userid")
            .and_then(serde_json::Value::as_str),
        Some("manager-1")
    );
    assert_eq!(
        request_value
            .get("language")
            .and_then(serde_json::Value::as_str),
        Some("zh_CN")
    );

    let terminate = ApprovalTerminateProcessInstanceRequest::new("PROC-1", "manager-1")
        .is_system(true)
        .remark("cancel from async test");
    let terminate_value = serde_json::to_value(terminate).expect("request should serialize");
    assert_eq!(
        terminate_value
            .get("is_system")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        terminate_value
            .get("remark")
            .and_then(serde_json::Value::as_str),
        Some("cancel from async test")
    );
}
