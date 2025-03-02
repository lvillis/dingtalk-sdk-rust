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


This project is a DingTalk API SDK written in Rust, providing convenient tools for sending messages via DingTalk robots (Webhook and Enterprise Bots). It leverages asynchronous programming using Tokio, offering straightforward and efficient interaction with DingTalk services.

## Features

- **Async Support**: Built using Tokio for efficient asynchronous communication.
- **Webhook Robot**: Supports various message types such as Text, Link, Markdown, ActionCard, and FeedCard.
- **Enterprise Robot**: Send messages to groups and individuals using DingTalk Enterprise API.
- **Automatic Signature**: Automatically generates signatures for secured Webhook interactions.
- **Comprehensive Error Handling**: Robust error reporting with clear error types and messages.

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

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
dingtalk-sdk = "1"
```

## Quick Example

### Webhook Robot

```rust
use dingtalk_sdk::DingTalkRobot;

#[tokio::main]
async fn main() {
    let robot = DingTalkRobot::new("your_token".into(), Some("your_secret".into()));

    match robot.send_text_message("Hello from Rust!", None, None, Some(false)).await {
        Ok(response) => println!("Sent successfully: {}", response),
        Err(e) => eprintln!("Error sending message: {}", e),
    }
}
```

### Enterprise Robot

```rust
use dingtalk_sdk::EnterpriseDingTalkRobot;

#[tokio::main]
async fn main() {
    let robot = EnterpriseDingTalkRobot::new("appkey".into(), "appsecret".into(), "robot_code".into());

    match robot.send_group_message("open_conversation_id", "Greetings", "Hello group from Rust!").await {
        Ok(response) => println!("Message sent: {}", response),
        Err(e) => eprintln!("Error sending message: {}", e),
    }
}
```

## License

This project is licensed under the MIT License.