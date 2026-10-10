use super::{AppPage, AppShell, PreviewTarget};

use crate::features::keyboard::{KeyboardEditorEvent, KeyboardEditorView};

use crate::features::mouse::{MouseEditorEvent, MouseEditorView};

use crate::presets::{KeyboardPresetData, MousePresetData};

use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

impl AppShell {
    pub(super) fn open_keyboard_editor(
        &mut self,
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        preset_data: Option<KeyboardPresetData>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.home
            .update(cx, |home, cx| home.set_mixer_visible(false, window, cx));
        self.stop_preview(None, cx);
        let editor = KeyboardEditorView::view(preset_id, preset_name, preset_data, window, cx);

        let subscription = cx.subscribe_in(
            &editor,
            window,
            |this, editor_entity, event: &KeyboardEditorEvent, window, cx| match event {
                KeyboardEditorEvent::BackRequested => {
                    this.stop_preview(None, cx);
                    this.editor_subscription.take();
                    this.page = AppPage::Home;
                    this.home
                        .update(cx, |home, cx| home.set_mixer_visible(true, window, cx));
                    cx.notify();
                }

                KeyboardEditorEvent::PreviewRequested { file_id, path } => {
                    this.start_preview(PreviewTarget::Keyboard(*file_id), path.clone(), window, cx);
                }

                KeyboardEditorEvent::StopPreviewRequested { file_id } => {
                    this.stop_preview(file_id.map(PreviewTarget::Keyboard), cx);
                }

                KeyboardEditorEvent::PlaybackStateChanged {
                    state: crate::features::keyboard::PreviewState::Error(message),
                    ..
                } => {
                    window.push_notification(
                        Notification::error(message.clone())
                            .title("Audio preview failed")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                }

                KeyboardEditorEvent::PlaybackStateChanged { .. } => {}

                KeyboardEditorEvent::SaveRequested {
                    preset_id,
                    preset_name,
                    summary,
                    data,
                } => {
                    this.start_keyboard_save(
                        editor_entity.clone(),
                        preset_id.clone(),
                        preset_name.clone(),
                        summary.clone(),
                        data.clone(),
                        window,
                        cx,
                    );
                }
            },
        );

        self.editor_subscription = Some(subscription);
        self.page = AppPage::KeyboardEditor(editor);

        cx.notify();
    }

    pub(super) fn open_mouse_editor(
        &mut self,
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        preset_data: Option<MousePresetData>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.home
            .update(cx, |home, cx| home.set_mixer_visible(false, window, cx));
        self.stop_preview(None, cx);
        let editor = MouseEditorView::view(preset_id, preset_name, preset_data, window, cx);
        let subscription = cx.subscribe_in(
            &editor,
            window,
            |this, editor_entity, event: &MouseEditorEvent, window, cx| match event {
                MouseEditorEvent::BackRequested => {
                    this.stop_preview(None, cx);
                    this.editor_subscription.take();
                    this.page = AppPage::Home;
                    this.home
                        .update(cx, |home, cx| home.set_mixer_visible(true, window, cx));
                    cx.notify();
                }

                MouseEditorEvent::PreviewRequested { file_id, path } => {
                    this.start_preview(PreviewTarget::Mouse(*file_id), path.clone(), window, cx);
                }

                MouseEditorEvent::StopPreviewRequested { file_id } => {
                    this.stop_preview(file_id.map(PreviewTarget::Mouse), cx);
                }

                MouseEditorEvent::PlaybackStateChanged {
                    state: crate::features::mouse::PreviewState::Error(message),
                    ..
                } => {
                    window.push_notification(
                        Notification::error(message.clone())
                            .title("Audio preview failed")
                            .placement(Anchor::BottomRight)
                            .autohide(true)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.notify();
                                cx.hide();
                            })),
                        cx,
                    );
                }

                MouseEditorEvent::PlaybackStateChanged { .. } => {}

                MouseEditorEvent::SaveRequested {
                    preset_id,
                    preset_name,
                    summary,
                    data,
                } => {
                    this.start_mouse_save(
                        editor_entity.clone(),
                        preset_id.clone(),
                        preset_name.clone(),
                        summary.clone(),
                        data.clone(),
                        window,
                        cx,
                    );
                }
            },
        );

        self.editor_subscription = Some(subscription);
        self.page = AppPage::MouseEditor(editor);
        cx.notify();
    }
}
