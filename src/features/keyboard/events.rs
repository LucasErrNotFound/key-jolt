use crate::features::preset_editor::PreviewState;
use crate::presets::KeyboardPresetData;
use gpui_kit::*;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) enum KeyboardEditorEvent {
    BackRequested,
    PreviewRequested {
        file_id: u64,
        path: PathBuf,
    },
    StopPreviewRequested {
        file_id: Option<u64>,
    },
    PlaybackStateChanged {
        state: PreviewState,
    },
    SaveRequested {
        preset_id: Option<SharedString>,
        preset_name: SharedString,
        summary: SharedString,
        data: KeyboardPresetData,
    },
}
