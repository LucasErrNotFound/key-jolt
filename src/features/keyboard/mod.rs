mod assignment;
mod attachments;
mod audio_details;
mod canvas;
mod commands;
mod discard_dialog;
mod drag_selection;
mod events;
mod header;
mod inspector;
mod layout;
mod lifecycle;
mod model;
mod playback;
mod preset;
mod selection;
mod selection_math;
mod sound_groups;
mod uploads;
mod view;
mod waveform;
mod waveform_math;

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

    canvas_geometry: drag_selection::SharedCanvasGeometry,
    canvas_focus: FocusHandle,
    selection_drag: Option<drag_selection::SelectionDrag>,
    suppress_key_click: bool,
    show_sound_catalog: bool,
    audio_details: HashMap<u64, audio_details::AudioDetails>,
    analysis_job: Option<audio_details::AnalysisJob>,

    playback_mode: PlaybackMode,
    preferred_playback_mode: PlaybackMode,
    keyboard_layout: KeyboardLayout,
    save_status: SaveStatus,
}

impl EventEmitter<KeyboardEditorEvent> for KeyboardEditorView {}

#[cfg(test)]
mod tests;
