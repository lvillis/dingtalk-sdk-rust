use dingtalk_sdk::{
    ActionCardButton, AppCredentials, ApprovalTerminateProcessInstanceRequest, Error, ErrorKind,
    FeedCardLink,
};

#[test]
fn app_credentials_debug_redacts_secret() {
    let credentials = AppCredentials::new("app-key", "super-secret");
    let debug_output = format!("{credentials:?}");

    assert!(debug_output.contains("app-key"));
    assert!(debug_output.contains("<redacted>"));
    assert!(!debug_output.contains("super-secret"));
}

#[test]
fn webhook_public_types_use_expected_serde_renames() {
    let button = ActionCardButton::new("Open", "https://example.com/open");
    let button_value = serde_json::to_value(button).expect("button should serialize");
    assert_eq!(
        button_value
            .get("actionURL")
            .and_then(serde_json::Value::as_str),
        Some("https://example.com/open")
    );

    let link = FeedCardLink::new(
        "Docs",
        "https://example.com/docs",
        "https://example.com/pic.png",
    );
    let link_value = serde_json::to_value(link).expect("link should serialize");
    assert_eq!(
        link_value
            .get("messageURL")
            .and_then(serde_json::Value::as_str),
        Some("https://example.com/docs")
    );
    assert_eq!(
        link_value.get("picURL").and_then(serde_json::Value::as_str),
        Some("https://example.com/pic.png")
    );
}

#[test]
fn api_error_helpers_report_kind_retryability_and_request_id() {
    let retryable_error = Error::Api {
        code: 130101,
        message: "transient error".to_string(),
        request_id: Some("req-1".to_string()),
        body_snippet: None,
    };

    assert_eq!(retryable_error.kind(), ErrorKind::Api);
    assert!(retryable_error.is_retryable());
    assert_eq!(retryable_error.request_id(), Some("req-1"));

    let non_retryable_error = Error::Api {
        code: 400001,
        message: "bad request".to_string(),
        request_id: None,
        body_snippet: None,
    };
    assert!(!non_retryable_error.is_retryable());
}

#[test]
fn terminate_request_builder_sets_optional_fields() {
    let request = ApprovalTerminateProcessInstanceRequest::new("PROC-1", "user-1")
        .is_system(true)
        .remark("cancelled by integration test");
    let value = serde_json::to_value(request).expect("request should serialize");

    assert_eq!(
        value
            .get("process_instance_id")
            .and_then(serde_json::Value::as_str),
        Some("PROC-1")
    );
    assert_eq!(
        value
            .get("operating_userid")
            .and_then(serde_json::Value::as_str),
        Some("user-1")
    );
    assert_eq!(
        value.get("is_system").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        value.get("remark").and_then(serde_json::Value::as_str),
        Some("cancelled by integration test")
    );
}
