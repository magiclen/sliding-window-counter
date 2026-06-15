use std::{
    fmt::{self, Debug, Formatter},
    hash::Hash,
    sync::Arc,
    time::Duration,
};

use moka::sync::Cache;
use parking_lot::Mutex;

use crate::{
    clock::{Clock, SystemClock},
    window::SlidingWindow,
};

/// Counts recent events for each key within a fixed sliding time window.
///
/// The counter keeps one small event queue per key and automatically evicts idle keys from the cache.
/// Cloning a counter is cheap and shares the same stored counts.
pub struct SlidingWindowCounter<K, C = SystemClock> {
    cache:  Cache<K, Arc<Mutex<SlidingWindow>>>,
    window: Duration,
    clock:  C,
}

impl<K, C> Clone for SlidingWindowCounter<K, C>
where
    C: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(), window: self.window, clock: self.clock.clone()
        }
    }
}

impl<K, C> Debug for SlidingWindowCounter<K, C>
where
    K: Debug + Eq + Hash + Send + Sync + 'static,
    C: Debug,
{
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SlidingWindowCounter")
            .field("cache", &self.cache)
            .field("window", &self.window)
            .field("clock", &self.clock)
            .finish()
    }
}

impl<K> SlidingWindowCounter<K, SystemClock>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
{
    /// Creates a counter that uses [`Instant::now`](std::time::Instant::now) as its time source.
    ///
    /// `window` is the time range used for counting recent events, and `max_keys` limits how many keys can stay in the cache.
    ///
    /// # Panics
    ///
    /// Panics if `window` is zero or if `max_keys` is zero.
    #[inline]
    #[must_use]
    pub fn new(window: Duration, max_keys: u64) -> Self {
        Self::with_clock(window, max_keys, SystemClock)
    }
}

impl<K, C> SlidingWindowCounter<K, C>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    C: Clock,
{
    /// Creates a counter with a custom clock.
    ///
    /// This is mainly useful for tests or for applications that already have their own time source.
    ///
    /// # Panics
    ///
    /// Panics if `window` is zero or if `max_keys` is zero.
    #[must_use]
    pub fn with_clock(window: Duration, max_keys: u64, clock: C) -> Self {
        assert!(!window.is_zero(), "window must be greater than zero");
        assert!(max_keys > 0, "max_keys must be greater than zero");

        Self {
            cache: Cache::builder().time_to_idle(window).max_capacity(max_keys).build(),
            window,
            clock,
        }
    }

    /// Records one event for `key` and returns the current count for that key.
    pub fn record(&self, key: K) -> usize {
        let now = self.clock.now();
        let window = self.cache.get_with(key, || Arc::new(Mutex::new(SlidingWindow::default())));
        let mut window = window.lock();

        window.record(now, self.window)
    }

    /// Returns the current count for `key` without adding a new event.
    pub fn count(&self, key: &K) -> usize {
        let now = self.clock.now();
        let Some(window) = self.cache.get(key) else {
            return 0;
        };
        let mut window = window.lock();

        window.count(now, self.window)
    }

    /// Returns the configured sliding time window.
    #[inline]
    pub const fn window(&self) -> Duration {
        self.window
    }
}
