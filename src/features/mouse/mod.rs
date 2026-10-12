mod assignment;
mod assignment_math;
mod attachments;
mod audio_details;
mod bulk_assignment;
mod commands;
mod diagram;
mod diagram_geometry;
mod discard_dialog;
mod events;
mod header;
mod inspector;
mod lifecycle;
mod model;
mod playback;
mod preset;
mod uploads;
mod view;
mod wheel;

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
    bulk_undo: Option<assignment_math::BulkAssignmentUndo>,
    selected_button: MouseButton,
    files: Vec<AudioFile>,
    next_file_id: u64,
    upload_tasks: HashMap<u64, Task<()>>,
    show_sound_catalog: bool,
    audio_details: HashMap<u64, audio_details::AudioDetails>,
    analysis_job: Option<audio_details::AnalysisJob>,
    wheel_offset: f32,
    wheel_task: Option<Task<()>>,
    playback_mode: PlaybackMode,
    save_status: SaveStatus,
}

impl EventEmitter<MouseEditorEvent> for MouseEditorView {}
