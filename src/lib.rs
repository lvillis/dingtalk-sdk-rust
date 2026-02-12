#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs, rustdoc::broken_intra_doc_links)]
//! DingTalk API SDK for Rust.
//!
//! This crate provides:
//! - async client (`Client`) and services (`WebhookService`, `EnterpriseService`)
//! - optional blocking client (`BlockingClient`) and services
//! - typed request models for contacts and approvals
//! - unified error model and retry integration via `reqx`
//!
//! # Quick Start (Async)
//!
//! ```no_run
//! use dingtalk_sdk::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::builder().build()?;
//!     let webhook = client.webhook("your_token", Some("your_secret".into()));
//!     let _ = webhook
//!         .send_text_message("Hello from rustdoc", None, None, Some(false))
//!         .await?;
//!     Ok(())
//! }
//! ```
//!
//! # Feature Overview
//!
//! At least one runtime mode is required:
//! - `async`
//! - `blocking`
//!
//! Each enabled runtime mode requires exactly one TLS backend:
//! - async: `async-tls-rustls-ring` / `async-tls-rustls-aws-lc-rs` / `async-tls-native`
//! - blocking: `blocking-tls-rustls-ring` / `blocking-tls-rustls-aws-lc-rs` / `blocking-tls-native`

// Require at least one client mode.
#[cfg(not(any(feature = "async", feature = "blocking")))]
compile_error!("Enable at least one client mode: `async` or `blocking`.");

// Async mode requires exactly one async TLS backend.
#[cfg(all(
    feature = "async",
    not(any(
        feature = "async-tls-rustls-ring",
        feature = "async-tls-rustls-aws-lc-rs",
        feature = "async-tls-native"
    ))
))]
compile_error!(
    "When `async` is enabled, enable one async TLS backend: \
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
    not(feature = "async"),
    any(
        feature = "async-tls-rustls-ring",
        feature = "async-tls-rustls-aws-lc-rs",
        feature = "async-tls-native"
    )
))]
compile_error!("Async TLS features require enabling `async`.");

// Blocking mode requires exactly one blocking TLS backend.
#[cfg(all(
    feature = "blocking",
    not(any(
        feature = "blocking-tls-rustls-ring",
        feature = "blocking-tls-rustls-aws-lc-rs",
        feature = "blocking-tls-native"
    ))
))]
compile_error!(
    "When `blocking` is enabled, enable one blocking TLS backend: \
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
    not(feature = "blocking"),
    any(
        feature = "blocking-tls-rustls-ring",
        feature = "blocking-tls-rustls-aws-lc-rs",
        feature = "blocking-tls-native"
    )
))]
compile_error!("Blocking TLS features require enabling `blocking`.");

mod api;
mod auth;
mod client;
mod error;
mod signature;
mod transport;
mod types;
mod util;

#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
pub use api::{BlockingEnterpriseService, BlockingWebhookService};
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub use api::{EnterpriseService, WebhookService};
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub use client::async_client::{Client, ClientBuilder};
#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
pub use client::blocking_client::{BlockingClient, BlockingClientBuilder};

#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
/// Blocking runtime service aliases.
pub mod blocking {
    pub use crate::{
        BlockingEnterpriseService as EnterpriseService, BlockingWebhookService as WebhookService,
    };
}

/// Application credentials used by enterprise APIs.
pub use auth::AppCredentials;
/// SDK error type and helpers.
pub use error::{Error, ErrorKind, Result};
/// Re-export of retry policy type from `reqx`.
pub use reqx::RetryPolicy;
/// Controls whether and how response snippets are retained in errors.
pub use transport::BodySnippetConfig;
/// Public webhook and enterprise request/response helper types.
pub use types::{
    ActionCardButton, ApprovalCreateProcessInstanceRequest, ApprovalFormComponentValue,
    ApprovalListProcessInstanceIdsRequest, ApprovalListProcessInstanceIdsResult,
    ApprovalTerminateProcessInstanceRequest, ContactCreateDepartmentRequest,
    ContactCreateUserRequest, ContactDeleteDepartmentRequest, ContactDeleteUserRequest,
    ContactGetDepartmentRequest, ContactGetUserByMobileRequest, ContactGetUserByUnionIdRequest,
    ContactGetUserRequest, ContactListSubDepartmentIdsRequest, ContactListSubDepartmentsRequest,
    ContactListUsersRequest, ContactUpdateDepartmentRequest, ContactUpdateUserRequest,
    FeedCardLink,
};
