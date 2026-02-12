use serde::Serialize;

/// Multi-button `actionCard` button item.
#[derive(Debug, Clone, Serialize)]
pub struct ActionCardButton {
    /// Button title.
    pub title: String,
    /// Redirect URL after clicking the button.
    #[serde(rename = "actionURL")]
    pub action_url: String,
}

impl ActionCardButton {
    /// Creates an action-card button.
    #[must_use]
    pub fn new(title: impl Into<String>, action_url: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            action_url: action_url.into(),
        }
    }
}

/// `feedCard` link item.
#[derive(Debug, Clone, Serialize)]
pub struct FeedCardLink {
    /// Link title.
    pub title: String,
    /// Message jump URL.
    #[serde(rename = "messageURL")]
    pub message_url: String,
    /// Image URL.
    #[serde(rename = "picURL")]
    pub pic_url: String,
}

impl FeedCardLink {
    /// Creates a feed-card link.
    #[must_use]
    pub fn new(
        title: impl Into<String>,
        message_url: impl Into<String>,
        pic_url: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            message_url: message_url.into(),
            pic_url: pic_url.into(),
        }
    }
}
