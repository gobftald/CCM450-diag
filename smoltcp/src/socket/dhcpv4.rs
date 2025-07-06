// 5
use crate::time::Duration;

/// Timeout and retry configuration.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 119
pub struct RetryConfig {
    pub discover_timeout: Duration,
    /// The REQUEST timeout doubles every 2 tries.
    pub initial_request_timeout: Duration,
    pub request_retries: u16,
    pub min_renew_timeout: Duration,
    /// An upper bound on how long to wait between retrying a renew or rebind.
    ///
    /// Set this to [`Duration::MAX`] if you don't want to impose an upper bound.
    pub max_renew_timeout: Duration,
}

// 131
impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            discover_timeout: Duration::from_secs(10),
            initial_request_timeout: Duration::from_secs(5),
            request_retries: 5,
            min_renew_timeout: Duration::from_secs(60),
            max_renew_timeout: Duration::MAX,
        }
    }
}
