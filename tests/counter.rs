use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use sliding_window_counter::{Clock, SlidingWindowCounter};

#[derive(Clone, Debug)]
struct ManualClock {
    now: Arc<Mutex<Instant>>,
}

impl ManualClock {
    fn new(now: Instant) -> Self {
        Self {
            now: Arc::new(Mutex::new(now))
        }
    }

    fn advance(&self, duration: Duration) {
        let mut now = self.now.lock().expect("manual clock mutex should not be poisoned");
        *now =
            now.checked_add(duration).expect("manual clock should stay in the valid Instant range");
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Instant {
        *self.now.lock().expect("manual clock mutex should not be poisoned")
    }
}

#[test]
fn record_returns_the_current_count_for_one_key() {
    let clock = ManualClock::new(Instant::now());
    let counter = SlidingWindowCounter::with_clock(Duration::from_secs(10), 10, clock);

    assert_eq!(counter.record(1), 1);
    assert_eq!(counter.record(1), 2);
    assert_eq!(counter.count(&1), 2);
}

#[test]
fn keys_are_counted_independently() {
    let clock = ManualClock::new(Instant::now());
    let counter = SlidingWindowCounter::with_clock(Duration::from_secs(10), 10, clock);

    assert_eq!(counter.record(1), 1);
    assert_eq!(counter.record(2), 1);
    assert_eq!(counter.record(1), 2);
    assert_eq!(counter.count(&2), 1);
}

#[test]
fn count_reads_without_recording_a_new_event() {
    let clock = ManualClock::new(Instant::now());
    let counter = SlidingWindowCounter::with_clock(Duration::from_secs(10), 10, clock);

    assert_eq!(counter.count(&1), 0);
    assert_eq!(counter.record(1), 1);
    assert_eq!(counter.count(&1), 1);
    assert_eq!(counter.count(&1), 1);
}

#[test]
fn expired_events_are_removed_before_counting() {
    let clock = ManualClock::new(Instant::now());
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
    let clock = ManualClock::new(Instant::now());
    let counter = SlidingWindowCounter::with_clock(Duration::from_secs(10), 10, clock);
    let cloned = counter.clone();

    assert_eq!(counter.record(1), 1);
    assert_eq!(cloned.record(1), 2);
    assert_eq!(counter.count(&1), 2);
}

#[test]
fn counter_with_system_clock_can_be_formatted_with_debug() {
    let counter = SlidingWindowCounter::<u64>::new(Duration::from_secs(10), 10);

    let output = format!("{counter:?}");

    assert!(output.contains("SlidingWindowCounter"));
    assert!(output.contains("cache"));
    assert!(output.contains("window"));
    assert!(output.contains("clock"));
}

#[test]
fn counter_with_debug_clock_can_be_formatted_with_debug() {
    let clock = ManualClock::new(Instant::now());
    let counter =
        SlidingWindowCounter::<u64, ManualClock>::with_clock(Duration::from_secs(10), 10, clock);

    let output = format!("{counter:?}");

    assert!(output.contains("SlidingWindowCounter"));
    assert!(output.contains("ManualClock"));
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
