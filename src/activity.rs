use std::sync::atomic::{AtomicU64, Ordering};

const OBSERVED: u64 = 1 << 63;
const INPUT: u64 = 1 << 32;
const GAIN: u64 = u32::MAX as u64;

#[derive(Clone, Copy)]
pub(crate) enum ActivityChannel {
    Keyboard,
    Mouse,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub(crate) struct ActivitySnapshot {
    pub(crate) input: bool,
    pub(crate) gain: f32,
}

pub(crate) struct ActivityMonitor {
    keyboard: AtomicU64,
    mouse: AtomicU64,
}

pub(crate) static ACTIVITY: ActivityMonitor = ActivityMonitor::new();

impl ActivityMonitor {
    const fn new() -> Self {
        Self {
            keyboard: AtomicU64::new(0),
            mouse: AtomicU64::new(0),
        }
    }

    fn slot(&self, channel: ActivityChannel) -> &AtomicU64 {
        match channel {
            ActivityChannel::Keyboard => &self.keyboard,
            ActivityChannel::Mouse => &self.mouse,
        }
    }

    pub(crate) fn set_observed(&self, observed: bool) {
        let value = if observed { OBSERVED } else { 0 };
        self.keyboard.store(value, Ordering::Relaxed);
        self.mouse.store(value, Ordering::Relaxed);
    }

    pub(crate) fn record_input(&self, channel: ActivityChannel) {
        let _ = self
            .slot(channel)
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                (value & OBSERVED != 0).then_some(value | INPUT)
            });
    }

    pub(crate) fn record_sound(&self, channel: ActivityChannel, gain: f32) {
        if !gain.is_finite() || gain <= 0.0 {
            return;
        }
        let gain = gain.clamp(0.0, 2.0).to_bits() as u64;
        let _ = self
            .slot(channel)
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                (value & OBSERVED != 0).then_some((value & !GAIN) | (value & GAIN).max(gain))
            });
    }

    pub(crate) fn take(&self, channel: ActivityChannel) -> ActivitySnapshot {
        let value =
            self.slot(channel)
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                    (value & OBSERVED != 0).then_some(OBSERVED)
                });
        match value {
            Ok(value) => ActivitySnapshot {
                input: value & INPUT != 0,
                gain: f32::from_bits((value & GAIN) as u32),
            },
            Err(_) => ActivitySnapshot::default(),
        }
    }
}

#[cfg(test)]
#[path = "activity_tests.rs"]
mod tests;
