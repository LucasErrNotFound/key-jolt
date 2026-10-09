use super::{KeyboardEditorEvent, KeyboardEditorView};

use super::assignment::maximum_assigned_sound_count;
use super::layout::KeyboardLayout;
use super::model::AudioFile;
use crate::features::preset_editor::{PlaybackMode, PreviewState, SaveStatus, UploadState};

use crate::presets::KeyboardPresetData;
use gpui_kit::component::input::{InputEvent, InputState};

use gpui_kit::*;
use std::collections::HashMap;

impl KeyboardEditorView {
    pub(crate) fn view(
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        preset_data: Option<KeyboardPresetData>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(preset_id, preset_name, preset_data, window, cx))
    }

    pub(crate) fn set_preview_state(
        &mut self,
        file_id: u64,
        state: PreviewState,
        cx: &mut Context<Self>,
    ) {
        if let Some(file) = self.files.iter_mut().find(|file| file.id == file_id) {
            file.preview_state = state.clone();
            cx.emit(KeyboardEditorEvent::PlaybackStateChanged { state });
            cx.notify();
        }
    }

    pub(crate) fn set_save_status(&mut self, status: SaveStatus, cx: &mut Context<Self>) {
        self.save_status = status;
        cx.notify();
    }

    pub(super) fn new(
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        preset_data: Option<KeyboardPresetData>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Preset name")
                .default_value(preset_name.as_ref())
        });
        let name_subscription =
            cx.subscribe_in(&name_input, window, |_, _, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            });
        let initial_data = preset_data.unwrap_or(KeyboardPresetData {
            selected_keys: [vec![], vec![], vec![]],
            files: Vec::new(),
            random_playback: false,
            layout_index: KeyboardLayout::Compact.index(),
            sync_selections: false,
        });
        let largest_sound_pool_size = maximum_assigned_sound_count(
            initial_data
                .files
                .iter()
                .map(|file| file.assigned_keys[initial_data.layout_index.min(2)].as_slice()),
        );
        let selected_keys = std::array::from_fn(|_| Vec::new());
        let files = initial_data
            .files
            .iter()
            .enumerate()
            .map(|(index, file)| AudioFile {
                id: index as u64,
                path: file.path.clone(),
                name: file.name.clone(),
                size: file.size.clone(),
                assigned_keys: file.assigned_keys.clone(),
                state: UploadState::Complete,
                preview_state: PreviewState::Stopped,
            })
            .collect::<Vec<_>>();
        let preferred_playback_mode = if initial_data.random_playback {
            PlaybackMode::Random
        } else {
            PlaybackMode::Sequential
        };
        let playback_mode = if largest_sound_pool_size < 2 {
            PlaybackMode::Sequential
        } else {
            preferred_playback_mode
        };
        let next_file_id = files.len() as u64;
        let keyboard_layout = match initial_data.layout_index {
            0 => KeyboardLayout::FullSize,
            1 => KeyboardLayout::Tkl,
            _ => KeyboardLayout::Compact,
        };
        let sync_selections = initial_data.sync_selections;

        Self {
            preset_id,
            preset_name,
            name_input,
            _name_subscription: name_subscription,
            initial_data,
            selected_keys,
            sync_selections,
            files,
            next_file_id,
            upload_tasks: HashMap::new(),
            hovered_file: None,
            playback_mode,
            preferred_playback_mode,
            keyboard_layout,
            save_status: SaveStatus::Idle,
        }
    }
}
