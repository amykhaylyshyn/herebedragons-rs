#[cfg(test)]
use mockall::automock;

use std::time::Instant;

#[cfg_attr(test, automock)]
pub trait Clock {
    fn now() -> Instant;
}

pub struct InstantClock;

impl Clock for InstantClock {
    fn now() -> Instant {
        Instant::now()
    }
}
