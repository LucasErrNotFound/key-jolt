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

    pub(super) fn persist_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let root = self.data_dir.clone();
        let settings = self.settings.clone();
        let generation = self.settings_generation.fetch_add(1, Ordering::AcqRel) + 1;
        let current_generation = self.settings_generation.clone();
        self._settings_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    std::thread::sleep(Duration::from_millis(250));
                    if current_generation.load(Ordering::Acquire) == generation {
                        Some(crate::settings::store::save_to(&root, &settings))
                    } else {
                        None
                    }
                })
                .await;
            if let Some(Err(error)) = result {
                _ = this.update_in(cx, |_, window, cx| {
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
                });
            }
        }));
    }
}
