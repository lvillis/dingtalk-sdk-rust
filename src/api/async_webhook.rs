use crate::{
    client::async_client::Client,
    error::Result,
    transport::{build_webhook_url, validate_standard_api_response},
    types::{
        ActionCardButton, FeedCardLink,
        internal::{
            ActionCardContent, FeedCardContent, LinkContent, MarkdownContent, TextContent,
            WebhookMessage, build_at,
        },
    },
};

/// Async webhook robot service.
#[derive(Clone)]
pub struct WebhookService {
    client: Client,
    token: String,
    secret: Option<String>,
}

impl WebhookService {
    pub(crate) fn new(client: Client, token: impl Into<String>, secret: Option<String>) -> Self {
        Self {
            client,
            token: token.into(),
            secret,
        }
    }

    async fn send_message(&self, message: &WebhookMessage) -> Result<String> {
        let url = build_webhook_url(
            self.client.webhook_base_url(),
            &self.token,
            self.secret.as_deref(),
        )?;
        let response = self
            .client
            .webhook_http()
            .post(url.as_str())
            .json(message)?
            .send()
            .await?;
        let body = response.text_lossy();
        validate_standard_api_response(&body)?;
        Ok(body)
    }

    /// Sends a text webhook message.
    pub async fn send_text_message(
        &self,
        content: &str,
        at_mobiles: Option<Vec<String>>,
        at_user_ids: Option<Vec<String>>,
        is_at_all: Option<bool>,
    ) -> Result<String> {
        let message = WebhookMessage::Text {
            text: TextContent {
                content: content.to_string(),
            },
            at: build_at(at_mobiles, at_user_ids, is_at_all),
        };
        self.send_message(&message).await
    }

    /// Sends a link webhook message.
    pub async fn send_link_message(
        &self,
        title: &str,
        text: &str,
        message_url: &str,
        pic_url: Option<&str>,
    ) -> Result<String> {
        let message = WebhookMessage::Link {
            link: LinkContent {
                title: title.to_string(),
                text: text.to_string(),
                message_url: message_url.to_string(),
                pic_url: pic_url.map(ToOwned::to_owned),
            },
            at: None,
        };
        self.send_message(&message).await
    }

    /// Sends a markdown webhook message.
    pub async fn send_markdown_message(
        &self,
        title: &str,
        text: &str,
        at_mobiles: Option<Vec<String>>,
        at_user_ids: Option<Vec<String>>,
        is_at_all: Option<bool>,
    ) -> Result<String> {
        let message = WebhookMessage::Markdown {
            markdown: MarkdownContent {
                title: title.to_string(),
                text: text.to_string(),
            },
            at: build_at(at_mobiles, at_user_ids, is_at_all),
        };
        self.send_message(&message).await
    }

    /// Sends a single-button action-card webhook message.
    pub async fn send_action_card_message_single(
        &self,
        title: &str,
        text: &str,
        single_title: &str,
        single_url: &str,
        btn_orientation: Option<&str>,
    ) -> Result<String> {
        let message = WebhookMessage::ActionCard {
            action_card: ActionCardContent {
                title: title.to_string(),
                text: text.to_string(),
                btn_orientation: btn_orientation.map(ToOwned::to_owned),
                single_title: Some(single_title.to_string()),
                single_url: Some(single_url.to_string()),
                btns: None,
            },
        };
        self.send_message(&message).await
    }

    /// Sends a multi-button action-card webhook message.
    pub async fn send_action_card_message_multi(
        &self,
        title: &str,
        text: &str,
        btns: Vec<ActionCardButton>,
        btn_orientation: Option<&str>,
    ) -> Result<String> {
        let message = WebhookMessage::ActionCard {
            action_card: ActionCardContent {
                title: title.to_string(),
                text: text.to_string(),
                btn_orientation: btn_orientation.map(ToOwned::to_owned),
                single_title: None,
                single_url: None,
                btns: Some(btns),
            },
        };
        self.send_message(&message).await
    }

    /// Sends a feed-card webhook message.
    pub async fn send_feed_card_message(&self, links: Vec<FeedCardLink>) -> Result<String> {
        let message = WebhookMessage::FeedCard {
            feed_card: FeedCardContent { links },
        };
        self.send_message(&message).await
    }
}
