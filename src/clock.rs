use std::time::Instant;

/// A source of monotonic time for a [`SlidingWindowCounter`](crate::SlidingWindowCounter).
///
/// Implement this trait when an application or a test needs to control how time moves.
pub trait Clock: Clone + Send + Sync + 'static {
    /// Returns the current monotonic instant.
    fn now(&self) -> Instant;
}

/// The default clock that reads time from [`Instant::now`].
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    #[inline]
    fn now(&self) -> Instant {
        Instant::now()
    }
}
