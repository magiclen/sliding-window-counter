//! A keyed sliding-window counter for tracking recent events.

mod clock;
mod counter;
mod window;

pub use crate::{
    clock::{Clock, SystemClock},
    counter::SlidingWindowCounter,
};
