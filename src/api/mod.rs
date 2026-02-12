#[cfg(feature = "async")]
mod async_enterprise;
#[cfg(feature = "async")]
mod async_webhook;
#[cfg(feature = "blocking")]
mod blocking_enterprise;
#[cfg(feature = "blocking")]
mod blocking_webhook;

#[cfg(feature = "async")]
/// Async enterprise service.
pub use async_enterprise::EnterpriseService;
#[cfg(feature = "async")]
/// Async webhook service.
pub use async_webhook::WebhookService;
#[cfg(feature = "blocking")]
/// Blocking enterprise service.
pub use blocking_enterprise::BlockingEnterpriseService;
#[cfg(feature = "blocking")]
/// Blocking webhook service.
pub use blocking_webhook::BlockingWebhookService;
