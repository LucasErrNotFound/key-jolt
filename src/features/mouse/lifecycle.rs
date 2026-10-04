use super::{MouseEditorEvent, MouseEditorView};

use super::model::{AudioFile, MouseButton};

use crate::features::preset_editor::{PlaybackMode, PreviewState, SaveStatus, UploadState};

use crate::presets::MousePresetData;
use gpui_kit::component::input::{InputEvent, InputState};

use gpui_kit::*;
use std::collections::HashMap;

impl MouseEditorView {
    pub(crate) fn view(
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        preset_data: Option<MousePresetData>,
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
            cx.emit(MouseEditorEvent::PlaybackStateChanged { state });
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
        preset_data: Option<MousePresetData>,
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
        let mut initial_data = preset_data.unwrap_or(MousePresetData {
            files: Vec::new(),
            random_playback: false,
        });
        let assigned_sound_count = initial_data
            .files
            .iter()
            .filter(|file| file.assigned_buttons[MouseButton::Left.index()])
            .count();
        if assigned_sound_count < 2 {
            initial_data.random_playback = false;
        }
        let files = initial_data
            .files
            .iter()
            .enumerate()
            .map(|(index, file)| AudioFile {
                id: index as u64,
                path: file.path.clone(),
                name: file.name.clone(),
                size: file.size.clone(),
                assigned_buttons: file.assigned_buttons,
                state: UploadState::Complete,
                preview_state: PreviewState::Stopped,
            })
            .collect::<Vec<_>>();
        let playback_mode = if initial_data.random_playback {
            PlaybackMode::Random
        } else {
            PlaybackMode::Sequential
        };

        let next_file_id = files.len() as u64;

        Self {
            preset_id,
            preset_name,
            name_input,
            _name_subscription: name_subscription,
            initial_data,
            selected_button: MouseButton::Left,
            files,
            next_file_id,
            upload_tasks: HashMap::new(),
            hovered_file: None,
            playback_mode,
            save_status: SaveStatus::Idle,
        }
    }
}
