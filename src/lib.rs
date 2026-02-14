#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs, rustdoc::broken_intra_doc_links)]
//! DingTalk API SDK for Rust.
//!
//! This crate provides:
//! - async client (`Client`) and services (`WebhookService`, `EnterpriseService`)
//! - optional blocking client (`BlockingClient`) and services
//! - typed request/response models for contacts and approvals
//! - unified error model and retry integration via `reqx`
//!
//! # Quick Start (Async)
//!
//! ```no_run
//! #[cfg(feature = "_async")]
//! use dingtalk_sdk::Client;
//!
//! #[cfg(feature = "_async")]
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::builder().build()?;
//!     let webhook = client.webhook("your_token", Some("your_secret".into()));
//!     let _ = webhook
//!         .send_text_message("Hello from rustdoc", None, None, Some(false))
//!         .await?;
//!     Ok(())
//! }
//!
//! #[cfg(not(feature = "_async"))]
//! fn main() {}
//! ```
//!
//! # Feature Overview
//!
//! At least one runtime mode is required:
//! - `_async` (internal)
//! - `_blocking` (internal)
//!
//! Each enabled runtime mode requires exactly one TLS backend:
//! - async: `async-tls-rustls-ring` / `async-tls-rustls-aws-lc-rs` / `async-tls-native`
//! - blocking: `blocking-tls-rustls-ring` / `blocking-tls-rustls-aws-lc-rs` / `blocking-tls-native`
//!
//! In normal usage, select TLS features directly. They automatically enable the
//! matching runtime mode.

// Require at least one client mode.
#[cfg(not(any(feature = "_async", feature = "_blocking")))]
compile_error!(
    "Enable at least one transport feature: \
     `async-tls-rustls-ring`, `async-tls-rustls-aws-lc-rs`, `async-tls-native`, \
     `blocking-tls-rustls-ring`, `blocking-tls-rustls-aws-lc-rs`, or `blocking-tls-native`."
);

// Async mode requires exactly one async TLS backend.
#[cfg(all(
    feature = "_async",
    not(any(
        feature = "async-tls-rustls-ring",
        feature = "async-tls-rustls-aws-lc-rs",
        feature = "async-tls-native"
    ))
))]
compile_error!(
    "When `_async` is enabled, enable one async TLS backend: \
     `async-tls-rustls-ring`, `async-tls-rustls-aws-lc-rs`, or `async-tls-native`."
);
#[cfg(all(
    feature = "async-tls-rustls-ring",
    feature = "async-tls-rustls-aws-lc-rs"
))]
compile_error!("`async-tls-rustls-ring` and `async-tls-rustls-aws-lc-rs` are mutually exclusive.");
#[cfg(all(feature = "async-tls-rustls-ring", feature = "async-tls-native"))]
compile_error!("`async-tls-rustls-ring` and `async-tls-native` are mutually exclusive.");
#[cfg(all(feature = "async-tls-rustls-aws-lc-rs", feature = "async-tls-native"))]
compile_error!("`async-tls-rustls-aws-lc-rs` and `async-tls-native` are mutually exclusive.");
#[cfg(all(
    not(feature = "_async"),
    any(
        feature = "async-tls-rustls-ring",
        feature = "async-tls-rustls-aws-lc-rs",
        feature = "async-tls-native"
    )
))]
compile_error!("Async TLS features require enabling `_async`.");

// Blocking mode requires exactly one blocking TLS backend.
#[cfg(all(
    feature = "_blocking",
    not(any(
        feature = "blocking-tls-rustls-ring",
        feature = "blocking-tls-rustls-aws-lc-rs",
        feature = "blocking-tls-native"
    ))
))]
compile_error!(
    "When `_blocking` is enabled, enable one blocking TLS backend: \
     `blocking-tls-rustls-ring`, `blocking-tls-rustls-aws-lc-rs`, or `blocking-tls-native`."
);
#[cfg(all(
    feature = "blocking-tls-rustls-ring",
    feature = "blocking-tls-rustls-aws-lc-rs"
))]
compile_error!(
    "`blocking-tls-rustls-ring` and `blocking-tls-rustls-aws-lc-rs` are mutually exclusive."
);
#[cfg(all(feature = "blocking-tls-rustls-ring", feature = "blocking-tls-native"))]
compile_error!("`blocking-tls-rustls-ring` and `blocking-tls-native` are mutually exclusive.");
#[cfg(all(
    feature = "blocking-tls-rustls-aws-lc-rs",
    feature = "blocking-tls-native"
))]
compile_error!("`blocking-tls-rustls-aws-lc-rs` and `blocking-tls-native` are mutually exclusive.");
#[cfg(all(
    not(feature = "_blocking"),
    any(
        feature = "blocking-tls-rustls-ring",
        feature = "blocking-tls-rustls-aws-lc-rs",
        feature = "blocking-tls-native"
    )
))]
compile_error!("Blocking TLS features require enabling `_blocking`.");

mod api;
mod auth;
mod client;
mod error;
mod retry;
mod signature;
mod transport;
mod types;
mod util;

#[cfg(feature = "_blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "_blocking")))]
pub use api::{BlockingEnterpriseService, BlockingWebhookService};
#[cfg(feature = "_async")]
#[cfg_attr(docsrs, doc(cfg(feature = "_async")))]
pub use api::{EnterpriseService, WebhookService};
#[cfg(feature = "_async")]
#[cfg_attr(docsrs, doc(cfg(feature = "_async")))]
pub use client::async_client::{Client, ClientBuilder};
#[cfg(feature = "_blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "_blocking")))]
pub use client::blocking_client::{BlockingClient, BlockingClientBuilder};

#[cfg(feature = "_blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "_blocking")))]
/// Blocking runtime service aliases.
pub mod blocking {
    pub use crate::{
        BlockingEnterpriseService as EnterpriseService, BlockingWebhookService as WebhookService,
    };
}

/// Application credentials used by enterprise APIs.
pub use auth::AppCredentials;
/// SDK error type and helpers.
pub use error::{Error, ErrorKind, HttpError, Result, TransportError};
/// SDK retry policy configuration.
pub use retry::RetryConfig;
/// Controls whether and how response snippets are retained in errors.
pub use transport::BodySnippetConfig;
/// Public webhook and enterprise request/response helper types.
pub use types::{
    ActionCardButton, ApprovalCreateProcessInstanceRequest, ApprovalFormComponentValue,
    ApprovalListProcessInstanceIdsRequest, ApprovalListProcessInstanceIdsResult,
    ApprovalProcessInstance, ApprovalTerminateProcessInstanceRequest,
    ContactCreateDepartmentRequest, ContactCreateDepartmentResult, ContactCreateUserRequest,
    ContactCreateUserResult, ContactDeleteDepartmentRequest, ContactDeleteUserRequest,
    ContactDepartment, ContactGetDepartmentRequest, ContactGetUserByMobileRequest,
    ContactGetUserByUnionIdRequest, ContactGetUserRequest, ContactListSubDepartmentIdsRequest,
    ContactListSubDepartmentIdsResult, ContactListSubDepartmentsRequest,
    ContactListSubDepartmentsResult, ContactListUsersRequest, ContactListUsersResult,
    ContactUpdateDepartmentRequest, ContactUpdateUserRequest, ContactUser, FeedCardLink,
};
