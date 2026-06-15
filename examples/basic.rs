use std::{thread, time::Duration};

use sliding_window_counter::SlidingWindowCounter;

fn main() {
    let counter = SlidingWindowCounter::new(Duration::from_secs(3), 100);
    let user_id = "user-1";

    println!("count after first event: {}", counter.record(user_id));
    thread::sleep(Duration::from_secs(1));

    println!("count after second event: {}", counter.record(user_id));
    thread::sleep(Duration::from_secs(1));

    println!("count after third event: {}", counter.record(user_id));
    println!("current count: {}", counter.count(&user_id));

    thread::sleep(Duration::from_millis(1500));
    println!("count after the first event expires: {}", counter.count(&user_id));

    thread::sleep(Duration::from_secs(1));
    println!("count after the second event expires: {}", counter.count(&user_id));

    thread::sleep(Duration::from_secs(1));
    println!("count after all events expire: {}", counter.count(&user_id));
}
