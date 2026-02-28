use std::time::Duration;

/// Defines how a command or operation should be retried upon failure.
pub trait RetryPolicy: Send + Sync {
    /// Determines whether a retry should be attempted and how long to wait.
    /// Returns `Some(Duration)` to wait before retrying, or `None` if retries are exhausted.
    fn should_retry(&self, attempt: u32, error: &crate::error::CoreError) -> Option<Duration>;
}

/// A policy that never retries.
pub struct NoRetry;

impl RetryPolicy for NoRetry {
    fn should_retry(&self, _attempt: u32, _error: &crate::error::CoreError) -> Option<Duration> {
        None
    }
}

/// A policy that retries with exponential backoff.
pub struct ExponentialBackoff {
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
}

impl ExponentialBackoff {
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
        }
    }

    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    pub fn with_max_delay(mut self, max: Duration) -> Self {
        self.max_delay = max;
        self
    }

    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }
}

impl RetryPolicy for ExponentialBackoff {
    fn should_retry(&self, attempt: u32, _error: &crate::error::CoreError) -> Option<Duration> {
        if attempt >= self.max_retries {
            return None;
        }

        let delay_ms = (self.initial_delay.as_millis() as f64 * self.multiplier.powi(attempt as i32)) as u64;
        let delay = Duration::from_millis(delay_ms).min(self.max_delay);
        
        Some(delay)
    }
}
