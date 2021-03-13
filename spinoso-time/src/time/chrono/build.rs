use std::time::Duration;
use chrono::prelude::*;

use crate::time::chrono::{Offset, Time};
use crate::NANOS_IN_SECOND;

impl Default for Time {
    /// The zero-argument [`Time#new`] constructor creates a local time set to
    /// the current system time.
    ///
    /// [`Time#new`]: https://ruby-doc.org/core-2.6.3/Time.html#method-c-new
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    /// Creates a new `Time` object for the current time with a local offset.
    ///
    /// # Examples
    ///
    /// ```
    /// use spinoso_time::Time;
    /// let now = Time::new();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::now()
    }

    /// Creates a new `Time` object for the current time with a local offset.
    ///
    /// This is same as [`new`](Self::new).
    ///
    /// # Examples
    ///
    /// ```
    /// use spinoso_time::Time;
    /// let now = Time::now();
    /// ```
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        let now = Utc::now();
        let offset = Offset::Local;
        let timestamp = now.timestamp();
        let sub_second_nanos = now.timestamp_subsec_nanos();
        Self {
            timestamp,
            sub_second_nanos,
            offset,
        }
    }

    /// TODO: Add Docs
    #[inline]
    #[must_use]
    pub fn at(seconds: f64, sub_second_nanos: f64) -> Self {
        let nanos_in_second = NANOS_IN_SECOND as f64;
        let offset = Offset::Local;
        let mut timestamp = seconds;
        let mut sub_second_nanos = sub_second_nanos;

        // If seconds is fractional, add fractional seconds to sub_second_nanos
        if timestamp.fract() != 0.0 {
            let nanos_from_fraction = seconds.fract() * nanos_in_second;
            timestamp = timestamp.floor();
            sub_second_nanos += nanos_from_fraction;
        }

        // If sub_second_nanos is more than 1 second, move overflow into seconds
        if sub_second_nanos.abs() > nanos_in_second {
            timestamp += sub_second_nanos / nanos_in_second;
            sub_second_nanos %= nanos_in_second;
        }
        
        let timestamp = timestamp as i64;
        let sub_second_nanos = sub_second_nanos as u32;

        Self {
            timestamp,
            sub_second_nanos,
            offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::time::chrono::{Offset, Time};

    #[test]
    fn time_new_is_local_offset() {
        let time = Time::new();
        assert_eq!(time.offset, Offset::Local);
    }

    #[test]
    fn time_now_is_local_offset() {
        let time = Time::now();
        assert_eq!(time.offset, Offset::Local);
    }

    #[test]
    fn time_default_is_local_offset() {
        let time = Time::default();
        assert_eq!(time.offset, Offset::Local);
    }
}
