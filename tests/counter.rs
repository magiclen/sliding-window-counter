use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use parking_lot::Mutex;
use sliding_window_counter::{Clock, SlidingWindowCounter};

#[test]
fn record_returns_the_current_count_for_one_key() {
    let counter = SlidingWindowCounter::new(Duration::from_secs(10), 10);

    assert_eq!(counter.record(1), 1);
    assert_eq!(counter.record(1), 2);
    assert_eq!(counter.count(&1), 2);
}

#[test]
fn keys_are_counted_independently() {
    let counter = SlidingWindowCounter::new(Duration::from_secs(10), 10);

    assert_eq!(counter.record(1), 1);
    assert_eq!(counter.record(2), 1);
    assert_eq!(counter.record(1), 2);
    assert_eq!(counter.count(&2), 1);
}

#[test]
fn count_reads_without_recording_a_new_event() {
    let counter = SlidingWindowCounter::new(Duration::from_secs(10), 10);

    assert_eq!(counter.count(&1), 0);
    assert_eq!(counter.record(1), 1);
    assert_eq!(counter.count(&1), 1);
    assert_eq!(counter.count(&1), 1);
}

#[test]
fn expired_events_are_removed_before_counting() {
    #[derive(Clone)]
    struct ManualClock(Arc<Mutex<Instant>>);

    impl ManualClock {
        fn new() -> Self {
            Self(Arc::new(Mutex::new(Instant::now())))
        }

        fn advance(&self, duration: Duration) {
            let mut now = self.0.lock();
            *now = now
                .checked_add(duration)
                .expect("manual clock should stay in the valid Instant range");
        }
    }

    impl Clock for ManualClock {
        fn now(&self) -> Instant {
            *self.0.lock()
        }
    }

    let clock = ManualClock::new();
    let counter = SlidingWindowCounter::with_clock(Duration::from_secs(10), 10, clock.clone());

    assert_eq!(counter.record(1), 1);
    clock.advance(Duration::from_secs(5));
    assert_eq!(counter.record(1), 2);
    clock.advance(Duration::from_secs(6));
    assert_eq!(counter.record(1), 2);
    clock.advance(Duration::from_secs(10));
    assert_eq!(counter.count(&1), 0);
}

#[test]
fn cloned_counters_share_the_same_counts() {
    let counter = SlidingWindowCounter::new(Duration::from_secs(10), 10);
    let cloned = counter.clone();

    assert_eq!(counter.record(1), 1);
    assert_eq!(cloned.record(1), 2);
    assert_eq!(counter.count(&1), 2);
}

#[test]
fn records_from_many_threads_are_counted() {
    const THREADS: usize = 8;
    const EVENTS_PER_THREAD: usize = 500;

    let counter = SlidingWindowCounter::<u64>::new(Duration::from_secs(60), 10);
    let mut handles = Vec::with_capacity(THREADS);

    for _ in 0..THREADS {
        let counter = counter.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..EVENTS_PER_THREAD {
                counter.record(1);
            }
        }));
    }

    for handle in handles {
        handle.join().expect("worker thread should finish");
    }

    assert_eq!(counter.count(&1), THREADS * EVENTS_PER_THREAD);
}
