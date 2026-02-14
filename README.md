<div align=right>Table of Contents↗️</div>

<h1 align=center><code>dingtalk-sdk-rust</code></h1>

<p align=center>Dingtalk SDK for Rust.</p>

<div align=center>
  <a href="https://crates.io/crates/dingtalk-sdk">
    <img src="https://img.shields.io/crates/v/dingtalk-sdk.svg" alt="crates.io version">
  </a>
  <a href="https://crates.io/crates/dingtalk-sdk">
    <img src="https://img.shields.io/crates/dr/dingtalk-sdk?color=ba86eb" alt="crates.io downloads">
  </a>
  <a href="https://crates.io/crates/dingtalk-sdk">
    <img src="https://img.shields.io/github/repo-size/lvillis/dingtalk-sdk-rust?style=flat-square&color=328657" alt="crates.io version">
  </a>
  <a href="https://github.com/lvillis/dingtalk-sdk-rust/actions">
    <img src="https://github.com/lvillis/dingtalk-sdk-rust/actions/workflows/ci.yaml/badge.svg" alt="build status">
  </a>
  <a href="mailto:lvillis@outlook.com?subject=Thanks%20for%20dingtalk-sdk-rust!">
    <img src="https://img.shields.io/badge/Say%20Thanks-!-1EAEDB.svg" alt="say thanks">
  </a>
</div>

---


This project is a DingTalk API SDK written in Rust, with async/blocking clients, configurable transport, and typed robot APIs (Webhook + Enterprise).

## Features

- **Layered architecture**:
  - Public API: `Client` / `BlockingClient` + service exports
  - Service layer: `api/*` (Webhook + Enterprise domain methods)
  - Transport helpers: `transport/*` (token cache, webhook signature URL, standard API validation)
  - Types layer: `types/*` (public requests + internal payload models)
  - Auth primitives: `auth/*` (`AppCredentials` with redacted debug output)

- **Client architecture**:
  - Async: `ClientBuilder -> Client -> webhook()/enterprise()`
  - Blocking: `BlockingClientBuilder -> BlockingClient -> webhook()/enterprise()`
- **Builder best-practice options**:
  - base URL override (`webhook_base_url`, `enterprise_base_url`)
  - transport retry (`with_retry` / `retry_policy`, optional non-idempotent retry)
  - default headers (`default_header`)
  - enterprise access token cache (`cache_access_token`, `token_refresh_margin`)
- **Service types**:
  - Async: `WebhookService`, `EnterpriseService`
  - Blocking: `BlockingWebhookService`, `BlockingEnterpriseService`
- **Selectable TLS backend per mode** (choose one for each enabled mode):
  - Choosing a TLS feature automatically enables the corresponding runtime mode.
  - Async mode: `async-tls-rustls-ring` / `async-tls-rustls-aws-lc-rs` / `async-tls-native`
  - Blocking mode: `blocking-tls-rustls-ring` / `blocking-tls-rustls-aws-lc-rs` / `blocking-tls-native`
- **Fixed signing backend**: `hmac` + `sha2` (HMAC-SHA256 for webhook signature)
- **Compile-time feature guards** for invalid combinations.
- **Unified error model**: `Error`, `ErrorKind`, retry/retry-after helpers.
- **Safe URL construction**: normalized base URL + segment-based endpoint building.

## Supported Message Types

### Webhook Robot
- [x] Text Messages
- [x] Link Messages
- [x] Markdown Messages
- [x] ActionCard Messages (Single and Multi-Button)
- [x] FeedCard Messages

### Enterprise Robot
- [x] Group Messages
- [x] Private (OTO) Messages
- [x] Automatic message reply handling based on message context
- [x] Contacts (User/Department Get/List/Create/Update/Delete + lookups)
- [x] Approvals (Create/Get/List IDs/Terminate)

## Installation

Default (async + rustls-ring):

```toml
[dependencies]
dingtalk-sdk = "1"
```

Async only:

```toml
[dependencies]
dingtalk-sdk = { version = "1", default-features = false, features = ["async-tls-rustls-ring"] }
```

Blocking only:

```toml
[dependencies]
dingtalk-sdk = { version = "1", default-features = false, features = ["blocking-tls-rustls-ring"] }
```

Switch TLS backend:

```toml
[dependencies]
dingtalk-sdk = { version = "1", default-features = false, features = ["async-tls-rustls-aws-lc-rs"] }
```

## Run Examples

Async webhook:

```bash
export DINGTALK_WEBHOOK_TOKEN=your_token
export DINGTALK_WEBHOOK_SECRET=your_secret
cargo run --example async_webhook
```

Async enterprise contacts:

```bash
export DINGTALK_APP_KEY=your_appkey
export DINGTALK_APP_SECRET=your_appsecret
export DINGTALK_ROBOT_CODE=your_robot_code
export DINGTALK_USER_ID=your_userid
cargo run --example async_enterprise_contacts
```

Blocking examples:

```bash
cargo run --no-default-features --features "blocking-tls-rustls-ring" --example blocking_webhook
cargo run --no-default-features --features "blocking-tls-rustls-ring" --example blocking_enterprise_contacts
```

## Quick Examples

### Async Client + Webhook Service

```rust
use dingtalk_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;
    let robot = client.webhook("your_token", Some("your_secret".into()));

    let response = robot
        .send_text_message("Hello from Rust!", None, None, Some(false))
        .await?;
    println!("Sent successfully: {}", response);
    Ok(())
}
```

Custom endpoint and retry policy:

```rust
use std::time::Duration;
use dingtalk_sdk::{Client, RetryConfig};

let client = Client::builder()
    .webhook_base_url("https://oapi.dingtalk.com")?
    .enterprise_base_url("https://api.dingtalk.com")?
    .retry(RetryConfig::standard().max_retries(3))
    .retry_non_idempotent(false)
    .token_refresh_margin(Duration::from_secs(180))
    .build()?;
```

### Async Enterprise Service

```rust
use dingtalk_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;
    let robot = client.enterprise("appkey", "appsecret", "robot_code");

    let response = robot
        .send_group_message("open_conversation_id", "Greetings", "Hello group from Rust!")
        .await?;
    println!("Message sent: {}", response);

    // Contacts: get user detail
    let user = robot
        .contact_get_user(dingtalk_sdk::ContactGetUserRequest::new("manager123"))
        .await?;
    println!("User: {}", serde_json::to_string_pretty(&user)?);

    // Approvals: query instance detail
    let process = robot
        .approval_get_process_instance("PROC-INSTANCE-ID")
        .await?;
    println!("Process: {}", serde_json::to_string_pretty(&process)?);
    Ok(())
}
```

### Blocking Client + Webhook Service

```rust
use dingtalk_sdk::BlockingClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BlockingClient::builder().build()?;
    let robot = client.webhook("your_token", Some("your_secret".into()));
    let response = robot.send_text_message("Hello from blocking Rust!", None, None, Some(false))?;
    println!("Sent successfully: {}", response);
    Ok(())
}
```

## Tests

Default test suite:

```bash
cargo test
```

Run tests with blocking feature set:

```bash
cargo test --no-default-features --features "blocking-tls-rustls-ring"
```

## License

This project is licensed under the MIT License.
