use super::{AppPage, AppShell, AppShellEvent};

use crate::features::keyboard::KeyboardEditorView;
use crate::features::mouse::MouseEditorView;
use crate::platform::file_dialog::begin_file_dialog;
use crate::presets::{KeyboardPresetData, MousePresetData, PresetKind};

use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

impl AppShell {
    pub(super) fn start_import_preset(
        &mut self,
        kind: PresetKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((dialog_guard, dialog)) = begin_file_dialog(window) else {
            return;
        };
        let root = self.data_dir.clone();
        let window_handle = window.window_handle();
        let title = match kind {
            PresetKind::Keyboard => "Import keyboard preset",
            PresetKind::Mouse => "Import mouse preset",
        };

        cx.spawn(async move |this, cx| {
            let file = dialog
                .set_title(title)
                .add_filter("KeyJolt preset", &["zip"])
                .pick_file()
                .await;
            drop(dialog_guard);
            let Some(file) = file else {
                return;
            };
            let source = file.path().to_path_buf();
            let result = cx
                .background_spawn(
                    async move { crate::presets::import_preset(&root, kind, &source) },
                )
                .await;

            _ = window_handle.update(cx, |_, window, cx| {
                _ = this.update(cx, |shell, cx| match result {
                    Ok(preset) => {
                        let id: SharedString = preset.id.clone().into();
                        let name: SharedString = preset.name.clone().into();
                        let summary: SharedString = preset.summary.clone().into();
                        match kind {
                            PresetKind::Keyboard => {
                                if let Some(data) = preset.keyboard {
                                    shell.home.update(cx, |home, cx| {
                                        home.apply_saved_keyboard(
                                            id.clone(),
                                            name.clone(),
                                            summary.clone(),
                                            data,
                                            window,
                                            cx,
                                        );
                                    });
                                }
                            }
                            PresetKind::Mouse => {
                                if let Some(data) = preset.mouse {
                                    shell.home.update(cx, |home, cx| {
                                        home.apply_saved_mouse(
                                            id.clone(),
                                            name.clone(),
                                            summary.clone(),
                                            data,
                                            window,
                                            cx,
                                        );
                                    });
                                }
                            }
                        }
                        shell.playback.reload(kind, id.to_string());
                        cx.emit(AppShellEvent::PresetReloadRequested {
                            kind,
                            id: id.clone(),
                        });
                        window.push_notification(
                            Notification::success(format!("\"{name}\" was imported."))
                                .title("Preset imported")
                                .placement(Anchor::BottomRight)
                                .autohide(true)
                                .on_click(cx.listener(|_, _, _, cx| {
                                    cx.notify();
                                    cx.hide();
                                })),
                            cx,
                        );
                        cx.notify();
                    }
                    Err(error) => {
                        window.push_notification(
                            Notification::error(format!("Could not import preset: {error}"))
                                .title("Preset import failed")
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
            });
        })
        .detach();
    }

    pub(super) fn start_delete_preset(
        &mut self,
        kind: PresetKind,
        id: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let root = self.data_dir.clone();
        let storage_id = id.clone();
        self._delete_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(
                    async move { crate::presets::delete_preset(&root, kind, &storage_id) },
                )
                .await;
            _ = this.update_in(cx, move |shell, window, cx| match result {
                Ok(()) => {
                    shell.home.update(cx, |home, cx| {
                        home.remove_preset(kind, &id, window, cx);
                    });
                    window.push_notification(
                        Notification::success("The preset was deleted.")
                            .title("Preset deleted")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                }
                Err(error) => {
                    window.push_notification(
                        Notification::error(format!("Could not delete preset: {error}"))
                            .title("Preset deletion failed")
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
        }));
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn start_keyboard_save(
        &mut self,
        editor: Entity<KeyboardEditorView>,
        requested_id: Option<SharedString>,
        name: SharedString,
        summary: SharedString,
        data: KeyboardPresetData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let existing_user_preset = requested_id
            .as_deref()
            .is_some_and(|id| self.home.read(cx).is_user_keyboard_preset(id));
        let id: SharedString =
            crate::presets::resolve_preset_id(requested_id.as_deref(), existing_user_preset).into();
        editor.update(cx, |editor, cx| {
            editor.set_save_status(crate::features::keyboard::SaveStatus::Saving, cx);
        });
        self.stop_preview(None, cx);
        let root = self.data_dir.clone();
        let worker_id = id.to_string();
        let worker_name = name.to_string();
        let worker_summary = summary.to_string();
        let worker_data = data.clone();
        self._save_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    crate::presets::save_keyboard_preset(
                        &root,
                        &worker_id,
                        &worker_name,
                        &worker_summary,
                        &worker_data,
                    )?;
                    crate::presets::load_preset(&root, PresetKind::Keyboard, &worker_id)
                })
                .await;
            _ = this.update_in(cx, move |shell, window, cx| match result {
                Ok(saved) => {
                    let saved_data = saved.keyboard.unwrap_or_else(|| data.clone());
                    shell.playback.reload(PresetKind::Keyboard, id.to_string());
                    shell.home.update(cx, |home, cx| {
                        home.apply_saved_keyboard(
                            id.clone(),
                            name.clone(),
                            summary.clone(),
                            saved_data,
                            window,
                            cx,
                        );
                    });
                    cx.emit(AppShellEvent::PresetReloadRequested {
                        kind: PresetKind::Keyboard,
                        id: id.clone(),
                    });
                    shell.editor_subscription.take();
                    shell.page = AppPage::Home;
                    window.push_notification(
                        Notification::success(format!("\"{name}\" was saved."))
                            .title("Keyboard preset saved")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                    cx.notify();
                }
                Err(error) => {
                    editor.update(cx, |editor, cx| {
                        editor.set_save_status(
                            crate::features::keyboard::SaveStatus::Failed(error.clone().into()),
                            cx,
                        );
                    });
                    window.push_notification(
                        Notification::error(format!("Could not save preset: {error}"))
                            .title("Keyboard preset save failed")
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
        }));
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn start_mouse_save(
        &mut self,
        editor: Entity<MouseEditorView>,
        requested_id: Option<SharedString>,
        name: SharedString,
        summary: SharedString,
        data: MousePresetData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let existing_user_preset = requested_id
            .as_deref()
            .is_some_and(|id| self.home.read(cx).is_user_mouse_preset(id));
        let id: SharedString =
            crate::presets::resolve_preset_id(requested_id.as_deref(), existing_user_preset).into();
        editor.update(cx, |editor, cx| {
            editor.set_save_status(crate::features::mouse::SaveStatus::Saving, cx);
        });
        self.stop_preview(None, cx);
        let root = self.data_dir.clone();
        let worker_id = id.to_string();
        let worker_name = name.to_string();
        let worker_summary = summary.to_string();
        let worker_data = data.clone();
        self._save_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    crate::presets::save_mouse_preset(
                        &root,
                        &worker_id,
                        &worker_name,
                        &worker_summary,
                        &worker_data,
                    )?;
                    crate::presets::load_preset(&root, PresetKind::Mouse, &worker_id)
                })
                .await;
            _ = this.update_in(cx, move |shell, window, cx| match result {
                Ok(saved) => {
                    let saved_data = saved.mouse.unwrap_or_else(|| data.clone());
                    shell.playback.reload(PresetKind::Mouse, id.to_string());
                    shell.home.update(cx, |home, cx| {
                        home.apply_saved_mouse(
                            id.clone(),
                            name.clone(),
                            summary.clone(),
                            saved_data,
                            window,
                            cx,
                        );
                    });
                    cx.emit(AppShellEvent::PresetReloadRequested {
                        kind: PresetKind::Mouse,
                        id: id.clone(),
                    });
                    shell.editor_subscription.take();
                    shell.page = AppPage::Home;
                    window.push_notification(
                        Notification::success(format!("\"{name}\" was saved."))
                            .title("Mouse preset saved")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                    cx.notify();
                }
                Err(error) => {
                    editor.update(cx, |editor, cx| {
                        editor.set_save_status(
                            crate::features::mouse::SaveStatus::Failed(error.clone().into()),
                            cx,
                        );
                    });
                    window.push_notification(
                        Notification::error(format!("Could not save preset: {error}"))
                            .title("Mouse preset save failed")
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
        }));
    }
}
