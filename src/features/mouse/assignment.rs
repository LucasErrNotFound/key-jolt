use super::MouseEditorView;
use super::model::MouseButton;
use gpui_kit::base::CheckboxState;
use gpui_kit::*;

impl MouseEditorView {
    pub(super) fn select_button(&mut self, button: MouseButton, cx: &mut Context<Self>) {
        if self.selected_button != button {
            let previous_count = self.assigned_sound_count();
            self.selected_button = button;
            self.update_playback_mode_for_selection_change(previous_count);
            cx.notify();
        }
    }

    pub(super) fn toggle_file_assignment(
        &mut self,
        id: u64,
        button: MouseButton,
        state: CheckboxState,
        cx: &mut Context<Self>,
    ) {
        let previous_count = self.assigned_sound_count();
        if let Some(file) = self.files.iter_mut().find(|file| file.id == id) {
            let assigned = state == CheckboxState::Checked;
            file.assigned_buttons[button.index()] = assigned;
        }
        self.update_playback_mode_for_selection_change(previous_count);
        cx.notify();
    }
}
