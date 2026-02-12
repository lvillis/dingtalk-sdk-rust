#[cfg(feature = "_async")]
mod async_enterprise;
#[cfg(feature = "_async")]
mod async_webhook;
#[cfg(feature = "_blocking")]
mod blocking_enterprise;
#[cfg(feature = "_blocking")]
mod blocking_webhook;

#[cfg(feature = "_async")]
/// Async enterprise service.
pub use async_enterprise::EnterpriseService;
#[cfg(feature = "_async")]
/// Async webhook service.
pub use async_webhook::WebhookService;
#[cfg(feature = "_blocking")]
/// Blocking enterprise service.
pub use blocking_enterprise::BlockingEnterpriseService;
#[cfg(feature = "_blocking")]
/// Blocking webhook service.
pub use blocking_webhook::BlockingWebhookService;

pub(crate) fn is_private_conversation(data: &serde_json::Value) -> bool {
    data.get("conversationType")
        .is_some_and(|value| value.as_str() == Some("1") || value.as_i64() == Some(1))
}

#[cfg(test)]
mod tests {
    use super::is_private_conversation;

    #[test]
    fn private_conversation_type_as_string() {
        let payload = serde_json::json!({ "conversationType": "1" });
        assert!(is_private_conversation(&payload));
    }

    #[test]
    fn private_conversation_type_as_number() {
        let payload = serde_json::json!({ "conversationType": 1 });
        assert!(is_private_conversation(&payload));
    }

    #[test]
    fn non_private_conversation_type_is_false() {
        let payload = serde_json::json!({ "conversationType": "2" });
        assert!(!is_private_conversation(&payload));
    }
}
