sliding-window-counter
===========

[![CI](https://github.com/magiclen/sliding-window-counter/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/sliding-window-counter/actions/workflows/ci.yml)

A keyed sliding-window counter for tracking recent events per key.

This crate provides a small, thread-safe counter for counting how many recent events happened for each key within a fixed time window.

It is useful for cases like login attempt tracking, request throttling hints, abuse signals, and other short-lived counters where old events should expire automatically.

Each key keeps its own window, and each window can also have a maximum number of stored events to keep memory usage bounded.

The counter uses an in-memory cache, so it is fast and easy to share inside one process.

It is best suited for local, best-effort decisions. Cache eviction, process restarts, or multiple application instances can make the count lower than the true global count, so strict security or billing rules should use a stronger shared storage layer.

## Example

```rust
use std::time::Duration;

use sliding_window_counter::SlidingWindowCounter;

let counter = SlidingWindowCounter::new(Duration::from_secs(60), 10000, 5);
let user_id = "user-42";

assert_eq!(Some(1), counter.record(user_id));
assert_eq!(Some(2), counter.record(user_id));
assert_eq!(2, counter.count(&user_id));

assert_eq!(Some(3), counter.record(user_id));
assert_eq!(Some(4), counter.record(user_id));
assert_eq!(Some(5), counter.record(user_id));

assert_eq!(None, counter.record(user_id));
assert_eq!(5, counter.count(&user_id));
```

## Crates.io

https://crates.io/crates/sliding-window-counter

## Documentation

https://docs.rs/sliding-window-counter

## License

[MIT](LICENSE)