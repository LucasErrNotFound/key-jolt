use super::MouseEditorView;
use gpui_kit::*;
use std::time::Duration;

impl MouseEditorView {
    pub(super) fn turn_wheel(
        &mut self,
        event: &ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        self.wheel_task = None;
        if cx.reduce_motion() {
            if self.wheel_offset != 0. {
                self.wheel_offset = 0.;
                cx.notify();
            }
            return;
        }
        let delta = f32::from(event.delta.pixel_delta(window.line_height()).y);
        if !delta.is_finite() || delta == 0. {
            return;
        }
        let step = -delta.signum() * 0.05;
        self.wheel_offset = (self.wheel_offset + step).rem_euclid(0.3);
        cx.notify();
        self.wheel_task = Some(cx.spawn(async move |this, cx| {
            for _ in 0..5 {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                let keep_going = this.update(cx, |view, cx| {
                    if cx.reduce_motion() {
                        view.wheel_offset = 0.;
                        view.wheel_task = None;
                        cx.notify();
                        return false;
                    }
                    view.wheel_offset = (view.wheel_offset + step).rem_euclid(0.3);
                    cx.notify();
                    true
                });
                if !matches!(keep_going, Ok(true)) {
                    return;
                }
            }
            _ = this.update(cx, |view, _| view.wheel_task = None);
        }));
    }
}
