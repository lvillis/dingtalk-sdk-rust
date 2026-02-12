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
