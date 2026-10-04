use super::MouseEditorView;
use crate::features::preset_editor::{PlaybackMode, UploadState};

use crate::presets::{MousePresetData, MouseSoundData};

use gpui_kit::*;

impl MouseEditorView {
    pub(super) fn has_unsaved_changes(&self, cx: &App) -> bool {
        self.current_preset_data() != self.initial_data
            || self.name_input.read(cx).value() != self.preset_name.as_ref()
    }

    pub(super) fn current_preset_data(&self) -> MousePresetData {
        MousePresetData {
            files: self
                .files
                .iter()
                .map(|file| MouseSoundData {
                    path: file.path.clone(),
                    name: file.name.clone(),
                    size: file.size.clone(),
                    assigned_buttons: file.assigned_buttons,
                })
                .collect(),
            random_playback: self.playback_mode == PlaybackMode::Random,
        }
    }

    pub(super) fn save_validation_error(&self, name: &str) -> Option<SharedString> {
        if let Err(message) = crate::presets::validate_preset_name(name) {
            return Some(message.into());
        }

        if !self.files.iter().any(|file| {
            file.assigned_buttons.iter().any(|assigned| *assigned)
                && matches!(file.state, UploadState::Success | UploadState::Complete)
        }) {
            return Some(
                "Assign at least one successfully uploaded sound to a mouse button.".into(),
            );
        }

        None
    }
}
