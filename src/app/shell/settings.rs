use super::AppShell;
use crate::settings::appearance::AppearanceSelection;
use crate::settings::catalog::apply_theme;
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use std::sync::atomic::Ordering;
use std::time::Duration;

impl AppShell {
    pub(super) fn apply_appearance(
        &mut self,
        selection: AppearanceSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Err(message) = apply_theme(&selection, cx) {
            self.title_bar.update(cx, |title_bar, cx| {
                title_bar.set_error(message.clone(), cx);
            });
            window.push_notification(
                Notification::error(message)
                    .title("Theme change failed")
                    .placement(Anchor::BottomRight)
                    .autohide(true)
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.notify();
                        cx.hide();
                    })),
                cx,
            );
            return;
        }
        self.settings.appearance_mode = selection.mode;
        self.settings.appearance_theme_id = selection.theme_id.clone();
        self.title_bar.update(cx, |title_bar, cx| {
            title_bar.set_selection(selection, cx);
        });
        self.persist_settings(window, cx);
    }

    pub(super) fn set_run_at_startup(
        &mut self,
        enabled: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !crate::platform::startup::is_supported()
            || self.startup_change_pending
            || self.settings.run_at_startup == enabled
        {
            return;
        }
        self.settings.run_at_startup = enabled;
        self.startup_change_pending = true;
        self.title_bar.update(cx, |title_bar, cx| {
            title_bar.set_startup_state(enabled, true, cx);
        });
        self.persist_settings(window, cx);
    }

    pub(super) fn persist_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let root = self.data_dir.clone();
        let settings = self.settings.clone();
        let generation = self.settings_generation.fetch_add(1, Ordering::AcqRel) + 1;
        let current_generation = self.settings_generation.clone();
        let settings_writer = self.settings_writer.clone();
        let delay = if self.startup_change_pending {
            Duration::ZERO
        } else {
            Duration::from_millis(250)
        };
        self._settings_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    if !delay.is_zero() {
                        std::thread::sleep(delay);
                    }
                    let mut writer = settings_writer
                        .lock()
                        .unwrap_or_else(|error| error.into_inner());
                    if current_generation.load(Ordering::Acquire) != generation {
                        return None;
                    }
                    let result = writer.save(&root, &settings);
                    Some((result, writer.run_at_startup))
                })
                .await;
            if let Some((result, persisted_startup)) = result {
                _ = this.update_in(cx, |shell, window, cx| {
                    if shell.settings_generation.load(Ordering::Acquire) != generation {
                        return;
                    }
                    shell.startup_change_pending = false;
                    if result.is_err() {
                        shell.settings.run_at_startup = persisted_startup;
                    }
                    shell.title_bar.update(cx, |title_bar, cx| {
                        title_bar.set_startup_state(shell.settings.run_at_startup, false, cx);
                    });
                    if let Err(error) = result {
                        window.push_notification(
                            Notification::error(format!("Could not save settings: {error}"))
                                .title("Settings save failed")
                                .placement(Anchor::BottomRight)
                                .autohide(true)
                                .on_click(cx.listener(|_, _, _, cx| {
                                    cx.notify();
                                    cx.hide();
                                })),
                            cx,
                        );
                    }
                });
            }
        }));
    }
}
