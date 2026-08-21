use core::clone::Clone;
use core::cmp::PartialEq;
/// the timestamp is a simple u64 representing the number of seconds since the epoch (January 1, 1970).
pub type Timestamp = u64;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SecurityState {
    Confiance, // The system is in a "Confiance" state, meaning that the rate of modifications is within acceptable limits.
    Phoenix, // Attack detected, we are in a "Phoenix" state, where the system is under attack and we need to take action to protect the data structure.
}

/// The `PhoenixMonitor` struct is responsible for monitoring the rate of modifications to a file or data structure.
pub struct PhoenixMonitor {
    /// Le number of tolerated modifications within the time window.
    pub max_authorized: u32,
    /// The time window in seconds.
    pub time: u64,
    // Internal state
    current_modifications: u32,
    /// The timestamp of the start of the current monitoring window.
    start: Timestamp,
}

impl PhoenixMonitor {
    /// Creates a new `PhoenixMonitor` with the specified maximum number of authorized modifications and time window.
    ///
    /// # Arguments
    /// * `max_authorized` - The maximum number of modifications allowed within the time window.
    /// * `time` - The time window in seconds.
    ///
    /// # Returns
    /// * Returns a new instance of `PhoenixMonitor`.
    ///
    /// # Examples
    /// ```no_run
    /// use ra::tree::metrics::PhoenixMonitor;
    /// let monitor = PhoenixMonitor::new(5, 60);
    /// ```
    #[must_use]
    pub const fn new(max_authorized: u32, time: u64) -> Self {
        Self {
            max_authorized,
            time,
            current_modifications: 0,
            start: 0,
        }
    }

    /// Called by the kernel whenever a file is modified
    ///
    /// # Arguments
    /// * `current_time` - The current timestamp in seconds since the epoch.
    /// # Returns
    ///
    /// A `SecurityState` indicating whether the system is in a "Confiance" state or a "Phoenix" state.
    ///
    /// # Examples
    /// ```no_run
    /// use ra::tree::metrics::{PhoenixMonitor, SecurityState};
    /// let mut monitor = PhoenixMonitor::new(5, 60);
    /// let state = monitor.save(10);
    /// assert_eq!(state, SecurityState::Confiance);
    /// ```
    ///
    #[must_use]
    pub const fn save(&mut self, current_time: Timestamp) -> SecurityState {
        // If the time has exceeded our monitoring window (e.g., 1 minute has passed)
        if current_time - self.start > self.time {
            // We reset the counter, the human has returned to a normal pace
            self.start = current_time;
            self.current_modifications = 1;
            return SecurityState::Confiance;
        }

        self.current_modifications += 1;

        if self.current_modifications > self.max_authorized {
            SecurityState::Phoenix
        } else {
            SecurityState::Confiance
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phoenix_monitor_creation() {
        let monitor = PhoenixMonitor::new(5, 60);
        assert_eq!(monitor.max_authorized, 5);
        assert_eq!(monitor.time, 60);
        assert_eq!(monitor.current_modifications, 0);
        assert_eq!(monitor.start, 0);
    }

    #[test]
    fn test_phoenix_monitor_save() {
        let mut monitor = PhoenixMonitor::new(5, 60);
        let state = monitor.save(10);
        assert_eq!(state, SecurityState::Confiance);
    }
    #[test]
    fn test_phoenix_monitor_save_exceeding_max() {
        let mut monitor = PhoenixMonitor::new(5, 60);
        for i in 0..6 {
            let state = monitor.save(i);
            if i < 5 {
                assert_eq!(state, SecurityState::Confiance);
            } else {
                assert_eq!(state, SecurityState::Phoenix);
            }
        }
    }

    #[test]
    fn test_phoenix_monitor_save_reset_after_time_window() {
        let mut monitor = PhoenixMonitor::new(5, 60);
        for i in 0..5 {
            let state = monitor.save(i);
            assert_eq!(state, SecurityState::Confiance);
        }
    }
}
