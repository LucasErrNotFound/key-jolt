mod assignment;
mod attachments;
mod commands;
mod diagram;
mod discard_dialog;
mod events;
mod lifecycle;
mod model;
mod playback;
mod preset;
mod uploads;
mod view;

use self::model::{AudioFile, MouseButton};

use crate::features::preset_editor::PlaybackMode;
use crate::presets::MousePresetData;
use gpui_kit::component::input::InputState;
use gpui_kit::*;
use std::collections::HashMap;

pub(crate) use crate::features::preset_editor::{PreviewState, SaveStatus};
pub(crate) use events::MouseEditorEvent;

pub(crate) struct MouseEditorView {
    preset_id: Option<SharedString>,
    preset_name: SharedString,
    name_input: Entity<InputState>,
    _name_subscription: Subscription,
    initial_data: MousePresetData,
    selected_button: MouseButton,
    files: Vec<AudioFile>,
    next_file_id: u64,
    upload_tasks: HashMap<u64, Task<()>>,
    hovered_file: Option<u64>,
    playback_mode: PlaybackMode,
    save_status: SaveStatus,
}

impl EventEmitter<MouseEditorEvent> for MouseEditorView {}
