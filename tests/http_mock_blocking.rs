#![cfg(feature = "_blocking")]

use dingtalk_sdk::{BlockingClient, ContactGetUserRequest, ErrorKind};
use httpmock::prelude::*;

#[test]
fn blocking_contact_get_user_returns_typed_payload() {
    let server = MockServer::start();

    let get_token = server.mock(|when, then| {
        when.method(GET)
            .path("/gettoken")
            .query_param("appkey", "app-key")
            .query_param("appsecret", "app-secret");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"errcode":0,"errmsg":"ok","access_token":"token-123","expires_in":7200}"#);
    });

    let get_user = server.mock(|when, then| {
        when.method(POST)
            .path("/topapi/v2/user/get")
            .query_param("access_token", "token-123")
            .body_includes("\"userid\":\"manager-1\"");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{
                "errcode":0,
                "errmsg":"ok",
                "result":{"userid":"manager-1","name":"Alice","unionid":"union-1"},
                "request_id":"req-1"
            }"#,
            );
    });

    let client = BlockingClient::builder()
        .webhook_base_url(server.base_url())
        .expect("mock webhook url should be valid")
        .enterprise_base_url(server.base_url())
        .expect("mock enterprise url should be valid")
        .build()
        .expect("client should build");
    let enterprise = client.enterprise("app-key", "app-secret", "robot-code");

    let user = enterprise
        .contact_get_user(ContactGetUserRequest::new("manager-1"))
        .expect("request should succeed");

    assert_eq!(user.userid.as_deref(), Some("manager-1"));
    assert_eq!(user.name.as_deref(), Some("Alice"));
    assert_eq!(user.unionid.as_deref(), Some("union-1"));

    get_token.assert();
    get_user.assert();
}

#[test]
fn blocking_webhook_error_keeps_snippet_out_of_display() {
    let server = MockServer::start();

    let send = server.mock(|when, then| {
        when.method(POST)
            .path("/robot/send")
            .query_param("access_token", "token-123")
            .body_includes("\"msgtype\":\"text\"");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"errcode":310000,"errmsg":"invalid","access_token":"sensitive-token"}"#);
    });

    let client = BlockingClient::builder()
        .webhook_base_url(server.base_url())
        .expect("mock webhook url should be valid")
        .enterprise_base_url(server.base_url())
        .expect("mock enterprise url should be valid")
        .build()
        .expect("client should build");
    let webhook = client.webhook("token-123", None);

    let error = webhook
        .send_text_message("hello", None, None, Some(false))
        .expect_err("request should fail");

    assert_eq!(error.kind(), ErrorKind::Api);
    assert!(error.to_string().contains("invalid"));
    assert!(!error.to_string().contains("body_snippet"));
    assert!(error.body_snippet().is_some());

    send.assert();
}
