//! Time units
//!
//! NOTE: the Instant and Duration types are lifted from smoltcp:
//! https://github.com/m-labs/smoltcp
//!
//! - [Instant] is used to represent absolute time.
//! - [Duration] is used to represet relative time.
//!
//! [Instant]: struct.Instant.html
//! [Duration]: struct.Duration.html

use core::{fmt, ops};

/// Bits per second
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Bps(pub u32);

/// Hertz
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Hertz(pub u32);

/// KiloHertz
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct KiloHertz(pub u32);

/// MegaHertz
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MegaHertz(pub u32);

/// MilliSeconds
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MilliSeconds(pub u32);

/// Extension trait that adds convenience methods to the `u32` type
pub trait U32Ext {
    /// Wrap in `Bps`
    fn bps(self) -> Bps;

    /// Wrap in `Hertz`
    fn hz(self) -> Hertz;

    /// Wrap in `KiloHertz`
    fn khz(self) -> KiloHertz;

    /// Wrap in `MegaHertz`
    fn mhz(self) -> MegaHertz;

    /// Wrap in `MilliSeconds`
    fn ms(self) -> MilliSeconds;
}

impl U32Ext for u32 {
    fn bps(self) -> Bps {
        Bps(self)
    }

    fn hz(self) -> Hertz {
        Hertz(self)
    }

    fn khz(self) -> KiloHertz {
        KiloHertz(self)
    }

    fn mhz(self) -> MegaHertz {
        MegaHertz(self)
    }

    fn ms(self) -> MilliSeconds {
        MilliSeconds(self)
    }
}

impl Into<Hertz> for KiloHertz {
    fn into(self) -> Hertz {
        Hertz(self.0 * 1_000)
    }
}

impl Into<Hertz> for MegaHertz {
    fn into(self) -> Hertz {
        Hertz(self.0 * 1_000_000)
    }
}

impl Into<KiloHertz> for MegaHertz {
    fn into(self) -> KiloHertz {
        KiloHertz(self.0 * 1_000)
    }
}

/// A representation of an absolute time value.
///
/// The `Instant` type is a wrapper around a `i64` value that
/// represents a number of milliseconds, monotonically increasing
/// since an arbitrary moment in time, such as system startup.
///
/// * A value of `0` is inherently arbitrary.
/// * A value less than `0` indicates a time before the starting point.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    pub millis: i64,
}

impl Instant {
    /// Create a new `Instant` from a number of milliseconds.
    pub fn from_millis<T: Into<i64>>(millis: T) -> Instant {
        Instant {
            millis: millis.into(),
        }
    }

    /// Create a new `Instant` from a number of seconds.
    pub fn from_secs<T: Into<i64>>(secs: T) -> Instant {
        Instant {
            millis: secs.into() * 1000,
        }
    }

    /// The fractional number of milliseconds that have passed
    /// since the beginning of time.
    pub fn millis(&self) -> i64 {
        self.millis % 1000
    }

    /// The number of whole seconds that have passed since the
    /// beginning of time.
    pub fn secs(&self) -> i64 {
        self.millis / 1000
    }

    /// The total number of milliseconds that have passed since
    /// the biginning of time.
    pub fn total_millis(&self) -> i64 {
        self.millis
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}s", self.secs(), self.millis())
    }
}

impl ops::Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Instant {
        Instant::from_millis(self.millis + rhs.total_millis() as i64)
    }
}

impl ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        self.millis += rhs.total_millis() as i64;
    }
}

impl ops::Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Duration) -> Instant {
        Instant::from_millis(self.millis - rhs.total_millis() as i64)
    }
}

impl ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        self.millis -= rhs.total_millis() as i64;
    }
}

impl ops::Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Duration {
        Duration::from_millis((self.millis - rhs.millis).abs() as u64)
    }
}

/// A relative amount of time.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    pub millis: u64,
}

impl Duration {
    /// Create a new `Duration` from a number of milliseconds.
    pub const fn from_millis(millis: u64) -> Duration {
        Duration { millis }
    }

    /// Create a new `Instant` from a number of seconds.
    pub const fn from_secs(secs: u64) -> Duration {
        Duration {
            millis: secs * 1000,
        }
    }

    /// The fractional number of milliseconds in this `Duration`.
    pub fn millis(&self) -> u64 {
        self.millis % 1000
    }

    /// The number of whole seconds in this `Duration`.
    pub fn secs(&self) -> u64 {
        self.millis / 1000
    }

    /// The total number of milliseconds in this `Duration`.
    pub fn total_millis(&self) -> u64 {
        self.millis
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:03}s", self.secs(), self.millis())
    }
}

impl ops::Add<Duration> for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        Duration::from_millis(self.millis + rhs.total_millis())
    }
}

impl ops::AddAssign<Duration> for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        self.millis += rhs.total_millis();
    }
}

impl ops::Sub<Duration> for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Duration {
        Duration::from_millis(
            self.millis
                .checked_sub(rhs.total_millis())
                .expect("overflow when subtracting durations"),
        )
    }
}

impl ops::SubAssign<Duration> for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        self.millis = self
            .millis
            .checked_sub(rhs.total_millis())
            .expect("overflow when subtracting durations");
    }
}

impl ops::Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        Duration::from_millis(self.millis * rhs as u64)
    }
}

impl ops::MulAssign<u32> for Duration {
    fn mul_assign(&mut self, rhs: u32) {
        self.millis *= rhs as u64;
    }
}

impl ops::Div<u32> for Duration {
    type Output = Duration;

    fn div(self, rhs: u32) -> Duration {
        Duration::from_millis(self.millis / rhs as u64)
    }
}

impl ops::DivAssign<u32> for Duration {
    fn div_assign(&mut self, rhs: u32) {
        self.millis /= rhs as u64;
    }
}

impl From<::core::time::Duration> for Duration {
    fn from(other: ::core::time::Duration) -> Duration {
        Duration::from_millis(other.as_secs() * 1000 + (other.subsec_nanos() / 1_000_000) as u64)
    }
}

impl Into<::core::time::Duration> for Duration {
    fn into(self) -> ::core::time::Duration {
        ::core::time::Duration::from_millis(self.total_millis())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_instant_ops() {
        // std::ops::Add
        assert_eq!(
            Instant::from_millis(4) + Duration::from_millis(6),
            Instant::from_millis(10)
        );
        // std::ops::Sub
        assert_eq!(
            Instant::from_millis(7) - Duration::from_millis(5),
            Instant::from_millis(2)
        );
    }

    #[test]
    fn test_instant_getters() {
        let instant = Instant::from_millis(5674);
        assert_eq!(instant.secs(), 5);
        assert_eq!(instant.millis(), 674);
        assert_eq!(instant.total_millis(), 5674);
    }

    #[test]
    fn test_instant_display() {
        assert_eq!(format!("{}", Instant::from_millis(5674)), "5.674s");
        assert_eq!(format!("{}", Instant::from_millis(5000)), "5.0s");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_instant_conversions() {
        let mut epoc: ::std::time::SystemTime = Instant::from_millis(0).into();
        assert_eq!(
            Instant::from(::std::time::UNIX_EPOCH),
            Instant::from_millis(0)
        );
        assert_eq!(epoc, ::std::time::UNIX_EPOCH);
        epoc = Instant::from_millis(2085955200i64 * 1000).into();
        assert_eq!(
            epoc,
            ::std::time::UNIX_EPOCH + ::std::time::Duration::from_secs(2085955200)
        );
    }

    #[test]
    fn test_duration_ops() {
        // std::ops::Add
        assert_eq!(
            Duration::from_millis(40) + Duration::from_millis(2),
            Duration::from_millis(42)
        );
        // std::ops::Sub
        assert_eq!(
            Duration::from_millis(555) - Duration::from_millis(42),
            Duration::from_millis(513)
        );
        // std::ops::Mul
        assert_eq!(Duration::from_millis(13) * 22, Duration::from_millis(286));
        // std::ops::Div
        assert_eq!(Duration::from_millis(53) / 4, Duration::from_millis(13));
    }

    #[test]
    fn test_duration_assign_ops() {
        let mut duration = Duration::from_millis(4735);
        duration += Duration::from_millis(1733);
        assert_eq!(duration, Duration::from_millis(6468));
        duration -= Duration::from_millis(1234);
        assert_eq!(duration, Duration::from_millis(5234));
        duration *= 4;
        assert_eq!(duration, Duration::from_millis(20936));
        duration /= 5;
        assert_eq!(duration, Duration::from_millis(4187));
    }

    #[test]
    #[should_panic(expected = "overflow when subtracting durations")]
    fn test_sub_from_zero_overflow() {
        let _ = Duration::from_millis(0) - Duration::from_millis(1);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn test_div_by_zero() {
        let _ = Duration::from_millis(4) / 0;
    }

    #[test]
    fn test_duration_getters() {
        let instant = Duration::from_millis(4934);
        assert_eq!(instant.secs(), 4);
        assert_eq!(instant.millis(), 934);
        assert_eq!(instant.total_millis(), 4934);
    }

    #[test]
    fn test_duration_conversions() {
        let mut std_duration = ::core::time::Duration::from_millis(4934);
        let duration: Duration = std_duration.into();
        assert_eq!(duration, Duration::from_millis(4934));
        assert_eq!(Duration::from(std_duration), Duration::from_millis(4934));

        std_duration = duration.into();
        assert_eq!(std_duration, ::core::time::Duration::from_millis(4934));
    }
}
