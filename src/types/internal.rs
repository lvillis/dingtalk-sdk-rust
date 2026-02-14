use serde::{Deserialize, Serialize};

use crate::types::enterprise::ApprovalProcessInstance;
use crate::types::webhook::{ActionCardButton, FeedCardLink};

#[derive(Serialize)]
#[serde(tag = "msgtype")]
pub(crate) enum WebhookMessage {
    #[serde(rename = "text")]
    Text {
        text: TextContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        at: Option<At>,
    },
    #[serde(rename = "link")]
    Link {
        link: LinkContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        at: Option<At>,
    },
    #[serde(rename = "markdown")]
    Markdown {
        markdown: MarkdownContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        at: Option<At>,
    },
    #[serde(rename = "actionCard")]
    ActionCard {
        #[serde(rename = "actionCard")]
        action_card: ActionCardContent,
    },
    #[serde(rename = "feedCard")]
    FeedCard {
        #[serde(rename = "feedCard")]
        feed_card: FeedCardContent,
    },
}

#[derive(Serialize)]
pub(crate) struct TextContent {
    pub(crate) content: String,
}

#[derive(Serialize)]
pub(crate) struct LinkContent {
    pub(crate) title: String,
    pub(crate) text: String,
    #[serde(rename = "messageUrl")]
    pub(crate) message_url: String,
    #[serde(rename = "picUrl", skip_serializing_if = "Option::is_none")]
    pub(crate) pic_url: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct MarkdownContent {
    pub(crate) title: String,
    pub(crate) text: String,
}

#[derive(Serialize)]
pub(crate) struct ActionCardContent {
    pub(crate) title: String,
    pub(crate) text: String,
    #[serde(rename = "btnOrientation", skip_serializing_if = "Option::is_none")]
    pub(crate) btn_orientation: Option<String>,
    #[serde(rename = "singleTitle", skip_serializing_if = "Option::is_none")]
    pub(crate) single_title: Option<String>,
    #[serde(rename = "singleURL", skip_serializing_if = "Option::is_none")]
    pub(crate) single_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) btns: Option<Vec<ActionCardButton>>,
}

#[derive(Serialize)]
pub(crate) struct FeedCardContent {
    pub(crate) links: Vec<FeedCardLink>,
}

#[derive(Serialize)]
pub(crate) struct At {
    #[serde(rename = "atMobiles", skip_serializing_if = "Option::is_none")]
    pub(crate) at_mobiles: Option<Vec<String>>,
    #[serde(rename = "atUserIds", skip_serializing_if = "Option::is_none")]
    pub(crate) at_user_ids: Option<Vec<String>>,
    #[serde(rename = "isAtAll", skip_serializing_if = "Option::is_none")]
    pub(crate) is_at_all: Option<bool>,
}

pub(crate) fn build_at(
    at_mobiles: Option<Vec<String>>,
    at_user_ids: Option<Vec<String>>,
    is_at_all: Option<bool>,
) -> Option<At> {
    if at_mobiles.is_some() || at_user_ids.is_some() || is_at_all.is_some() {
        Some(At {
            at_mobiles,
            at_user_ids,
            is_at_all,
        })
    } else {
        None
    }
}

#[derive(Serialize)]
pub(crate) struct MsgParam {
    pub(crate) title: String,
    pub(crate) text: String,
}

pub(crate) fn serialize_to_json_string<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: ?Sized + Serialize,
{
    let serialized = serde_json::to_string(value).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&serialized)
}

#[derive(Serialize)]
pub(crate) struct GroupMessageRequest<'a> {
    #[serde(rename = "msgParam", serialize_with = "serialize_to_json_string")]
    pub(crate) msg_param: MsgParam,
    #[serde(rename = "msgKey")]
    pub(crate) msg_key: &'a str,
    #[serde(rename = "robotCode")]
    pub(crate) robot_code: &'a str,
    #[serde(rename = "openConversationId")]
    pub(crate) open_conversation_id: &'a str,
}

#[derive(Serialize)]
pub(crate) struct OtoMessageRequest<'a> {
    #[serde(rename = "msgParam", serialize_with = "serialize_to_json_string")]
    pub(crate) msg_param: MsgParam,
    #[serde(rename = "msgKey")]
    pub(crate) msg_key: &'a str,
    #[serde(rename = "robotCode")]
    pub(crate) robot_code: &'a str,
    #[serde(rename = "userIds", skip_serializing_if = "Vec::is_empty")]
    pub(crate) user_ids: Vec<&'a str>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GetTokenResponse {
    pub(crate) errcode: i64,
    pub(crate) errmsg: String,
    pub(crate) access_token: Option<String>,
    pub(crate) expires_in: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StandardApiResponse {
    pub(crate) errcode: Option<i64>,
    pub(crate) errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TopApiResultResponse<T> {
    pub(crate) errcode: i64,
    pub(crate) errmsg: String,
    pub(crate) result: Option<T>,
    pub(crate) request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TopApiSimpleResponse {
    pub(crate) errcode: i64,
    pub(crate) errmsg: String,
    pub(crate) request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApprovalCreateProcessInstanceResponse {
    pub(crate) errcode: i64,
    pub(crate) errmsg: String,
    pub(crate) process_instance_id: Option<String>,
    pub(crate) request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApprovalGetProcessInstanceResponse {
    pub(crate) errcode: i64,
    pub(crate) errmsg: String,
    pub(crate) process_instance: Option<ApprovalProcessInstance>,
    pub(crate) request_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_get_process_instance_response_parses_process_instance_field() {
        let raw = r#"{
            "errcode": 0,
            "errmsg": "ok",
            "process_instance": {
                "process_instance_id": "PROC-123"
            },
            "request_id": "REQ-1"
        }"#;

        let parsed: ApprovalGetProcessInstanceResponse =
            serde_json::from_str(raw).expect("response should deserialize");

        assert_eq!(parsed.errcode, 0);
        assert_eq!(
            parsed
                .process_instance
                .and_then(|instance| instance.process_instance_id)
                .as_deref(),
            Some("PROC-123")
        );
    }
}
