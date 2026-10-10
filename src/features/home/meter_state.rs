use crate::activity::ActivitySnapshot;
use std::time::{Duration, Instant};

pub(super) const SEGMENTS: usize = 16;

const HOLD: Duration = Duration::from_millis(65);
const DECAY: Duration = Duration::from_millis(200);

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) struct MeterFrame {
    pub(super) segments: usize,
    pub(super) dot: f32,
}

#[derive(Default)]
struct Pulse {
    value: f32,
    started: Option<Instant>,
}

impl Pulse {
    fn trigger(&mut self, value: f32, now: Instant) {
        self.value = value;
        self.started = Some(now);
    }

    fn value(&self, now: Instant, reduced_motion: bool) -> f32 {
        let Some(started) = self.started else {
            return 0.0;
        };
        let elapsed = now.saturating_duration_since(started);
        if elapsed >= HOLD + DECAY {
            return 0.0;
        }
        if reduced_motion || elapsed <= HOLD {
            return self.value;
        }
        self.value * (1.0 - (elapsed - HOLD).as_secs_f32() / DECAY.as_secs_f32())
    }
}

#[derive(Default)]
pub(super) struct MeterState {
    dot: Pulse,
    level: Pulse,
}

impl MeterState {
    pub(super) fn tick(
        &mut self,
        snapshot: ActivitySnapshot,
        enabled: bool,
        audible: bool,
        now: Instant,
        reduced_motion: bool,
    ) -> MeterFrame {
        if !enabled {
            self.reset();
            return MeterFrame::default();
        }
        if snapshot.input {
            self.dot.trigger(1.0, now);
        }
        if !audible {
            self.level = Pulse::default();
        } else if snapshot.gain.is_finite() && snapshot.gain > 0.0 {
            self.level.trigger(snapshot.gain.clamp(0.0, 2.0), now);
        }
        let level = self.level.value(now, reduced_motion);
        MeterFrame {
            segments: ((level / 2.0 * SEGMENTS as f32).ceil() as usize).min(SEGMENTS),
            dot: self.dot.value(now, reduced_motion),
        }
    }

    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
#[path = "meter_tests.rs"]
mod tests;
