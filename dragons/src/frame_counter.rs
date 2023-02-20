use std::{
    marker::PhantomData,
    time::{Duration, Instant},
};

use crate::clock::Clock;

#[derive(Debug)]
pub struct FrameCounter<TClock: Clock> {
    last_frame_time: Option<Instant>,
    last_frame_duration: Option<Duration>,
    avg_frame_duration: Option<Duration>,
    frame_duration: Vec<Duration>,
    cursor: usize,
    avg_count: usize,
    _data: PhantomData<TClock>,
}

impl<TClock: Clock> FrameCounter<TClock> {
    pub fn frame_done(&mut self) {
        let now = TClock::now();
        if let Some(prev_frame_time) = self.last_frame_time {
            let frame_duration = now - prev_frame_time;
            if self.frame_duration.len() < self.avg_count {
                self.frame_duration.push(frame_duration);
            } else {
                self.frame_duration[self.cursor] = frame_duration;
            }
            self.cursor = (self.cursor + 1) % self.avg_count;
            self.last_frame_duration = Some(frame_duration)
        }

        self.last_frame_time = Some(now);

        if !self.frame_duration.is_empty() {
            let total_frame_duration_seconds = self
                .frame_duration
                .iter()
                .fold(0f64, |acc, val| acc + val.as_secs_f64());
            self.avg_frame_duration = Some(Duration::from_secs_f64(
                total_frame_duration_seconds / self.frame_duration.len() as f64,
            ));
        }
    }

    pub fn last_frame_duration(&self) -> Option<Duration> {
        self.last_frame_duration
    }

    pub fn last_frame_duration_f32(&self) -> Option<f32> {
        self.last_frame_duration.map(|x| x.as_secs_f32())
    }

    pub fn avg_frame_duration(&self) -> Option<Duration> {
        self.avg_frame_duration
    }
}

impl<TClock: Clock> Default for FrameCounter<TClock> {
    fn default() -> Self {
        Self {
            avg_count: 16,
            frame_duration: Vec::with_capacity(16),
            last_frame_time: Default::default(),
            last_frame_duration: Default::default(),
            avg_frame_duration: Default::default(),
            cursor: Default::default(),
            _data: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::FrameCounter;
    use crate::clock::MockClock;
    use lazy_static::lazy_static;
    use parking_lot::Mutex;

    // Lock is necessary due to mocking static method Clock::now()
    lazy_static! {
        static ref MTX: Mutex<()> = Default::default();
    }

    #[test]
    fn test_last_frame_duration() {
        let _lock = MTX.lock();
        let mut frame_counter = FrameCounter::<MockClock>::default();

        assert!(frame_counter.last_frame_duration().is_none());
        assert!(frame_counter.avg_frame_duration().is_none());

        let now = Instant::now();
        let ctx = MockClock::now_context();
        ctx.expect().return_const(now);

        frame_counter.frame_done();

        ctx.expect().return_const(now + Duration::from_millis(30));

        assert!(frame_counter.last_frame_duration().is_none());
        assert!(frame_counter.avg_frame_duration().is_none());

        frame_counter.frame_done();

        assert_eq!(
            frame_counter.last_frame_duration(),
            Some(Duration::from_millis(30))
        );
        assert_eq!(
            frame_counter.avg_frame_duration(),
            Some(Duration::from_millis(30))
        );
    }
}
