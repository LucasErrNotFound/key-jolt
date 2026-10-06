use super::super::title_bar::AppTitleBarEvent;
use super::{AppShell, AppShellEvent};

use crate::features::home::HomeEvent;
use crate::presets::PresetKind;
use crate::settings::appearance::{AppearanceSelection, pair_selection, resolve_selection};

use gpui_kit::*;

impl AppShell {
    pub(super) fn on_home_event(
        &mut self,
        event: &HomeEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            HomeEvent::EditRequested(preset_id) => {
                let preset_name = self.home.read(cx).active_preset_name();
                let preset_data = self.home.read(cx).keyboard_preset_data(preset_id);

                self.open_keyboard_editor(
                    Some(preset_id.clone()),
                    preset_name,
                    preset_data,
                    window,
                    cx,
                );
            }

            HomeEvent::ConfigureMouseRequested(preset_id) => {
                let preset_name = self.home.read(cx).active_mouse_preset_name();
                let preset_data = self.home.read(cx).mouse_preset_data(preset_id);
                self.open_mouse_editor(
                    Some(preset_id.clone()),
                    preset_name,
                    preset_data,
                    window,
                    cx,
                );
            }

            HomeEvent::CreateKeyboardRequested => {
                self.open_keyboard_editor(None, "".into(), None, window, cx);
            }

            HomeEvent::CreateMouseRequested => {
                self.open_mouse_editor(None, "".into(), None, window, cx);
            }

            HomeEvent::ImportKeyboardRequested => {
                self.start_import_preset(PresetKind::Keyboard, window, cx);
            }

            HomeEvent::ImportMousesRequested => {
                self.start_import_preset(PresetKind::Mouse, window, cx);
            }

            HomeEvent::DeleteKeyboardRequested(id) => {
                self.start_delete_preset(PresetKind::Keyboard, id.clone(), window, cx);
            }

            HomeEvent::DeleteMouseRequested(id) => {
                self.start_delete_preset(PresetKind::Mouse, id.clone(), window, cx);
            }

            HomeEvent::SettingsChanged { settings, persist } => {
                let keyboard_changed =
                    self.settings.active_keyboard_preset_id != settings.active_keyboard_preset_id;
                let mouse_changed =
                    self.settings.active_mouse_preset_id != settings.active_mouse_preset_id;
                let mut settings = settings.clone();
                settings.run_at_startup = self.settings.run_at_startup;
                settings.appearance_mode = self.settings.appearance_mode;
                settings.appearance_theme_id = self.settings.appearance_theme_id.clone();
                self.settings = settings.clone();
                self.playback.update_settings(&settings);
                if keyboard_changed {
                    self.playback.reload(
                        PresetKind::Keyboard,
                        settings.active_keyboard_preset_id.as_deref().unwrap_or(""),
                    );
                }
                if mouse_changed {
                    self.playback.reload(
                        PresetKind::Mouse,
                        settings.active_mouse_preset_id.as_deref().unwrap_or(""),
                    );
                }
                cx.emit(AppShellEvent::SettingsChanged(settings.clone()));
                if *persist {
                    self.persist_settings(window, cx);
                }
            }
        }
    }

    pub(super) fn on_title_bar_event(
        &mut self,
        event: &AppTitleBarEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            AppTitleBarEvent::RunAtStartup(enabled) => {
                self.set_run_at_startup(*enabled, window, cx);
            }
            AppTitleBarEvent::Mode(mode) => {
                let selection = pair_selection(
                    &AppearanceSelection {
                        mode: self.settings.appearance_mode,
                        theme_id: self.settings.appearance_theme_id.clone(),
                    },
                    *mode,
                    &self.themes,
                )
                .or_else(|| resolve_selection(*mode, None, &self.themes));
                if let Some(selection) = selection {
                    self.apply_appearance(selection, window, cx);
                }
            }
            AppTitleBarEvent::Theme(theme_id) => {
                if let Some(theme) = self.themes.iter().find(|theme| &theme.id == theme_id) {
                    let selection = AppearanceSelection {
                        mode: theme.mode,
                        theme_id: theme.id.clone(),
                    };
                    self.apply_appearance(selection, window, cx);
                } else {
                    self.title_bar.update(cx, |title_bar, cx| {
                        title_bar.set_error(format!("Theme not found: {theme_id}"), cx);
                    });
                }
            }
        }
    }
}
