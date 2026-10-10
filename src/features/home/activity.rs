use super::HomeView;
use crate::activity::{ACTIVITY, ActivityChannel, ActivitySnapshot};
use gpui_kit::*;
use std::time::{Duration, Instant};

impl HomeView {
    pub(crate) fn set_mixer_visible(
        &mut self,
        visible: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.mixer_visible = visible;
        self.refresh_activity_task(window, cx);
    }

    pub(super) fn refresh_activity_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.activity_task.take();
        ACTIVITY.set_observed(false);
        self.keyboard_activity.reset(cx);
        self.mouse_activity.reset(cx);
        if !self.mixer_visible || !window.is_visible() {
            return;
        }
        ACTIVITY.set_observed(true);
        self.activity_task = Some(cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(33))
                    .await;
                let keep_running = this.update_in(cx, |home, window, cx| {
                    if !home.mixer_visible || !window.is_visible() {
                        return false;
                    }
                    let now = Instant::now();
                    home.keyboard_activity.tick(
                        ACTIVITY.take(ActivityChannel::Keyboard),
                        home.is_active && !home.keyboard_muted,
                        home.is_active && !home.keyboard_muted && home.keyboard_volume > 0.0,
                        now,
                        cx,
                    );
                    home.mouse_activity.tick(
                        ACTIVITY.take(ActivityChannel::Mouse),
                        home.is_active && !home.mouse_muted,
                        home.is_active && !home.mouse_muted && home.mouse_volume > 0.0,
                        now,
                        cx,
                    );
                    true
                });
                if !matches!(keep_running, Ok(true)) {
                    break;
                }
            }
        }));
    }

    pub(super) fn silence_inactive_meters(&mut self, cx: &mut Context<Self>) {
        let now = Instant::now();
        if (self.is_active && !self.keyboard_muted)
            != (self.settings.app_enabled && !self.settings.keyboard_muted)
        {
            ACTIVITY.take(ActivityChannel::Keyboard);
            self.keyboard_activity.reset(cx);
        }
        if (self.is_active && !self.mouse_muted)
            != (self.settings.app_enabled && !self.settings.mouse_muted)
        {
            ACTIVITY.take(ActivityChannel::Mouse);
            self.mouse_activity.reset(cx);
        }
        if !self.is_active || self.keyboard_muted || self.keyboard_volume <= 0.0 {
            self.keyboard_activity.tick(
                ActivitySnapshot::default(),
                self.is_active && !self.keyboard_muted,
                false,
                now,
                cx,
            );
        }
        if !self.is_active || self.mouse_muted || self.mouse_volume <= 0.0 {
            self.mouse_activity.tick(
                ActivitySnapshot::default(),
                self.is_active && !self.mouse_muted,
                false,
                now,
                cx,
            );
        }
    }
}

impl Drop for HomeView {
    fn drop(&mut self) {
        ACTIVITY.set_observed(false);
    }
}
