#![cfg(feature = "_blocking")]

use std::time::Duration;

use dingtalk_sdk::{BlockingClient, ContactListSubDepartmentsRequest, ErrorKind};

#[test]
fn blocking_client_builder_and_services_smoke_test() {
    let client = BlockingClient::builder()
        .client_name("dingtalk-sdk-tests/blocking")
        .request_timeout(Duration::from_secs(3))
        .connect_timeout(Duration::from_secs(2))
        .total_timeout(Duration::from_secs(5))
        .no_system_proxy(true)
        .default_header("x-sdk-test", "blocking")
        .cache_access_token(false)
        .token_refresh_margin(Duration::from_secs(30))
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
fn blocking_client_builder_rejects_query_in_base_url() {
    let error = BlockingClient::builder()
        .enterprise_base_url("https://api.dingtalk.com?debug=true")
        .expect_err("base url with query must be rejected");

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
