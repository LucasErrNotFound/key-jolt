mod assignment;
mod attachments;
mod canvas;
mod commands;
mod discard_dialog;
mod events;
mod layout;
mod lifecycle;
mod model;
mod playback;
mod preset;
mod selection;
mod uploads;
mod view;

use self::layout::KeyboardLayout;
use self::model::AudioFile;
use crate::features::preset_editor::PlaybackMode;
use crate::presets::KeyboardPresetData;
use gpui_kit::component::input::InputState;
use gpui_kit::*;
use std::collections::HashMap;

pub(crate) use crate::features::preset_editor::{PreviewState, SaveStatus};
pub(crate) use events::KeyboardEditorEvent;

pub(crate) struct KeyboardEditorView {
    preset_id: Option<SharedString>,
    preset_name: SharedString,
    name_input: Entity<InputState>,
    _name_subscription: Subscription,
    initial_data: KeyboardPresetData,

    selected_keys: [Vec<&'static str>; 3],
    sync_selections: bool,

    files: Vec<AudioFile>,
    next_file_id: u64,
    upload_tasks: HashMap<u64, Task<()>>,

    hovered_file: Option<u64>,

    playback_mode: PlaybackMode,
    preferred_playback_mode: PlaybackMode,
    keyboard_layout: KeyboardLayout,
    save_status: SaveStatus,
}

impl EventEmitter<KeyboardEditorEvent> for KeyboardEditorView {}

#[cfg(test)]
mod tests;
