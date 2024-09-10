use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use std::{
    env, fs,
    io::{Error, ErrorKind},
    path::PathBuf,
    time::SystemTime,
};
use base64::Engine;

mod msg;
use msg::*;

pub use msg::{
    DingTalkMessage, DingTalkMessageActionCardBtn, DingTalkMessageActionCardBtnOrientation,
    DingTalkMessageActionCardHideAvatar, DingTalkMessageFeedCardLink, DingTalkMessageType,
    DingTalkType,
};

pub type XResult<T> = Result<T, Box<dyn std::error::Error>>;

const CONTENT_TYPE: &str = "Content-Type";
const APPLICATION_JSON_UTF8: &str = "application/json; charset=utf-8";

const DEFAULT_DINGTALK_ROBOT_URL: &str = "https://oapi.dingtalk.com/robot/send";
const DEFAULT_WECHAT_WORK_ROBOT_URL: &str = "https://qyapi.weixin.qq.com/cgi-bin/webhook/send";

/// `DingTalk` is a simple SDK for DingTalk webhook robot
///
/// Document https://ding-doc.dingtalk.com/doc#/serverapi2/qf2nxq
///
/// Sample code:
/// ```ignore
/// let dt = DingTalk::new("<token>", "");
/// dt.send_text("Hello world!")?;
/// ```
///
/// At all sample:
/// ```ignore
/// dt.send_message(&DingTalkMessage::new_text("Hello World!").at_all())?;
/// ```
#[derive(Default)]
pub struct DingTalk {
    pub dingtalk_type: DingTalkType,
    pub default_webhook_url: String,
    pub access_token: String,
    pub sec_token: String,
    pub direct_url: String,
}

impl DingTalkMessage {
    /// New text DingTalk message
    pub fn new_text(text_content: &str) -> Self {
        Self::new(DingTalkMessageType::Text).text(text_content)
    }

    /// New markdown DingTalk message
    pub fn new_markdown(markdown_title: &str, markdown_content: &str) -> Self {
        Self::new(DingTalkMessageType::Markdown).markdown(markdown_title, markdown_content)
    }

    /// New link DingTalk message
    pub fn new_link(
        link_title: &str,
        link_text: &str,
        link_pic_url: &str,
        link_message_url: &str,
    ) -> Self {
        Self::new(DingTalkMessageType::Link).link(
            link_title,
            link_text,
            link_pic_url,
            link_message_url,
        )
    }

    /// New action card DingTalk message
    pub fn new_action_card(title: &str, text: &str) -> Self {
        let mut s = Self::new(DingTalkMessageType::ActionCard);
        s.action_card_title = title.into();
        s.action_card_text = text.into();
        s
    }

    /// New feed card DingTalk message
    pub fn new_feed_card() -> Self {
        Self::new(DingTalkMessageType::FeedCard)
    }

    /// New DingTalk message
    pub fn new(message_type: DingTalkMessageType) -> Self {
        DingTalkMessage {
            message_type,
            ..Default::default()
        }
    }

    /// Set text
    pub fn text(mut self, text_content: &str) -> Self {
        self.text_content = text_content.into();
        self
    }

    /// Set markdown
    pub fn markdown(mut self, markdown_title: &str, markdown_content: &str) -> Self {
        self.markdown_title = markdown_title.into();
        self.markdown_content = markdown_content.into();
        self
    }

    /// Set link
    pub fn link(
        mut self,
        link_title: &str,
        link_text: &str,
        link_pic_url: &str,
        link_message_url: &str,
    ) -> Self {
        self.link_title = link_title.into();
        self.link_text = link_text.into();
        self.link_pic_url = link_pic_url.into();
        self.link_message_url = link_message_url.into();
        self
    }

    /// Set action card show avator(default show)
    pub fn action_card_show_avatar(mut self) -> Self {
        self.action_card_hide_avatar = DingTalkMessageActionCardHideAvatar::Show;
        self
    }

    /// Set action card hide avator
    pub fn action_card_hide_avatar(mut self) -> Self {
        self.action_card_hide_avatar = DingTalkMessageActionCardHideAvatar::Hide;
        self
    }

    /// Set action card btn vertical(default vertical)
    pub fn action_card_btn_vertical(mut self) -> Self {
        self.action_card_btn_orientation = DingTalkMessageActionCardBtnOrientation::Vertical;
        self
    }

    /// Set action card btn landscape
    pub fn action_card_btn_landscape(mut self) -> Self {
        self.action_card_btn_orientation = DingTalkMessageActionCardBtnOrientation::Landscape;
        self
    }

    /// Set action card single btn
    pub fn set_action_card_signle_btn(mut self, btn: DingTalkMessageActionCardBtn) -> Self {
        self.action_card_single_btn = Some(btn);
        self
    }

    /// Add action card btn
    pub fn add_action_card_btn(mut self, btn: DingTalkMessageActionCardBtn) -> Self {
        self.action_card_btns.push(btn);
        self
    }

    /// Add feed card link
    pub fn add_feed_card_link(mut self, link: DingTalkMessageFeedCardLink) -> Self {
        self.feed_card_links.push(link);
        self
    }

    /// Add feed card link detail
    pub fn add_feed_card_link_detail(self, title: &str, message_url: &str, pic_url: &str) -> Self {
        self.add_feed_card_link(DingTalkMessageFeedCardLink {
            title: title.into(),
            message_url: message_url.into(),
            pic_url: pic_url.into(),
        })
    }

    /// At all
    pub fn at_all(mut self) -> Self {
        self.at_all = true;
        self
    }

    /// At mobiles
    pub fn at_mobiles(mut self, mobiles: &[String]) -> Self {
        for m in mobiles {
            self.at_mobiles.push(m.clone());
        }
        self
    }
}

impl DingTalk {
    /// Create `DingTalk` from token:
    /// wechatwork:access_token
    /// dingtalk:access_token?sec_token
    pub fn from_token(token: &str) -> XResult<Self> {
        if token.starts_with("dingtalk:") {
            let token_and_or_sec = &token["dingtalk:".len()..];
            let mut token_and_or_sec_vec = token_and_or_sec.split('?');
            let access_token = match token_and_or_sec_vec.next() {
                Some(t) => t,
                None => token_and_or_sec,
            };
            let sec_token = match token_and_or_sec_vec.next() {
                Some(t) => t,
                None => "",
            };
            Ok(Self::new(access_token, sec_token))
        } else if token.starts_with("wechatwork:") {
            Ok(Self::new_wechat(&token["wechatwork:".len()..]))
        } else if token.starts_with("wecom:") {
            Ok(Self::new_wechat(&token["wecom:".len()..]))
        } else {
            Err(Box::new(Error::new(
                ErrorKind::Other,
                format!("Tokne format erorr: {}", token),
            )))
        }
    }

    /// Create `DingTalk` from file
    ///
    /// Format see `DingTalk::from_json(json: &str)`
    pub fn from_file(f: &str) -> XResult<Self> {
        let f_path_buf = if f.starts_with("~/") {
            let home = PathBuf::from(env::var("HOME")?);
            home.join(f.chars().skip(2).collect::<String>())
        } else {
            PathBuf::from(f)
        };
        let f_content = fs::read_to_string(f_path_buf)?;
        Self::from_json(&f_content)
    }

    /// Create `DingTalk` from JSON string
    ///
    /// Format:
    /// ```json
    /// {
    ///     "default_webhook_url": "", // option
    ///     "access_token": "<access token>",
    ///     "sec_token": "<sec token>" // option
    /// }
    /// ```
    pub fn from_json(json: &str) -> XResult<Self> {
        let json_value: Value = serde_json::from_str(json)?;
        if !json_value.is_object() {
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                format!("JSON format erorr: {}", json),
            )));
        }
        let type_str = json_value["type"]
            .as_str()
            .unwrap_or_default()
            .to_lowercase();
        let dingtalk_type = match type_str.as_str() {
            "wechat" | "wechatwork" | "wecom" => DingTalkType::WeChatWork,
            _ => DingTalkType::DingTalk,
        };

        let default_webhook_url = json_value["default_webhook_url"]
            .as_str()
            .unwrap_or_else(|| match dingtalk_type {
                DingTalkType::DingTalk => DEFAULT_DINGTALK_ROBOT_URL,
                DingTalkType::WeChatWork => DEFAULT_WECHAT_WORK_ROBOT_URL,
            })
            .to_owned();
        let access_token = json_value["access_token"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        let sec_token = json_value["sec_token"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        let direct_url = json_value["direct_url"]
            .as_str()
            .unwrap_or_default()
            .to_owned();

        Ok(DingTalk {
            dingtalk_type,
            default_webhook_url,
            access_token,
            sec_token,
            direct_url,
        })
    }

    /// Create `DingTalk` from url, for outgoing robot
    pub fn from_url(direct_url: &str) -> Self {
        DingTalk {
            direct_url: direct_url.into(),
            ..Default::default()
        }
    }

    /// Create `DingTalk`
    /// `access_token` is access token, `sec_token` can be empty `""`
    pub fn new(access_token: &str, sec_token: &str) -> Self {
        DingTalk {
            default_webhook_url: DEFAULT_DINGTALK_ROBOT_URL.into(),
            access_token: access_token.into(),
            sec_token: sec_token.into(),
            ..Default::default()
        }
    }

    /// Create `DingTalk` for WeChat Work
    pub fn new_wechat(key: &str) -> Self {
        DingTalk {
            default_webhook_url: DEFAULT_WECHAT_WORK_ROBOT_URL.into(),
            dingtalk_type: DingTalkType::WeChatWork,
            access_token: key.into(),
            ..Default::default()
        }
    }

    /// Set default webhook url
    pub fn set_default_webhook_url(&mut self, default_webhook_url: &str) {
        self.default_webhook_url = default_webhook_url.into();
    }

    /// Send DingTalk message
    ///
    /// 1. Create DingTalk JSON message
    /// 2. POST JSON message to DingTalk server
    pub async fn send_message(&self, dingtalk_message: DingTalkMessage) -> XResult<()> {
        let mut message_json = match dingtalk_message.message_type {
            DingTalkMessageType::Text => serde_json::to_value(InnerTextMessage {
                msgtype: DingTalkMessageType::Text,
                text: InnerTextMessageText {
                    content: dingtalk_message.text_content,
                },
            }),
            DingTalkMessageType::Link => serde_json::to_value(InnerLinkMessage {
                msgtype: DingTalkMessageType::Link,
                link: InnerLinkMessageLink {
                    title: dingtalk_message.link_title,
                    text: dingtalk_message.link_text,
                    pic_url: dingtalk_message.link_pic_url,
                    message_url: dingtalk_message.link_message_url,
                },
            }),
            DingTalkMessageType::Markdown => serde_json::to_value(InnerMarkdownMessage {
                msgtype: DingTalkMessageType::Markdown,
                markdown: InnerMarkdownMessageMarkdown {
                    title: dingtalk_message.markdown_title,
                    text: dingtalk_message.markdown_content,
                },
            }),
            DingTalkMessageType::ActionCard => serde_json::to_value(InnerActionCardMessage {
                msgtype: DingTalkMessageType::ActionCard,
                action_card: InnerActionCardMessageActionCard {
                    title: dingtalk_message.action_card_title,
                    text: dingtalk_message.action_card_text,
                    hide_avatar: dingtalk_message.action_card_hide_avatar,
                    btn_orientation: dingtalk_message.action_card_btn_orientation,
                },
            }),
            DingTalkMessageType::FeedCard => serde_json::to_value(InnerFeedCardMessage {
                msgtype: DingTalkMessageType::FeedCard,
                feed_card: InnerFeedCardMessageFeedCard {
                    links: {
                        let mut links: Vec<InnerFeedCardMessageFeedCardLink> = vec![];
                        for feed_card_link in &dingtalk_message.feed_card_links {
                            links.push(InnerFeedCardMessageFeedCardLink {
                                title: feed_card_link.title.clone(),
                                message_url: feed_card_link.message_url.clone(),
                                pic_url: feed_card_link.pic_url.clone(),
                            });
                        }
                        links
                    },
                },
            }),
        }?;
        if DingTalkMessageType::ActionCard == dingtalk_message.message_type {
            if dingtalk_message.action_card_single_btn.is_some() {
                if let Some(single_btn) = dingtalk_message.action_card_single_btn.as_ref() {
                    message_json["actionCard"]["singleTitle"] = single_btn.title.as_str().into();
                    message_json["actionCard"]["singleURL"] = single_btn.action_url.as_str().into();
                };
            } else {
                let mut btns: Vec<InnerActionCardMessageBtn> = vec![];
                for action_card_btn in &dingtalk_message.action_card_btns {
                    btns.push(InnerActionCardMessageBtn {
                        title: action_card_btn.title.clone(),
                        action_url: action_card_btn.action_url.clone(),
                    });
                }
                message_json["actionCard"]["btns"] = serde_json::to_value(btns)?;
            }
        }
        if dingtalk_message.at_all || !dingtalk_message.at_mobiles.is_empty() {
            if let Some(m) = message_json.as_object_mut() {
                let mut at_mobiles: Vec<Value> = vec![];
                for m in &dingtalk_message.at_mobiles {
                    at_mobiles.push(Value::String(m.clone()));
                }
                let mut at_map = serde_json::Map::new();
                at_map.insert("atMobiles".into(), Value::Array(at_mobiles));
                at_map.insert("isAtAll".into(), Value::Bool(dingtalk_message.at_all));

                m.insert("at".into(), Value::Object(at_map));
            }
        }
        self.send(&serde_json::to_string(&message_json)?).await
    }

    /// Send text message
    pub async fn send_text(&self, text_message: &str) -> XResult<()> {
        self.send_message(DingTalkMessage::new_text(text_message))
            .await
    }

    /// Send markdown message
    pub async fn send_markdown(&self, title: &str, text: &str) -> XResult<()> {
        self.send_message(DingTalkMessage::new_markdown(title, text))
            .await
    }

    /// Send link message
    pub async fn send_link(
        &self,
        link_title: &str,
        link_text: &str,
        link_pic_url: &str,
        link_message_url: &str,
    ) -> XResult<()> {
        self.send_message(DingTalkMessage::new_link(
            link_title,
            link_text,
            link_pic_url,
            link_message_url,
        ))
        .await
    }

    /// Direct send JSON message
    pub async fn send(&self, json_message: &str) -> XResult<()> {
        let client = reqwest::Client::new();
        let response = match client
            .post(&self.generate_signed_url()?)
            .header(CONTENT_TYPE, APPLICATION_JSON_UTF8)
            .body(json_message.as_bytes().to_vec())
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return Err(Box::new(Error::new(
                    ErrorKind::Other,
                    format!("Unknown error: {}", e),
                )) as Box<dyn std::error::Error>);
            }
        };

        match response.status().as_u16() {
            200_u16 => Ok(()),
            _ => Err(Box::new(Error::new(
                ErrorKind::Other,
                format!("Unknown status: {}", response.status().as_u16()),
            )) as Box<dyn std::error::Error>),
        }
    }

    /// Generate signed dingtalk webhook URL
    pub fn generate_signed_url(&self) -> XResult<String> {
        if !self.direct_url.is_empty() {
            return Ok(self.direct_url.clone());
        }
        let mut signed_url = String::with_capacity(1024);
        signed_url.push_str(&self.default_webhook_url);

        if self.default_webhook_url.ends_with('?') {
            // Just Ok
        } else if self.default_webhook_url.contains('?') {
            if !self.default_webhook_url.ends_with('&') {
                signed_url.push('&');
            }
        } else {
            signed_url.push('?');
        }

        match self.dingtalk_type {
            DingTalkType::DingTalk => signed_url.push_str("access_token="),
            DingTalkType::WeChatWork => signed_url.push_str("key="),
        }
        signed_url.push_str(&urlencoding::encode(&self.access_token));

        if !self.sec_token.is_empty() {
            let timestamp = &format!(
                "{}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );
            let timestamp_and_secret = &format!("{}\n{}", timestamp, self.sec_token);

            // Use the recommended new method for base64 encoding
            let hmac_sha256 = base64::engine::general_purpose::STANDARD.encode(
                &calc_hmac_sha256(self.sec_token.as_bytes(), timestamp_and_secret.as_bytes())?[..],
            );

            signed_url.push_str("&timestamp=");
            signed_url.push_str(timestamp);
            signed_url.push_str("&sign=");
            signed_url.push_str(&urlencoding::encode(&hmac_sha256));
        }

        Ok(signed_url)
    }

}

/// calc hmac_sha256 digest
fn calc_hmac_sha256(key: &[u8], message: &[u8]) -> XResult<Vec<u8>> {
    let mut mac = match Hmac::<Sha256>::new_from_slice(key) {
        Ok(m) => m,
        Err(e) => {
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                format!("Hmac error: {}", e),
            )));
        }
    };
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}
