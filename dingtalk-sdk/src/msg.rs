use serde::{Deserialize, Serialize};

/// Send Dingtalk or WeChatWork message
#[derive(Clone, Copy, Debug)]
pub enum DingTalkType {
    /// DingTalk
    DingTalk,
    /// WeChatWork
    WeChatWork,
}

/// Default DingTalkType is DingTalk
impl Default for DingTalkType {
    fn default() -> Self {
        DingTalkType::DingTalk
    }
}

/// DingTalk message type
/// * Text - text message
/// * Markdown - markdown message
/// * Link - link message
/// * ActionCard - action card message
/// * FeedCard - feed card message
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DingTalkMessageType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "markdown")]
    Markdown,
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "actionCard")]
    ActionCard,
    #[serde(rename = "feedCard")]
    FeedCard,
}

/// Default DingTalkMessageType is Text
impl Default for DingTalkMessageType {
    fn default() -> Self {
        DingTalkMessageType::Text
    }
}

/// DingTalk messge action card avatar
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum DingTalkMessageActionCardHideAvatar {
    #[serde(rename = "1")]
    Hide,
    #[serde(rename = "0")]
    Show,
}

// default value
impl Default for DingTalkMessageActionCardHideAvatar {
    fn default() -> Self {
        DingTalkMessageActionCardHideAvatar::Show
    }
}

/// DingTalk message action card orientation
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum DingTalkMessageActionCardBtnOrientation {
    #[serde(rename = "0")]
    Vertical,
    #[serde(rename = "1")]
    Landscape,
}

/// default value
impl Default for DingTalkMessageActionCardBtnOrientation {
    fn default() -> Self {
        DingTalkMessageActionCardBtnOrientation::Vertical
    }
}

/// DingTalk message action card btn
#[derive(Debug)]
pub struct DingTalkMessageActionCardBtn {
    pub title: String,
    pub action_url: String,
}

/// DingTalk message feed card link
#[derive(Debug)]
pub struct DingTalkMessageFeedCardLink {
    pub title: String,
    pub message_url: String,
    pub pic_url: String,
}

/// DingTalk message
#[derive(Debug, Default)]
pub struct DingTalkMessage {
    pub message_type: DingTalkMessageType,
    pub text_content: String,
    pub markdown_title: String,
    pub markdown_content: String,
    pub link_text: String,
    pub link_title: String,
    pub link_pic_url: String,
    pub link_message_url: String,
    pub action_card_title: String,
    pub action_card_text: String,
    pub action_card_hide_avatar: DingTalkMessageActionCardHideAvatar,
    pub action_card_btn_orientation: DingTalkMessageActionCardBtnOrientation,
    pub action_card_single_btn: Option<DingTalkMessageActionCardBtn>,
    pub action_card_btns: Vec<DingTalkMessageActionCardBtn>,
    pub feed_card_links: Vec<DingTalkMessageFeedCardLink>,
    pub at_all: bool,
    pub at_mobiles: Vec<String>,
}

///////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InnerTextMessageText {
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InnerTextMessage {
    pub msgtype: DingTalkMessageType,
    pub text: InnerTextMessageText,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerLinkMessageLink {
    pub title: String,
    pub text: String,
    pub pic_url: String,
    pub message_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InnerLinkMessage {
    pub msgtype: DingTalkMessageType,
    pub link: InnerLinkMessageLink,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerMarkdownMessageMarkdown {
    pub title: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InnerMarkdownMessage {
    pub msgtype: DingTalkMessageType,
    pub markdown: InnerMarkdownMessageMarkdown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerActionCardMessageActionCard {
    pub title: String,
    pub text: String,
    pub hide_avatar: DingTalkMessageActionCardHideAvatar,
    pub btn_orientation: DingTalkMessageActionCardBtnOrientation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InnerActionCardMessageBtn {
    pub title: String,
    #[serde(rename = "actionURL")]
    pub action_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerActionCardMessage {
    pub msgtype: DingTalkMessageType,
    pub action_card: InnerActionCardMessageActionCard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InnerFeedCardMessageFeedCardLink {
    pub title: String,
    #[serde(rename = "messageURL")]
    pub message_url: String,
    #[serde(rename = "picURL")]
    pub pic_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InnerFeedCardMessageFeedCard {
    pub links: Vec<InnerFeedCardMessageFeedCardLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerFeedCardMessage {
    pub msgtype: DingTalkMessageType,
    pub feed_card: InnerFeedCardMessageFeedCard,
}
