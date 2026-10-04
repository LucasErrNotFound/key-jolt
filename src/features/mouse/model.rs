use crate::features::preset_editor::{PreviewState, UploadState};

use gpui_kit::*;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    pub(super) fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Middle => 2,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Left => "Left click",
            Self::Right => "Right click",
            Self::Middle => "Middle · scroll wheel",
        }
    }
}

#[derive(Clone)]
pub(super) struct AudioFile {
    pub(super) id: u64,
    pub(super) path: PathBuf,
    pub(super) name: SharedString,
    pub(super) size: Option<SharedString>,
    pub(super) assigned_buttons: [bool; 3],
    pub(super) state: UploadState,
    pub(super) preview_state: PreviewState,
}

impl AudioFile {
    pub(super) fn new(id: u64, path: PathBuf, name: SharedString) -> Self {
        Self {
            id,
            path,
            name,
            size: None,
            assigned_buttons: [false; 3],
            state: UploadState::Uploading,
            preview_state: PreviewState::Stopped,
        }
    }
}
