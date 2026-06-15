use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

#[derive(Debug, Default)]
pub(crate) struct SlidingWindow {
    events: VecDeque<Instant>,
}

impl SlidingWindow {
    pub(crate) fn record(&mut self, now: Instant, window: Duration) -> usize {
        self.remove_expired(now, window);
        self.events.push_back(now);

        self.events.len()
    }

    pub(crate) fn count(&mut self, now: Instant, window: Duration) -> usize {
        self.remove_expired(now, window);

        self.events.len()
    }

    fn remove_expired(&mut self, now: Instant, window: Duration) {
        let Some(cutoff) = now.checked_sub(window) else {
            return;
        };

        // Events are stored from oldest to newest, so the first fresh event lets us stop scanning.
        while let Some(&oldest) = self.events.front() {
            if oldest <= cutoff {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }
}
