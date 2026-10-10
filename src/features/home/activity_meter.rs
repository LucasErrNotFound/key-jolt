use super::meter_state::{MeterFrame, MeterState, SEGMENTS};
use crate::activity::ActivitySnapshot;
use gpui_kit::component::*;
use gpui_kit::*;
use std::time::Instant;

pub(super) struct ActivityWidgets {
    state: MeterState,
    pub(super) dot: Entity<ActivityIndicator>,
    pub(super) bars: Entity<ActivityIndicator>,
}

impl ActivityWidgets {
    pub(super) fn new(label: &'static str, cx: &mut App) -> Self {
        Self {
            state: MeterState::default(),
            dot: cx.new(|_| ActivityIndicator::new(label, true)),
            bars: cx.new(|_| ActivityIndicator::new(label, false)),
        }
    }

    pub(super) fn tick(
        &mut self,
        snapshot: ActivitySnapshot,
        enabled: bool,
        audible: bool,
        now: Instant,
        cx: &mut App,
    ) {
        let frame = self
            .state
            .tick(snapshot, enabled, audible, now, cx.reduce_motion());
        self.apply(frame, cx);
    }

    pub(super) fn reset(&mut self, cx: &mut App) {
        self.state.reset();
        self.apply(MeterFrame::default(), cx);
    }

    fn apply(&self, frame: MeterFrame, cx: &mut App) {
        for indicator in [&self.dot, &self.bars] {
            indicator.update(cx, |indicator, cx| {
                let changed = if indicator.is_dot {
                    indicator.frame.dot != frame.dot
                } else {
                    indicator.frame.segments != frame.segments
                };
                if changed {
                    indicator.frame = frame;
                    cx.notify();
                }
            });
        }
    }
}

pub(super) struct ActivityIndicator {
    label: &'static str,
    is_dot: bool,
    frame: MeterFrame,
}

impl ActivityIndicator {
    fn new(label: &'static str, is_dot: bool) -> Self {
        Self {
            label,
            is_dot,
            frame: MeterFrame::default(),
        }
    }
}

impl Render for ActivityIndicator {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.is_dot {
            return div()
                .id("input-activity-dot")
                .aria_label(format!("{} input activity", self.label))
                .size(rems(0.5))
                .rounded_full()
                .bg(if self.frame.dot > 0.0 {
                    cx.theme().green.opacity(0.3 + self.frame.dot * 0.7)
                } else {
                    cx.theme().muted_foreground.opacity(0.25)
                })
                .into_any_element();
        }
        v_flex()
            .id("sound-activity-meter")
            .aria_label(format!("{} volume-aware sound activity", self.label))
            .w(rems(0.65))
            .h_full()
            .gap(rems(0.15))
            .children((0..SEGMENTS).rev().map(|index| {
                let color = if index >= 14 {
                    cx.theme().red
                } else if index >= 10 {
                    cx.theme().yellow
                } else {
                    cx.theme().green
                };
                div()
                    .flex_1()
                    .w_full()
                    .rounded(rems(0.1))
                    .bg(if index < self.frame.segments {
                        color
                    } else {
                        cx.theme().muted_foreground.opacity(0.2)
                    })
            }))
            .into_any_element()
    }
}
