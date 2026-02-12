use std::fmt;

/// Enterprise app credentials (`appkey` + `appsecret`).
///
/// `Debug` output redacts `appsecret`.
#[derive(Clone)]
pub struct AppCredentials {
    appkey: String,
    appsecret: String,
}

impl AppCredentials {
    /// Creates credentials from app key and app secret.
    #[must_use]
    pub fn new(appkey: impl Into<String>, appsecret: impl Into<String>) -> Self {
        Self {
            appkey: appkey.into(),
            appsecret: appsecret.into(),
        }
    }

    /// Returns the application key.
    #[must_use]
    pub fn appkey(&self) -> &str {
        &self.appkey
    }

    /// Returns the application secret.
    #[must_use]
    pub fn appsecret(&self) -> &str {
        &self.appsecret
    }
}

impl fmt::Debug for AppCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppCredentials")
            .field("appkey", &self.appkey)
            .field("appsecret", &"<redacted>")
            .finish()
    }
}
