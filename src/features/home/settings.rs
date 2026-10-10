use super::{HomeEvent, HomeView};

use crate::settings::store::AppSettings;
use gpui_kit::component::slider::SliderState;
use gpui_kit::*;
use std::path::PathBuf;

impl HomeView {
    pub(crate) fn data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    pub(crate) fn startup_warnings(&self) -> &[String] {
        &self.startup_warnings
    }

    pub(crate) fn settings(&self) -> AppSettings {
        self.settings.clone()
    }

    pub(crate) fn set_app_enabled_from_shortcut(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.is_active = enabled;
        self.emit_settings(true, cx);
    }

    pub(super) fn toggle_mute(
        muted_field: &mut bool,
        _slider: &Entity<SliderState>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        *muted_field = !*muted_field;
        cx.notify();
    }

    pub(super) fn emit_settings(&mut self, persist: bool, cx: &mut Context<Self>) {
        self.silence_inactive_meters(cx);
        self.settings.keyboard_volume = self.keyboard_volume;
        self.settings.mouse_volume = self.mouse_volume;
        self.settings.keyboard_muted = self.keyboard_muted;
        self.settings.mouse_muted = self.mouse_muted;
        self.settings.app_enabled = self.is_active;
        self.settings.active_keyboard_preset_id =
            (!self.active_id.is_empty()).then(|| self.active_id.to_string());
        self.settings.active_mouse_preset_id =
            (!self.mouse_active_id.is_empty()).then(|| self.mouse_active_id.to_string());
        cx.emit(HomeEvent::SettingsChanged {
            settings: self.settings.clone(),
            persist,
        });
        cx.notify();
    }
}
