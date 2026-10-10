use super::KeyboardEditorView;
use crate::features::preset_editor::{PlaybackMode, UploadState};

use crate::presets::{
    KeyboardPresetData, KeyboardSoundData, canonical_key_identifier as canonical_key_id,
};

use gpui_kit::*;
use std::collections::BTreeSet;

impl KeyboardEditorView {
    pub(super) fn has_unsaved_changes(&self, cx: &App) -> bool {
        let current_data = self.preset_data(false);
        let files_unchanged =
            current_data.files.len() == self.initial_data.files.len()
                && current_data.files.iter().zip(&self.initial_data.files).all(
                    |(current, initial)| {
                        current.path == initial.path
                            && current.name == initial.name
                            && current.size == initial.size
                            && &current.assigned_keys
                                == initial
                                    .independent_assigned_keys
                                    .as_ref()
                                    .unwrap_or(&initial.assigned_keys)
                    },
                );
        let data_unchanged = files_unchanged
            && current_data.random_playback == self.initial_data.random_playback
            && current_data.layout_index == self.initial_data.layout_index
            && current_data.sync_selections == self.initial_data.sync_selections;

        !data_unchanged || self.name_input.read(cx).value() != self.preset_name.as_ref()
    }

    pub(super) fn mapped_key_count(&self) -> usize {
        self.files
            .iter()
            .flat_map(|file| &file.assigned_keys[self.keyboard_layout.index()])
            .map(|key| canonical_key_id(key).to_string())
            .collect::<BTreeSet<_>>()
            .len()
    }

    pub(super) fn current_preset_data(&self) -> KeyboardPresetData {
        self.preset_data(true)
    }

    fn preset_data(&self, include_sync_preview: bool) -> KeyboardPresetData {
        KeyboardPresetData {
            selected_keys: self.selected_keys.clone(),
            files: self
                .files
                .iter()
                .map(|file| KeyboardSoundData {
                    path: file.path.clone(),
                    name: file.name.clone(),
                    size: file.size.clone(),
                    assigned_keys: if include_sync_preview {
                        file.assigned_keys.clone()
                    } else {
                        file.edited_assigned_keys().clone()
                    },
                    independent_assigned_keys: if include_sync_preview && self.sync_selections {
                        file.independent_assigned_keys.clone()
                    } else {
                        None
                    },
                })
                .collect(),
            random_playback: self.preferred_playback_mode == PlaybackMode::Random,
            layout_index: self.keyboard_layout.index(),
            sync_selections: self.sync_selections,
        }
    }

    pub(super) fn save_validation_error(&self, name: &str) -> Option<SharedString> {
        if let Err(message) = crate::presets::validate_preset_name(name) {
            return Some(message.into());
        }

        if self.mapped_key_count() == 0 {
            return Some(
                "Assign a sound to at least one key in the active keyboard size before saving."
                    .into(),
            );
        }

        let has_assigned_sound = self.files.iter().any(|file| {
            !file.assigned_keys[self.keyboard_layout.index()].is_empty()
                && matches!(file.state, UploadState::Success | UploadState::Complete)
        });
        if !has_assigned_sound {
            return Some("Assign at least one successfully uploaded sound.".into());
        }

        None
    }
}
