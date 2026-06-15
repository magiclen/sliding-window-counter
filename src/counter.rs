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
///
/// # Consistency
///
/// Each `record` call uses Moka's key-level entry API before updating the per-key event window, so concurrent `record` calls for the same key do not overwrite each other or write only to a detached stale handle.
///
/// However, this counter is still best-effort in broader terms: counts may be lower than the true global count when the process restarts, when multiple application instances share traffic, or when a key is genuinely evicted from the cache (e.g. capacity pressure or time-to-idle expiry of an idle key).
///
/// Use this type for local, bounded, in-memory counting, not as the only strict security limit for login or payment flows.
pub struct SlidingWindowCounter<K, C = SystemClock> {
    cache:              Cache<K, Arc<Mutex<SlidingWindow>>>,
    window:             Duration,
    max_events_per_key: usize,
    clock:              C,
}

impl<K, C> Clone for SlidingWindowCounter<K, C>
where
    C: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            cache:              self.cache.clone(),
            window:             self.window,
            max_events_per_key: self.max_events_per_key,
            clock:              self.clock.clone(),
        }
    }
}

impl<K, C> Debug for SlidingWindowCounter<K, C> {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SlidingWindowCounter")
            .field("window", &self.window)
            .field("max_events_per_key", &self.max_events_per_key)
            .field("*entry_count", &self.cache.entry_count())
            .finish_non_exhaustive()
    }
}

impl<K> SlidingWindowCounter<K, SystemClock>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
{
    /// Creates a counter that uses [`Instant::now`](std::time::Instant::now) as its time source.
    ///
    /// `window` is the time range used for counting recent events, `max_keys` limits how many keys can stay in the cache, and `max_events_per_key` limits how many event timestamps are stored for one key.
    ///
    /// # Panics
    ///
    /// Panics if `window`, `max_keys`, or `max_events_per_key` is zero.
    #[inline]
    #[must_use]
    pub fn new(window: Duration, max_keys: u64, max_events_per_key: usize) -> Self {
        Self::with_clock(window, max_keys, max_events_per_key, SystemClock)
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
    /// Panics if `window`, `max_keys`, or `max_events_per_key` is zero.
    #[must_use]
    pub fn with_clock(
        window: Duration,
        max_keys: u64,
        max_events_per_key: usize,
        clock: C,
    ) -> Self {
        assert!(!window.is_zero(), "window must be greater than zero");
        assert!(max_keys > 0, "max_keys must be greater than zero");
        assert!(max_events_per_key > 0, "max_events_per_key must be greater than zero");

        Self {
            cache: Cache::builder().time_to_idle(window).max_capacity(max_keys).build(),
            window,
            max_events_per_key,
            clock,
        }
    }

    /// Records one event for `key` and returns the current count for that key.
    ///
    /// Returns `None` when this record exceeds `max_events_per_key`. In that case the newest event is still stored, but the oldest stored event for the key is removed.
    pub fn record(&self, key: K) -> Option<usize> {
        let now = self.clock.now();
        let window = self.window;
        let max_events_per_key = self.max_events_per_key;
        let mut result = None;

        self.cache.entry(key).and_upsert_with(|maybe_entry| {
            let arc = match maybe_entry {
                Some(entry) => entry.into_value(),
                None => Arc::new(Mutex::new(SlidingWindow::default())),
            };

            result = arc.lock().record(now, window, max_events_per_key);

            arc
        });

        result
    }

    /// Returns the current stored count for `key` without adding a new event.
    ///
    /// The returned count is capped by `max_events_per_key`.
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

    /// Returns the maximum number of event timestamps stored for one key.
    #[inline]
    pub const fn max_events_per_key(&self) -> usize {
        self.max_events_per_key
    }
}
