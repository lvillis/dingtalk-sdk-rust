use std::time::Duration;

/// Retry policy configuration for SDK HTTP requests.
///
/// `max_retries` means retry attempts after the first request.
/// For example, `max_retries = 2` allows up to 3 total attempts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryConfig {
    /// Maximum retry attempts after the initial request.
    pub max_retries: usize,
    /// Base exponential backoff duration.
    pub base_backoff: Duration,
}

impl RetryConfig {
    /// Creates a retry configuration with explicit values.
    #[must_use]
    pub fn new(max_retries: usize, base_backoff: Duration) -> Self {
        Self {
            max_retries,
            base_backoff,
        }
    }

    /// Returns a conservative default policy.
    #[must_use]
    pub fn standard() -> Self {
        Self {
            max_retries: 2,
            base_backoff: Duration::from_millis(200),
        }
    }

    /// Sets max retry attempts.
    #[must_use]
    pub fn max_retries(mut self, value: usize) -> Self {
        self.max_retries = value;
        self
    }

    /// Sets base backoff duration.
    #[must_use]
    pub fn base_backoff(mut self, value: Duration) -> Self {
        self.base_backoff = value;
        self
    }
}
