use super::{MouseEditorEvent, MouseEditorView};

use super::model::MouseButton;
use crate::features::preset_editor::{SaveStatus, UploadState};

use gpui_kit::component::WindowExt;

use gpui_kit::component::notification::Notification;
use gpui_kit::*;

impl MouseEditorView {
    pub(super) fn request_save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.save_status, SaveStatus::Saving) || !self.has_unsaved_changes(cx) {
            return;
        }
        let preset_name = self.name_input.read(cx).value().trim().to_string();
        if let Some(message) = self.save_validation_error(&preset_name) {
            window.push_notification(
                Notification::warning(message)
                    .title("Cannot save preset")
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
        let mapped_button_count = [MouseButton::Left, MouseButton::Right, MouseButton::Middle]
            .iter()
            .filter(|button| {
                self.files.iter().any(|file| {
                    file.assigned_buttons[button.index()]
                        && matches!(file.state, UploadState::Success | UploadState::Complete)
                })
            })
            .count();
        let mapped_button_label = if mapped_button_count == 1 {
            "button"
        } else {
            "buttons"
        };
        cx.emit(MouseEditorEvent::SaveRequested {
            preset_id: self.preset_id.clone(),
            preset_name: preset_name.into(),
            summary: format!("Custom · {mapped_button_count} {mapped_button_label} mapped").into(),
            data: self.current_preset_data(),
        });
    }
}
