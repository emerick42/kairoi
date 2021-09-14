//! Framerate limitation for the database.
//!
//! This module provide a [`Clock`] structure, that can be used to trigger callbacks, following
//! a configured framerate. The clock can be run using the [`Clock::start`] method. While this
//! clock is limiting the number of tick per second occurring (it is guaranted that it won't exceed
//! the configured framerate), it also tries to maximize it. Executed callbacks are timed, and
//! elapsed time are subtracted from theoretical frame durations.
//!
//! # Examples
//!
//! ```
//! let clock = Clock::ticking_at(60);
//! clock.start(|| {
//!     println!("One Hello, 60 times per second.");
//! });
//! ```

use std::time::{Duration, Instant};
use std::thread;

pub struct Clock {
    framerate: u8,
}

impl Clock {
    /// Create a new clock, ticking at the given framerate (in tick per second).
    pub fn with_framerate(framerate: u8) -> Self {
        Clock {
            framerate: framerate,
        }
    }

    /// Start this clock, ticking at a rate following the configured framerate, and calling the
    /// given callback at each tick. This method is never ending.
    pub fn start(self, mut callback: impl FnMut () -> ()) -> () {
        loop {
            let previous_time = Instant::now();

            callback();

            // Put the thread asleep to run at a maximum of 128 time per second.
            let now = Instant::now();
            let elapsed_time = now.duration_since(previous_time);

            match Duration::new(0, 1_000_000_000u32 / self.framerate as u32).checked_sub(elapsed_time) {
                Some(sleep_time) => {
                    thread::sleep(sleep_time);
                },
                None => {},
            };
        };
    }
}
